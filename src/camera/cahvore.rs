use crate::{camera::model::*, util::vec_to_str, vector::Vector};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PupilType {
    Perspective,
    Fisheye,
    General,
}

impl PupilType {
    pub fn to_u8(&self) -> u8 {
        match &self {
            PupilType::Perspective => 0,
            PupilType::Fisheye => 1,
            PupilType::General => 2,
        }
    }
}

// For use when indexing a vector of components during (de)serialization
pub mod components {
    pub const CENTER: usize = 0;
    pub const AXIS: usize = 1;
    pub const HORIZONTAL: usize = 2;
    pub const VERTICAL: usize = 3;
    pub const OPTICAL: usize = 4;
    pub const RADIAL: usize = 5;
    pub const ENTRANCE: usize = 6;
    pub const PUPIL_TYPE: usize = 7;
    pub const LINEARITY: usize = 8;
}

pub mod linearity {
    pub static LINEARITY_FISHEYE: f64 = 0.0;
    pub static LINEARITY_PERSPECTIVE: f64 = 1.0;
    pub static LINEARITY_GENERAL: f64 = 2.0;
}

const LIN_EPS: f64 = 1e-10;

pub static CHIP_LIMIT: f64 = 1e-8;
pub static NEWTON_ITERATION_MAX: usize = 100;

pub static TOLERANCE: f64 = 1e-12;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cahvore {
    // Camera center vector C
    #[serde(with = "crate::vector::vector_format")]
    pub c: Vector,

    // Camera axis unit vector A
    #[serde(with = "crate::vector::vector_format")]
    pub a: Vector,

    // Horizontal information vector H
    #[serde(with = "crate::vector::vector_format")]
    pub h: Vector,

    // Vertical information vector V
    #[serde(with = "crate::vector::vector_format")]
    pub v: Vector,

    // Optical axis unit vector O
    #[serde(with = "crate::vector::vector_format")]
    pub o: Vector,

    // Radial lens distortion coefficients
    #[serde(with = "crate::vector::vector_format")]
    pub r: Vector,

    #[serde(with = "crate::vector::vector_format")]
    pub e: Vector,

    pub pupil_type: PupilType,

    pub linearity: f64,
}

impl Default for Cahvore {
    fn default() -> Self {
        Cahvore {
            c: Vector::default(),
            a: Vector::default(),
            h: Vector::default(),
            v: Vector::default(),
            o: Vector::default(),
            r: Vector::default(),
            e: Vector::default(),
            pupil_type: PupilType::Perspective,
            linearity: linearity::LINEARITY_PERSPECTIVE,
        }
    }
}

impl Cahvore {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        c: Vector,
        a: Vector,
        h: Vector,
        v: Vector,
        o: Vector,
        r: Vector,
        e: Vector,
        pupil_type: PupilType, //PupilType::General
        linearity: f64,        //LINEARITY_FISHEYE
    ) -> Self {
        Cahvore {
            c,
            a,
            h,
            v,
            o,
            r,
            e,
            pupil_type,
            linearity,
        }
    }

    pub fn new_cahv(c: Vector, a: Vector, h: Vector, v: Vector) -> Self {
        Cahvore {
            c,
            a,
            h,
            v,
            o: a,
            r: Vector::new(0.0, 0.0, 0.0),
            e: Vector::new(0.0, 0.0, 0.0),
            pupil_type: PupilType::Perspective,
            linearity: linearity::LINEARITY_PERSPECTIVE,
        }
    }

    pub fn new_cahvor(c: Vector, a: Vector, h: Vector, v: Vector, o: Vector, r: Vector) -> Self {
        Cahvore {
            c,
            a,
            h,
            v,
            o,
            r,
            e: Vector::new(0.0, 0.0, 0.0),
            pupil_type: PupilType::Perspective,
            linearity: linearity::LINEARITY_PERSPECTIVE,
        }
    }

    //  Adapted from https://github.com/digimatronics/ComputerVision/blob/master/src/vw/Camera/CAHVOREModel.cc
    pub fn linearize(
        &self,
        src_width: usize,
        src_height: usize,
        out_width: usize,
        out_height: usize,
    ) -> Result<Cahvore> {
        // A perspective model can't reach 90 degrees; keep the half-FOV safely below it.
        let max_half_fov = std::f64::consts::PI * 85.0 / 180.0;
        let min_fov = true; // true: crop to the narrowest edge angle; false: widest

        let sw = src_width as f64;
        let sh = src_height as f64;
        let mid_x = (sw - 1.0) / 2.0;
        let mid_y = (sh - 1.0) / 2.0;

        // Boresight of the output camera: the ray through the middle of the source image.
        let a = self
            .ls_to_look_vector(&ImageCoordinate {
                line: mid_y,
                sample: mid_x,
            })?
            .look_direction
            .normalized();

        // Right/down basis perpendicular to `a`, seeded with the source camera's "down".
        let down_in = self.a.cross_product(&self.h).normalized();
        let rt = down_in.cross_product(&a).normalized();
        let dn = a.cross_product(&rt).normalized();

        // (sample, line) points along the left/right edges and the top/bottom edges.
        let left_right = [
            (0.0, 0.0),
            (0.0, mid_y),
            (0.0, sh - 1.0),
            (sw - 1.0, 0.0),
            (sw - 1.0, mid_y),
            (sw - 1.0, sh - 1.0),
        ];
        let top_bottom = [
            (0.0, 0.0),
            (mid_x, 0.0),
            (sw - 1.0, 0.0),
            (0.0, sh - 1.0),
            (mid_x, sh - 1.0),
            (sw - 1.0, sh - 1.0),
        ];

        // Range of cos(angle from `a`) after projecting each edge ray into the plane
        // perpendicular to `perp`. Points that fail to unproject are skipped.
        let cos_range = |pts: &[(f64, f64)], perp: &Vector| -> Option<(f64, f64)> {
            let mut range: Option<(f64, f64)> = None;
            for &(sample, line) in pts {
                let Ok(lv) = self.ls_to_look_vector(&ImageCoordinate { line, sample }) else {
                    continue;
                };
                let d = lv.look_direction.normalized();
                let flat = d.subtract(&perp.scale(perp.dot_product(&d))).normalized();
                let cs = a.dot_product(&flat).clamp(-1.0, 1.0);
                range = Some(match range {
                    None => (cs, cs),
                    Some((lo, hi)) => (lo.min(cs), hi.max(cs)),
                });
            }
            range
        };

        let (h_lo, h_hi) = cos_range(&left_right, &dn)
            .ok_or_else(|| anyhow!("linearize: no left/right edge points could be unprojected"))?;
        let (v_lo, v_hi) = cos_range(&top_bottom, &rt)
            .ok_or_else(|| anyhow!("linearize: no top/bottom edge points could be unprojected"))?;

        let (mut cos_h, mut cos_v) = if min_fov { (h_hi, v_hi) } else { (h_lo, v_lo) };

        // cos is decreasing on [0, pi], so this limits the half-FOV to max_half_fov.
        let min_cos = max_half_fov.cos();
        cos_h = cos_h.max(min_cos);
        cos_v = cos_v.max(min_cos);

        let sin2_h = 1.0 - cos_h * cos_h;
        let sin2_v = 1.0 - cos_v * cos_v;
        if sin2_h < 1e-12 || sin2_v < 1e-12 {
            return Err(anyhow!("linearize: degenerate field of view"));
        }

        // Pixel scale: half the output size times cot(half-FOV).
        let hs = (out_width as f64 / 2.0) * cos_h / sin2_h.sqrt();
        let vs = (out_height as f64 / 2.0) * cos_v / sin2_v.sqrt();

        // Principal point at the center of the output image (width -> H, height -> V).
        let hc = (out_width as f64 - 1.0) / 2.0;
        let vc = (out_height as f64 - 1.0) / 2.0;

        Ok(Cahvore {
            c: self.c,
            a,
            h: a.scale(hc).add(&rt.scale(hs)),
            v: a.scale(vc).add(&dn.scale(vs)),
            o: a, // no distortion, so O = A
            r: Vector::new(0.0, 0.0, 0.0),
            e: Vector::new(0.0, 0.0, 0.0),
            pupil_type: PupilType::Perspective,
            linearity: linearity::LINEARITY_PERSPECTIVE,
        })
    }

    fn invert_distortion(&self, chip: f64) -> Result<f64> {
        if self.r.norm() < 1e-15 {
            Ok(chip)
        } else {
            let mut chi = chip;

            for x in 0..=NEWTON_ITERATION_MAX {
                let chi2 = chi * chi;
                let chi4 = chi2 * chi2;

                let poly = 1.0 + self.r.x + self.r.y * chi2 + self.r.z * chi4;
                let f = chi * poly - chip;

                if f.abs() < TOLERANCE {
                    return Ok(chi);
                }

                let fprime = 1.0 + self.r.x + 3.0 * self.r.y * chi2 + 5.0 * self.r.z * chi4;

                if fprime.abs() < 1e-15 {
                    return Err(anyhow!(
                        "Derivatite too small at iteration {}: f'(X) = {}",
                        x,
                        fprime
                    ));
                }

                chi -= f / fprime;

                if chi.abs() > 1e6 {
                    return Err(anyhow!(
                        "Divergence detected at iteration {}: X = {}",
                        x,
                        chi
                    ));
                }
            }
            Ok(chi)
        }
    }

    pub fn model_type(&self) -> ModelType {
        ModelType::CAHVORE
    }

    pub fn c(&self) -> Vector {
        self.c
    }

    pub fn a(&self) -> Vector {
        self.a
    }

    pub fn h(&self) -> Vector {
        self.h
    }

    pub fn v(&self) -> Vector {
        self.v
    }

    pub fn o(&self) -> Vector {
        self.o
    }

    pub fn r(&self) -> Vector {
        self.r
    }

    pub fn e(&self) -> Vector {
        self.e
    }

    pub fn f(&self) -> f64 {
        self.a.cross_product(&self.h).len()
    }

    fn theta_from_chi(chi: f64, lin: f64) -> Result<f64> {
        if lin > LIN_EPS {
            Ok((lin * chi).atan() / lin)
        } else if lin < -LIN_EPS {
            let x = lin * chi;
            if x.abs() > 1.0 {
                return Err(anyhow!("chi out of range for sin-type linearity"));
            }
            Ok(x.asin() / lin)
        } else {
            Ok(chi)
        }
    }

    fn chi_from_theta(theta: f64, lin: f64) -> Result<f64> {
        if lin > LIN_EPS {
            if lin * theta >= std::f64::consts::FRAC_PI_2 {
                return Err(anyhow!("Out of bounds: theta beyond the perspective limit"));
            }
            Ok((lin * theta).tan() / lin)
        } else if lin < -LIN_EPS {
            if (lin * theta).abs() > std::f64::consts::FRAC_PI_2 {
                return Err(anyhow!("Out of bounds: theta beyond the sin-mapping limit"));
            }
            Ok((lin * theta).sin() / lin)
        } else {
            Ok(theta)
        }
    }

    pub fn ls_to_look_vector(&self, coordinate: &ImageCoordinate) -> Result<LookVector> {
        let a = self.a.normalized();
        let o = self.o.normalized();

        // 1. CAHV ray for the pixel. Exact: no assumption that H' is perpendicular to V'.
        let f = self.v - a * coordinate.line;
        let g = self.h - a * coordinate.sample;
        let denom = a.dot_product(&self.v.cross_product(&self.h));
        if denom.abs() < 1e-12 {
            return Err(anyhow!("Numerical error: degenerate H/V/A"));
        }
        let rp = f.cross_product(&g) / denom;

        // 2. Decompose about the OPTICAL axis O (distortion is centered on O, not A).
        let zetap = rp.dot_product(&o);
        if zetap <= 0.0 {
            return Err(anyhow!(
                "Out of bounds: ray is at or beyond 90 deg from optical axis"
            ));
        }
        let lambdap = rp - o * zetap;
        let chip = lambdap.norm() / zetap;

        // On-axis: no distortion, no pupil shift.
        if chip < CHIP_LIMIT {
            return Ok(LookVector {
                origin: self.c,
                look_direction: o,
            });
        }

        // 3. Undo radial distortion, then undo the linearity mapping.
        let chi = self.invert_distortion(chip)?;
        let theta = Cahvore::theta_from_chi(chi, self.linearity)?;

        let (sin_t, cos_t) = theta.sin_cos();

        // 4. Entrance-pupil shift along O (moves the ray origin, not the direction).
        let s = if theta.abs() > 1e-8 {
            (theta / sin_t - 1.0) * (self.e.x + self.e.y * theta * theta + self.e.z * theta.powi(4))
        } else {
            0.0
        };
        let origin = self.c + o * s;

        // 5. Ray of incidence: unit length.
        let direction = lambdap.normalized() * sin_t + o * cos_t;

        Ok(LookVector {
            origin,
            look_direction: direction,
        })
    }

    pub fn ls_to_look_vector_old(&self, coordinate: &ImageCoordinate) -> Result<LookVector> {
        let line = coordinate.line;
        let samp = coordinate.sample;

        let f = self.v.subtract(&self.a.scale(line));
        let g = self.h.subtract(&self.a.scale(samp));
        let w3 = f.cross_product(&g);

        let inv_adotf = 1.0 / self.a.dot_product(&self.v.cross_product(&self.h));
        let rp = w3.scale(inv_adotf);

        let zetap = rp.dot_product(&self.o);

        let lambdap = rp.subtract(&self.o.scale(zetap));
        let chip = lambdap.len() / zetap;

        let (center_point, ray_of_incidence) = match chip < CHIP_LIMIT {
            true => (self.c, self.o),
            false => {
                let chi = match self.invert_distortion(chip) {
                    Ok(chi) => chi,
                    Err(why) => panic!("Failed to invert distortion: {}", why),
                };

                let linchi = self.linearity * chi;
                let theta = if self.linearity < -EPSILON {
                    linchi.asin() / self.linearity
                } else if self.linearity < EPSILON {
                    linchi.atan() / self.linearity
                } else {
                    chi
                };

                let theta2 = theta * theta;
                let theta3 = theta2 * theta;
                let theta4 = theta3 * theta;

                // compute the shift of the entrance pupil
                let s = ((theta / theta.sin()) - 1.0)
                    * (self.e.x + self.e.y * theta2 + self.e.z * theta4);

                let center_point = self.c.add(&self.o.scale(s));

                let f2 = lambdap.normalized().scale(theta.sin());
                let g = self.o.scale(theta.cos());
                let ray_of_incidence = f2.add(&g);

                (center_point, ray_of_incidence)
            }
        };

        Ok(LookVector {
            origin: center_point,
            look_direction: ray_of_incidence,
        })
    }

    pub fn apply_distortion(chi: f64, r_params: &Vector) -> f64 {
        let chi2 = chi * chi;
        let chi4 = chi2 * chi2;
        chi * (1.0 + r_params.x + r_params.y * chi2 + r_params.z * chi4)
    }

    pub fn xyz_to_ls(&self, xyz: &Vector, infinity: bool) -> Result<ImageCoordinate> {
        let a = self.a.normalized();
        let o = self.o.normalized();

        // Vector from the camera center to the point. For points at infinity the
        // input is a direction, and the pupil shift is negligible.
        let p_c = if infinity { *xyz } else { xyz - &self.c };
        let e = if infinity {
            Vector::new(0.0, 0.0, 0.0)
        } else {
            self.e
        };

        // Decompose about the optical axis O.
        let zeta = p_c.dot_product(&o);
        let lambda = p_c - o * zeta;
        let lambda_mag = lambda.norm();

        if p_c.norm() < 1e-12 {
            return Err(anyhow!("Numerical error: point is at the camera center"));
        }

        // Solve for theta, the angle of the incident ray from O, including the
        // entrance-pupil shift s(theta) = (theta/sin(theta) - 1) * E(theta).
        //   F(theta) = zeta*sin(t) - lambda*cos(t) - (t - sin(t)) * E(t) = 0
        let mut theta = lambda_mag.atan2(zeta);

        if lambda_mag > 1e-12 && e.norm() > 1e-15 {
            let mut converged = false;
            for _ in 0..NEWTON_ITERATION_MAX {
                let (sin_t, cos_t) = theta.sin_cos();
                let t2 = theta * theta;
                let t3 = t2 * theta;
                let t4 = t2 * t2;

                let ee = e.x + e.y * t2 + e.z * t4;
                let de = 2.0 * e.y * theta + 4.0 * e.z * t3;

                let f = zeta * sin_t - lambda_mag * cos_t - (theta - sin_t) * ee;
                let fp =
                    zeta * cos_t + lambda_mag * sin_t - (1.0 - cos_t) * ee - (theta - sin_t) * de;

                if fp.abs() < 1e-15 {
                    return Err(anyhow!("Newton iteration: derivative too small"));
                }

                let dtheta = f / fp;
                theta -= dtheta;

                if dtheta.abs() < TOLERANCE {
                    converged = true;
                    break;
                }
            }
            if !converged {
                return Err(anyhow!("CAHVORE: theta Newton iteration did not converge"));
            }
        }

        // Ray from the optical axis: undistorted chi, then radial distortion.
        // rp is proportional to O + chi_dist * lambda_hat, so there is no division
        // by chi and no special case near the axis.
        let rp = if lambda_mag < 1e-12 || theta.abs() < CHIP_LIMIT {
            o
        } else {
            let chi = Cahvore::chi_from_theta(theta, self.linearity)?;
            let chi_dist = Cahvore::apply_distortion(chi, &self.r);
            o + lambda * (chi_dist / lambda_mag)
        };

        // Standard CAHV projection of the (distorted) ray.
        let alpha = rp.dot_product(&a);
        if alpha <= 1e-12 {
            return Err(anyhow!(
                "Out of bounds: point projects behind the image plane"
            ));
        }

        Ok(ImageCoordinate {
            sample: rp.dot_product(&self.h) / alpha,
            line: rp.dot_product(&self.v) / alpha,
        })
    }

    pub fn pixel_angle_horiz(&self) -> f64 {
        let a = self.v.dot_product(&self.a);
        let s = self.a.scale(a);
        let f = self.v.subtract(&s).len();
        (1.0 / f).atan()
    }

    pub fn pixel_angle_vert(&self) -> f64 {
        let a = self.h.dot_product(&self.a);
        let s = self.a.scale(a);
        let f = self.h.subtract(&s).len();
        (1.0 / f).atan()
    }

    pub fn scale(&self, sx: f64, sy: f64) -> Self {
        Self {
            c: self.c,
            a: self.a,
            h: self.h * sx,
            v: self.v * sy,
            o: self.o,
            r: self.r,
            e: self.e,
            pupil_type: self.pupil_type.clone(),
            linearity: self.linearity,
        }
    }

    pub fn shift(&self, dx: f64, dy: f64) -> Self {
        Self {
            c: self.c,
            a: self.a,
            h: self.h + dx * self.a,
            v: self.v + dy * self.a,
            o: self.o,
            r: self.r,
            e: self.e,
            pupil_type: self.pupil_type.clone(),
            linearity: self.linearity,
        }
    }

    pub fn translate(&self, translation: Vector) -> Self {
        Self {
            c: self.c + translation,
            a: self.a,
            h: self.h,
            v: self.v,
            o: self.o,
            r: self.r,
            e: self.e,
            pupil_type: self.pupil_type.clone(),
            linearity: self.linearity,
        }
    }

    pub fn serialize(&self) -> String {
        format!(
            "{};{};{};{};{};{};{};{};{}",
            vec_to_str(&self.c.to_vec()),
            vec_to_str(&self.a.to_vec()),
            vec_to_str(&self.h.to_vec()),
            vec_to_str(&self.v.to_vec()),
            vec_to_str(&self.o.to_vec()),
            vec_to_str(&self.r.to_vec()),
            vec_to_str(&self.e.to_vec()),
            self.pupil_type.to_u8(),
            self.linearity
        )
    }
}
