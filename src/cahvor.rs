
use crate::{
    vector::Vector,
    matrix::Matrix
};

use serde::{
    Deserialize, 
    Serialize
};

static EPSILON:f64 = 1.0e-15;
static CONV:f64 = 1.0e-6;
static MAXITER:u8 = 20;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Mode {
    Cahvor,
    Cahv
}
impl Default for Mode {
    fn default() -> Self {
        Mode::Cahv
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point {
    pub i: f64,
    pub j: f64
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ImageCoordinate {
    pub line: f64,
    pub sample: f64
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LookVector {
    pub origin:Vector,
    pub look_direction:Vector
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cahvor {
    
    
    #[serde(skip_deserializing)]
    #[serde(default)]
    pub mode: Mode,

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
    pub fn f(&self) -> f64 {
        self.hs()
    }

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

    pub fn project_object_to_image_point(&self, p:&Vector) -> Point {
        Point{
            i:self.i(&p),
            j:self.j(&p)
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

    // Adapted from https://github.com/NASA-AMMOS/VICAR/blob/master/vos/java/jpl/mipl/mars/pig/PigCoreCAHVOR.java
    pub fn ls_to_look_vector(&self, coordinate:&ImageCoordinate) -> LookVector {
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
                panic!("cahvor 2d to 3d: Too many iterations");
            }

            let u_2 = u * u;
            let poly = ((k5 * u_2 + k3) * u_2 + k1) * u - 1.0;
            let deriv = (5.0 * k5 * u_2 + 3.0 * k3) * u_2 + k1;
            if deriv < EPSILON {
                panic!("Cahvor 2d to 3d: Distortion is too negative");
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

        LookVector{
            origin:origin,
            look_direction:look_direction
        }
    }

    // Adapted from https://github.com/NASA-AMMOS/VICAR/blob/master/vos/java/jpl/mipl/mars/pig/PigCoreCAHVOR.java
    pub fn xyz_to_ls(&self, xyz:&Vector, infinity:bool) -> ImageCoordinate {
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

}