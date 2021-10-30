
use crate::{
    vector::Vector,
    matrix::Matrix
};


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Mode {
    Cahvor,
    Cahv
}

pub struct Point {
    pub i: f64,
    pub j: f64
}

#[derive(Debug, Clone)]
pub struct Cahvor {
    
    pub mode: Mode,

    // Camera center vector C
    pub c: Vector,

    // Camera axis unit vector A
    pub a: Vector,

    // Horizontal information vector H
    pub h: Vector,

    // Vertical information vector V
    pub v: Vector,

    // Optical axis unit vector O
    pub o: Vector,
    
    // Radial lens distortion coefficients
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
        let mut o = self.o.clone();
        o.scale(z);

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
        l.scale(m);
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
        let i = self.i(&p);
        let j = self.j(&p);

        Point{
            i,
            j
        }
    } 

    // i -> column (origin at upper left)
    pub fn i(&self, p:&Vector) -> f64 {
        let a = p.subtract(&self.c).dot_product(&self.h);
        let b = p.subtract(&self.c).dot_product(&self.a);
        a / b
    }

    // j -> row (origin at upper left)
    pub fn j(&self, p:&Vector) -> f64 {
        let a = p.subtract(&self.c).dot_product(&self.v);
        let b = p.subtract(&self.c).dot_product(&self.a);
        a / b
    }
}