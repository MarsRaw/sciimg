use crate::{
    camera::cahv::*, camera::model::*, error, matrix::Matrix, max, min, util::vec_to_str,
    vector::Vector,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cahvor {
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
}

impl Cahvor {
    pub fn default() -> Self {
        Cahvor {
            c: Vector::default(),
            a: Vector::default(),
            h: Vector::default(),
            v: Vector::default(),
            o: Vector::default(),
            r: Vector::default(),
        }
    }

    pub fn hc(&self) -> f64 {
        self.a.dot_product(&self.h)
    }

    pub fn vc(&self) -> f64 {
        self.a.dot_product(&self.v)
    }

    pub fn hs(&self) -> f64 {
        let cp = self.a.cross_product(&self.h);
        cp.len()
    }

    // Alias to hs() for focal length

    pub fn vs(&self) -> f64 {
        let cp = self.a.cross_product(&self.v);
        cp.len()
    }

    pub fn zeta(&self, p: &Vector) -> f64 {
        p.subtract(&self.c).dot_product(&self.o)
    }

    pub fn _lambda(&self, p: &Vector, z: f64) -> Vector {
        let o = self.o.scale(z);
        p.subtract(&self.c).subtract(&o)
    }

    pub fn lambda(&self, p: &Vector) -> Vector {
        let z = self.zeta(p);
        self._lambda(&p, z)
    }

    pub fn tau(&self, p: &Vector) -> f64 {
        let z = self.zeta(p);
        let l = self._lambda(p, z);

        l.dot_product(&l) / z.powi(2)
    }

    pub fn mu(&self, p: &Vector) -> f64 {
        let t = self.tau(p);
        self.r.x + self.r.y * t + self.r.z * t.powi(2)
    }

    pub fn corrected_point(&self, p: &Vector) -> Vector {
        let mut l = self.lambda(p);
        let m = self.mu(p);
        l = l.scale(m);
        p.add(&l)
    }

    pub fn rotation_matrix(&self, _w: f64, _o: f64, _k: f64) -> Matrix {
        let w = _w.to_radians();
        let o = _o.to_radians();
        let k = _k.to_radians();

        Matrix::new_with_values(
            o.cos() * k.cos(),
            w.sin() * o.sin() * k.sin() + w.cos() * k.sin(),
            -(w.cos() * o.sin() * k.cos() + w.sin() * k.sin()),
            0.0,
            -(o.cos() * k.sin()),
            -(w.sin() * o.sin() * k.sin() + w.cos() * k.cos()),
            w.cos() * o.sin() * k.sin() + w.sin() * k.cos(),
            0.0,
            o.sin(),
            -(w.sin() * o.cos()),
            w.cos() * o.cos(),
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        )
    }

    pub fn project_object_to_image_point(&self, p: &Vector) -> ImageCoordinate {
        ImageCoordinate {
            sample: self.i(p),
            line: self.j(p),
        }
    }

    // i -> column (origin at upper left)
    pub fn i(&self, p: &Vector) -> f64 {
        let pmc = p.subtract(&self.c);
        let a = pmc.dot_product(&self.h);
        let b = pmc.dot_product(&self.a);
        a / b
    }

    // j -> row (origin at upper left)
    pub fn j(&self, p: &Vector) -> f64 {
        let pmc = p.subtract(&self.c);
        let a = pmc.dot_product(&self.v);
        let b = pmc.dot_product(&self.a);
        a / b
    }
}

impl CameraModelTrait for Cahvor {
    fn model_type(&self) -> ModelType {
        ModelType::CAHVOR
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
        Vector::default()
    }

    fn box_clone(&self) -> Box<dyn CameraModelTrait + 'static> {
        Box::new((*self).clone())
    }

    fn f(&self) -> f64 {
        self.hs()
    }

    // Adapted from https://github.com/NASA-AMMOS/VICAR/blob/master/vos/java/jpl/mipl/mars/pig/PigCoreCAHVOR.java
    fn ls_to_look_vector(&self, coordinate: &ImageCoordinate) -> error::Result<LookVector> {
        let line = coordinate.line;
        let sample = coordinate.sample;

        let origin = self.c;

        let f = self.v.subtract(&self.a.scale(line));
        let g = self.h.subtract(&self.a.scale(sample));
        let mut rr = f.cross_product(&g).normalized();

        let t = self.v.cross_product(&self.h);
        if t.dot_product(&self.a) < 0.0 {
            rr = rr.inversed();
        }

        let omega = rr.dot_product(&self.o);
        let omega_2 = omega * omega;
        let wo = self.o.scale(omega);
        let lambda = rr.subtract(&wo);
        let tau = lambda.dot_product(&lambda) / omega_2;
        let k1 = 1.0 + self.r.x;
        let k3 = self.r.y * tau;
        let k5 = self.r.z * tau * tau;
        let mu = self.r.x + k3 + k5;
        let mut u = 1.0 - mu;

        for i in 0..(MAXITER + 1) {
            if i >= MAXITER {
                return Err("cahvor 2d to 3d: Too many iterations");
            }

            let u_2 = u * u;
            let poly = ((k5 * u_2 + k3) * u_2 + k1) * u - 1.0;
            let deriv = (5.0 * k5 * u_2 + 3.0 * k3) * u_2 + k1;
            if deriv <= EPSILON {
                return Err("Cahvor 2d to 3d: Distortion is too negative");
            } else {
                let du = poly / deriv;
                u -= du;
                if du.abs() < CONV {
                    break;
                }
            }
        }

        Ok(LookVector {
            origin,
            look_direction: rr.subtract(&lambda.scale(1.0 - u)).normalized(),
        })
    }

    // Adapted from https://github.com/NASA-AMMOS/VICAR/blob/master/vos/java/jpl/mipl/mars/pig/PigCoreCAHVOR.java
    fn xyz_to_ls(&self, xyz: &Vector, infinity: bool) -> ImageCoordinate {
        if infinity {
            let omega = xyz.dot_product(&self.o);
            let omega_2 = omega * omega;
            let wo = self.o.scale(omega);
            let lambda = xyz.subtract(&wo);
            let tau = lambda.dot_product(&lambda) / omega_2;
            let mu = self.r.x + (self.r.y * tau) + (self.r.z * tau * tau);
            let pp_c = xyz.add(&lambda.scale(mu));

            let alpha = pp_c.dot_product(&self.a);
            let beta = pp_c.dot_product(&self.h);
            let gamma = pp_c.dot_product(&self.v);

            ImageCoordinate {
                sample: beta / alpha,
                line: gamma / alpha,
            }
        } else {
            let p_c = xyz.subtract(&self.c);
            let omega = p_c.dot_product(&self.o);
            let omega_2 = omega * omega;
            let wo = self.o.scale(omega);
            let lambda = p_c.subtract(&wo);
            let tau = lambda.dot_product(&lambda) / omega_2;
            let mu = self.r.x + (self.r.y * tau) + (self.r.z * tau * tau);
            let pp = xyz.add(&lambda.scale(mu));

            let pp_c = pp.subtract(&self.c);
            let alpha = pp_c.dot_product(&self.a);
            let beta = pp_c.dot_product(&self.h);
            let gamma = pp_c.dot_product(&self.v);

            ImageCoordinate {
                sample: beta / alpha,
                line: gamma / alpha,
            }
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
            "{};{};{};{};{};{}",
            vec_to_str(&self.c.to_vec()),
            vec_to_str(&self.a.to_vec()),
            vec_to_str(&self.h.to_vec()),
            vec_to_str(&self.v.to_vec()),
            vec_to_str(&self.o.to_vec()),
            vec_to_str(&self.r.to_vec())
        )
    }
}

//  Adapted from https://github.com/digimatronics/ComputerVision/blob/master/src/vw/Camera/CAHVORModel.cc
pub fn linearize(
    camera_model: &Cahvor,
    cahvor_width: usize,
    cahvor_height: usize,
    cahv_width: usize,
    cahv_height: usize,
) -> Cahv {
    let minfov = true;

    let mut output_camera = Cahv::default();
    output_camera.c = camera_model.c;

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

    for local in vpts.iter() {
        if let Ok(lv) = camera_model.ls_to_look_vector(&ImageCoordinate {
            line: local.y,
            sample: local.x,
        }) {
            output_camera.a = output_camera.a.add(&lv.look_direction);
        }
    }

    for local in hpts.iter() {
        if let Ok(lv) = camera_model.ls_to_look_vector(&ImageCoordinate {
            line: local.y,
            sample: local.x,
        }) {
            output_camera.a = output_camera.a.add(&lv.look_direction);
        }
    }

    output_camera.a = output_camera.a.normalized();

    let mut dn = camera_model.a.cross_product(&camera_model.h).normalized();
    let mut rt = dn.cross_product(&output_camera.a);
    dn = output_camera.a.cross_product(&rt).normalized();
    rt = rt.normalized();

    let mut hmin = 1.0;
    let mut hmax = -1.0;
    for loop_ in hpts.iter() {
        if let Ok(lv) = camera_model.ls_to_look_vector(&ImageCoordinate {
            line: loop_.y,
            sample: loop_.x,
        }) {
            let u3 = lv.look_direction;
            let sn = output_camera
                .a
                .cross_product(&u3.subtract(&dn.scale(dn.dot_product(&u3))).normalized())
                .len();
            hmin = min!(hmin, sn);
            hmax = max!(hmax, sn);
        }
    }

    let mut vmin = 1.0;
    let mut vmax = -1.0;
    for loop_ in vpts.iter() {
        if let Ok(lv) = camera_model.ls_to_look_vector(&ImageCoordinate {
            line: loop_.y,
            sample: loop_.x,
        }) {
            let u3 = lv.look_direction;
            let sn = output_camera
                .a
                .cross_product(&u3.subtract(&rt.scale(rt.dot_product(&u3))).normalized())
                .len();
            vmin = min!(vmin, sn);
            vmax = max!(vmax, sn);
        }
    }

    println!("{}, {} -- {}, {}", hmin, hmax, vmin, vmax);
    let image_center = Vector::new(cahv_width as f64, cahv_height as f64, 0.0)
        .subtract(&Vector::new(1.0, 1.0, 0.0))
        .scale(0.5);
    println!("image_center -> {:?}", image_center);
    let image_center_2 = image_center.multiply(&image_center);
    println!("image_center_2 -> {:?}", image_center_2);

    let scale_factors = if minfov {
        image_center_2
            .divide(&Vector::new(hmin * hmin, vmin * vmin, 0.0))
            .subtract(&image_center_2)
            .sqrt()
    } else {
        image_center_2
            .divide(&Vector::new(hmax * hmax, vmax * vmax, 0.0))
            .subtract(&image_center_2)
            .sqrt()
    };

    println!("scale_factors -> {:?}", scale_factors);
    output_camera.h = rt
        .scale(scale_factors.x)
        .add(&output_camera.a.scale(image_center.x));
    output_camera.v = dn
        .scale(scale_factors.y)
        .add(&output_camera.a.scale(image_center.y));

    output_camera
}
