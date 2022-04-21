
use crate::{
    error,
    vector::Vector,
    //matrix::Matrix,
    camera::model::*
};

use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PupilType {
    Perspective,
    Fisheye,
    General
}

pub static CHIP_LIMIT : f64 = 1e-8;
pub static NEWTON_ITERATION_MAX : usize = 100;

pub static LINEARITY_PERSPECTIVE:f64 = 1.0;
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

    pub pupil_type:PupilType,

    pub linearity:f64
}

impl Cahvore {
    pub fn default() -> Self {
        Cahvore { 
            c:Vector::default(),
            a:Vector::default(),
            h:Vector::default(),
            v:Vector::default(),
            o:Vector::default(),
            r:Vector::default(),
            e:Vector::default(),
            pupil_type:PupilType::General,
            linearity:LINEARITY_FISHEYE
        }
    }
}

impl CameraModelTrait for Cahvore {

    fn model_type(&self) -> ModelType {
        ModelType::CAHVORE
    }

    fn c(&self) -> Vector {
        self.c.clone()
    }

    fn a(&self) -> Vector {
        self.a.clone()
    }

    fn h(&self) -> Vector {
        self.h.clone()
    }

    fn v(&self) -> Vector {
        self.v.clone()
    }

    fn o(&self) -> Vector {
        self.o.clone()
    }

    fn r(&self) -> Vector {
        self.r.clone()
    }

    fn e(&self) -> Vector {
        self.e.clone()
    }

    fn box_clone(&self) -> Box<dyn CameraModelTrait + 'static> {
        Box::new((*self).clone())
    }

    fn f(&self) -> f64 {
        self.a.cross_product(&self.h).len()
    }

    // Adapted from https://github.com/NASA-AMMOS/VICAR/blob/master/vos/java/jpl/mipl/mars/pig/PigCoreCAHVORE.java
    fn ls_to_look_vector(&self, _coordinate:&ImageCoordinate) -> error::Result<LookVector> {
        panic!("Not yet implemented");
    }

    // Adapted from https://github.com/NASA-AMMOS/VICAR/blob/master/vos/java/jpl/mipl/mars/pig/PigCoreCAHVORE.java
    fn xyz_to_ls(&self, _xyz:&Vector, _infinity:bool) -> ImageCoordinate {
        panic!("Not yet implemented");
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


