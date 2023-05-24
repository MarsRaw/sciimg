use crate::{
    camera::cahv::*,
    //matrix::Matrix,
    camera::model::*,
    max,
    min,
    util::vec_to_str,
    vector::Vector,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PupilType {
    Perspective,
    Fisheye,
    General,
}

pub static CHIP_LIMIT: f64 = 1e-8;
pub static NEWTON_ITERATION_MAX: usize = 100;

pub static LINEARITY_PERSPECTIVE: f64 = 1.0;
pub static LINEARITY_FISHEYE: f64 = 0.0;

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
            pupil_type: PupilType::General,
            linearity: LINEARITY_FISHEYE,
        }
    }
}

impl CameraModelTrait for Cahvore {
    fn model_type(&self) -> ModelType {
        ModelType::CAHVORE
    }

    fn c(&self) -> Vector {
        self.c
    }

    fn a(&self) -> Vector {
        self.a
    }

    fn h(&self) -> Vector {
        self.h
    }

    fn v(&self) -> Vector {
        self.v
    }

    fn o(&self) -> Vector {
        self.o
    }

    fn r(&self) -> Vector {
        self.r
    }

    fn e(&self) -> Vector {
        self.e
    }

    fn box_clone(&self) -> CameraModelType {
        Box::new((*self).clone())
    }

    fn f(&self) -> f64 {
        self.a.cross_product(&self.h).len()
    }

    // Adapted from https://github.com/NASA-AMMOS/VICAR/blob/master/vos/java/jpl/mipl/mars/pig/PigCoreCAHVORE.java
    fn ls_to_look_vector(&self, coordinate: &ImageCoordinate) -> Result<LookVector> {
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
                let mut chi = chip;

                for x in 1..=NEWTON_ITERATION_MAX {
                    let chi2 = chi * chi;
                    let chi3 = chi2 * chi;
                    let chi4 = chi3 * chi;
                    let chi5 = chi4 * chi;

                    let deriv =
                        (1.0 + self.r.x) + (3.0 * self.r.y * chi2) + (5.0 * self.r.z * chi4);

                    let dchi = if deriv == 0.0 {
                        0.0
                    } else {
                        ((1.0 + self.r.x) * chi)
                            + (self.r.y * chi3)
                            + ((self.r.z * chi5) - chip) / deriv
                    };

                    chi -= dchi;

                    if dchi.abs() < CHIP_LIMIT {
                        break;
                    }

                    if x >= NEWTON_ITERATION_MAX {
                        eprintln!("CAHVORE: Too many iterations without sufficient convergence");
                        break;
                    }
                }

                let linchi = self.linearity * chi;
                let theta = if self.linearity < (-1.0 * EPSILON) {
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

    // Adapted from https://github.com/NASA-AMMOS/VICAR/blob/master/vos/java/jpl/mipl/mars/pig/PigCoreCAHVORE.java
    fn xyz_to_ls(&self, xyz: &Vector, _infinity: bool) -> ImageCoordinate {
        let p_c = xyz.subtract(&self.c);
        let zeta = p_c.dot_product(&self.o);
        let f = self.o.scale(zeta);
        let lambda = p_c.subtract(&f);
        let lamda_mag = lambda.len();

        let mut theta = lamda_mag.atan2(zeta);

        for x in 1..=NEWTON_ITERATION_MAX {
            let cos_theta = theta.cos();
            let sin_theta = theta.sin();
            let theta2 = theta * theta;
            let theta3 = theta2 * theta;
            let theta4 = theta3 * theta;

            let upsilon = (zeta * cos_theta) + (lamda_mag * sin_theta)
                - ((1.0 - cos_theta) * (self.e.x + self.e.y * theta2 + self.e.z * theta4))
                - ((theta - sin_theta)
                    * (self.e.x + 2.0 * self.e.y * theta + 4.0 * self.e.z * theta3));
            let dtheta = ((zeta * sin_theta - lamda_mag * cos_theta)
                - (theta - sin_theta) * (self.e.x + self.e.y * theta2 + self.e.z * theta4))
                / upsilon;
            theta -= dtheta;

            if dtheta.abs() < CHIP_LIMIT {
                break;
            }

            if x >= NEWTON_ITERATION_MAX {
                eprintln!("CAHVORE: Too many iterations without sufficient convergence");
                break;
            }
        }

        if theta * self.linearity.abs() > (std::f64::consts::PI / 2.0) {
            eprintln!("CAVHORE: theta out of bounds");
            return ImageCoordinate {
                sample: 0.0,
                line: 0.0,
            };
            //panic!("CAVHORE: theta out of bounds");
        }

        let rp = if theta < CHIP_LIMIT {
            p_c
        } else {
            let linth = self.linearity * theta;
            let chi = if self.linearity < (-1.0 * EPSILON) {
                linth.sin() / self.linearity
            } else if self.linearity > EPSILON {
                linth.tan() / self.linearity
            } else {
                theta
            };

            let chi2 = chi * chi;
            let chi3 = chi2 * chi;
            let chi4 = chi3 * chi;

            let zetap = lamda_mag / chi;

            let mu = self.r.x + self.r.y * chi2 + self.r.z * chi4;

            let f = self.o.scale(zetap);
            let g = lambda.scale(1.0 + mu);

            f.add(&g)
        };

        let alpha = rp.dot_product(&self.a);
        let beta = rp.dot_product(&self.h);
        let gamma = rp.dot_product(&self.v);

        ImageCoordinate {
            sample: beta / alpha,
            line: gamma / alpha,
        }
    }

    fn pixel_angle_horiz(&self) -> f64 {
        let a = self.v.dot_product(&self.a);
        let s = self.a.scale(a);
        let f = self.v.subtract(&s).len();
        (1.0 / f).atan()
    }

    fn pixel_angle_vert(&self) -> f64 {
        let a = self.h.dot_product(&self.a);
        let s = self.a.scale(a);
        let f = self.h.subtract(&s).len();
        (1.0 / f).atan()
    }

    fn serialize(&self) -> String {
        format!(
            "{};{};{};{};{};{};{};{};0.0",
            vec_to_str(&self.c.to_vec()),
            vec_to_str(&self.a.to_vec()),
            vec_to_str(&self.h.to_vec()),
            vec_to_str(&self.v.to_vec()),
            vec_to_str(&self.o.to_vec()),
            vec_to_str(&self.r.to_vec()),
            vec_to_str(&self.e.to_vec()),
            self.linearity
        )
    }
}

//  Adapted from https://github.com/digimatronics/ComputerVision/blob/master/src/vw/Camera/CAHVOREModel.cc
pub fn linearize(
    camera_model: &Cahvore,
    cahvor_width: usize,
    cahvor_height: usize,
    cahv_width: usize,
    cahv_height: usize,
) -> Cahv {
    let limfov = std::f64::consts::PI * (3.0 / 4.0);
    let minfov = true;

    let mut output_camera = Cahv {
        c: camera_model.c,
        a: Vector::default(),
        h: Vector::default(),
        v: Vector::default(),
    };

    let hpts = vec![
        Vector::default(),
        Vector::new(0.0, (cahvor_height as f64 - 1.0) / 2.0, 0.0),
        Vector::new(0.0, cahvor_height as f64 - 1.0, 0.0),
        Vector::new(cahvor_width as f64 - 1.0, 0.0, 0.0),
        Vector::new(
            cahvor_width as f64 - 1.0,
            (cahvor_height as f64 - 1.0) / 2.0,
            0.0,
        ),
        Vector::new(cahvor_width as f64, cahvor_height as f64, 0.0)
            .subtract(&Vector::new(1.0, 1.0, 0.0)),
    ];

    let vpts = vec![
        Vector::default(),
        Vector::new((cahvor_width as f64 - 1.0) / 2.0, 0.0, 0.0),
        Vector::new(cahvor_width as f64 - 1.0, 0.0, 0.0),
        Vector::new(0.0, cahvor_height as f64 - 1.0, 0.0),
        Vector::new(
            (cahvor_width as f64 - 1.0) / 2.0,
            cahvor_height as f64 - 1.0,
            0.0,
        ),
        Vector::new(cahvor_width as f64, cahvor_height as f64, 0.0)
            .subtract(&Vector::new(1.0, 1.0, 0.0)),
    ];

    let p2_sample = (cahvor_width as f64 - 1.0) / 2.0;
    let p2_line = (cahvor_height as f64 - 1.0) / 2.0;

    output_camera.a = camera_model
        .ls_to_look_vector(&ImageCoordinate {
            line: p2_line,
            sample: p2_sample,
        })
        .expect("Failed to project boresight")
        .look_direction;

    let mut dn = camera_model.a.cross_product(&camera_model.h);
    //let mut rt = dn.cross_product(&camera_model.a).normalized();
    dn = dn.normalized();

    let mut rt = dn.cross_product(&output_camera.a);
    dn = output_camera.a.cross_product(&rt).normalized();
    rt = rt.normalized();

    let mut hmin = 1.0;
    let mut hmax = -1.0;
    for local in hpts.iter() {
        if let Ok(lv) = camera_model.ls_to_look_vector(&ImageCoordinate {
            line: local.y,
            sample: local.x,
        }) {
            let cs = output_camera.a.dot_product(
                &lv.look_direction
                    .subtract(&dn.scale(dn.dot_product(&lv.look_direction)))
                    .normalized(),
            );
            hmin = min!(hmin, cs);
            hmax = max!(hmax, cs);
        }
    }

    let mut vmin = 1.0;
    let mut vmax = -1.0;
    for local in vpts.iter() {
        if let Ok(lv) = camera_model.ls_to_look_vector(&ImageCoordinate {
            line: local.y,
            sample: local.x,
        }) {
            let cs = output_camera.a.dot_product(
                &lv.look_direction
                    .subtract(&rt.scale(rt.dot_product(&lv.look_direction)))
                    .normalized(),
            );
            vmin = min!(vmin, cs);
            vmax = max!(vmax, cs);
        }
    }

    let cahv_image_size = Vector::new(cahv_height as f64, cahv_width as f64, 0.0);

    let mut cosines = Vector::new(0.0, 0.0, 1.0);
    if minfov {
        cosines.x = hmax;
        cosines.y = vmax;
    } else {
        cosines.x = hmin;
        cosines.y = vmin;
    }

    if cosines.x.acos() > limfov {
        cosines.x = limfov.cos();
    }
    if cosines.y.acos() > limfov {
        cosines.y = limfov.cos();
    }

    let scalars = cahv_image_size.scale(0.5).multiply(&cosines).divide(
        &Vector::new(1.0, 1.0, 0.0)
            .subtract(&cosines.multiply(&cosines))
            .sqrt(),
    );

    let centers = cahv_image_size
        .subtract(&Vector::new(1.0, 1.0, 0.0))
        .scale(0.5);

    output_camera.h = output_camera.a.scale(centers.x).add(&rt.scale(scalars.x));
    output_camera.v = output_camera.a.scale(centers.y).add(&dn.scale(scalars.y));

    output_camera
}
