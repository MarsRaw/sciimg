
use crate::{
    error,
    vector::Vector,
    matrix::Matrix,
    camera::model::*
};

use serde::{
    Deserialize, 
    Serialize
};


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
    pub r: Vector
}


impl Cahvor {
    pub fn default() -> Self {
        Cahvor{
            c:Vector::default(),
            a:Vector::default(),
            h:Vector::default(),
            v:Vector::default(),
            o:Vector::default(),
            r:Vector::default()
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

    pub fn zeta(&self, p:&Vector) -> f64 {
        p.subtract(&self.c).dot_product(&self.o)
    }

    pub fn _lambda(&self, p:&Vector, z:f64) -> Vector {
        let o = self.o.scale(z);
        p.subtract(&self.c).subtract(&o)
    }


    pub fn lambda(&self, p:&Vector) -> Vector {
        let z = self.zeta(&p);
        self._lambda(&p, z)
    }

    pub fn tau(&self, p:&Vector) -> f64 {
        let z = self.zeta(&p);
        let l = self._lambda(&p, z);

        l.dot_product(&l) / z.powi(2)
    }

    pub fn mu(&self, p:&Vector) -> f64 {
        let t = self.tau(&p);
        self.r.x + self.r.y * t + self.r.z * t.powi(2)
    }

    pub fn corrected_point(&self, p:&Vector) -> Vector {
        let mut l = self.lambda(&p);
        let m = self.mu(&p);
        l = l.scale(m);
        p.add(&l)
    }

    pub fn rotation_matrix(&self, _w:f64, _o:f64, _k:f64) -> Matrix {
        let w = _w.to_radians();
        let o = _o.to_radians();
        let k = _k.to_radians();

        Matrix::new_with_values(
                o.cos() * k.cos(), w.sin() * o.sin() * k.sin() + w.cos() * k.sin(), -(w.cos() * o.sin() * k.cos() + w.sin() * k.sin()),
                -(o.cos() * k.sin()), -(w.sin() * o.sin() * k.sin() + w.cos() * k.cos()), w.cos() * o.sin() * k.sin() + w.sin() * k.cos(),
                o.sin(), -(w.sin() * o.cos()), w.cos() * o.cos()
        )
    }

    pub fn project_object_to_image_point(&self, p:&Vector) -> ImageCoordinate {
        ImageCoordinate{
            sample:self.i(&p),
            line:self.j(&p)
        }
    } 

    // i -> column (origin at upper left)
    pub fn i(&self, p:&Vector) -> f64 {
        let pmc = p.subtract(&self.c);
        let a = pmc.dot_product(&self.h);
        let b = pmc.dot_product(&self.a);
        a / b
    }

    // j -> row (origin at upper left)
    pub fn j(&self, p:&Vector) -> f64 {
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
        self.c.clone()
    }

    fn a(&self) -> Vector {
        self.c.clone()
    }

    fn h(&self) -> Vector {
        self.c.clone()
    }

    fn v(&self) -> Vector {
        self.c.clone()
    }

    fn o(&self) -> Vector {
        self.c.clone()
    }

    fn r(&self) -> Vector {
        self.c.clone()
    }

    fn box_clone(&self) -> Box<dyn CameraModelTrait + 'static> {
        Box::new((*self).clone())
    }

    fn f(&self) -> f64 {
        self.hs()
    }

    // Adapted from https://github.com/NASA-AMMOS/VICAR/blob/master/vos/java/jpl/mipl/mars/pig/PigCoreCAHVOR.java
    fn ls_to_look_vector(&self, coordinate:&ImageCoordinate) -> error::Result<LookVector> {
        let line = coordinate.line;
        let sample = coordinate.sample;

        let origin = self.c.clone();

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
        let mut mu = self.r.x + k3 + k5;
        let mut u = 1.0 - mu;

        for i in 0..(MAXITER+1) {
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

        mu = 1.0 - u;
        let pp = lambda.scale(mu);
        let look_direction = rr.subtract(&pp).normalized();

        Ok(LookVector{
            origin:origin,
            look_direction:look_direction
        })
    }

    // Adapted from https://github.com/NASA-AMMOS/VICAR/blob/master/vos/java/jpl/mipl/mars/pig/PigCoreCAHVOR.java
    fn xyz_to_ls(&self, xyz:&Vector, infinity:bool) -> ImageCoordinate {
        if infinity == true {
            let omega = xyz.dot_product(&self.o);
            let omega_2 = omega * omega;
            let wo = self.o.scale(omega);
            let lambda = xyz.subtract(&wo);
            let tau = lambda.dot_product(&lambda) / omega_2;
            let mu = self.r.x + (self.r.y * tau) + (self.r.z * tau * tau);
            let pp_c = lambda.scale(mu);
            let pp_c2 = pp_c.add(&pp_c);

            let alpha = pp_c2.dot_product(&self.a);
            let beta = pp_c2.dot_product(&self.h);
            let gamma = pp_c2.dot_product(&self.v);

            ImageCoordinate{
                line: beta / alpha,
                sample: gamma / alpha
            }
        } else {
            let p_c = xyz.subtract(&self.c);
            let omega = p_c.dot_product(&self.o);
            let omega_2 = omega * omega;
            let wo = self.o.scale(omega);
            let lambda = p_c.subtract(&wo);
            let tau = lambda.dot_product(&lambda) / omega_2;
            let mu = self.r.x + (self.r.y * tau) + (self.r.z * tau * tau);
            let pp = lambda.scale(mu);

            let pp_c = pp.subtract(&self.c);
            let alpha = pp_c.dot_product(&self.a);
            let beta = pp_c.dot_product(&self.h);
            let gamma = pp_c.dot_product(&self.v);

            ImageCoordinate{
                line: beta / alpha,
                sample: gamma / alpha
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

}


