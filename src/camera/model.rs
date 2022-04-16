
use crate::{
    error,
    vector::Vector
};

pub static EPSILON:f64 = 1.0e-15;
pub static CONV:f64 = 1.0e-6;
pub static MAXITER:u8 = 20;

#[derive(PartialEq, Eq)]
pub enum ModelType {
    CAHV,
    CAHVOR,
    CAHVORE
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

pub trait CameraModelTrait {
    fn model_type(&self) -> ModelType;
    fn f(&self) -> f64;
    fn pixel_angle_horiz(&self) -> f64;
    fn pixel_angle_vert(&self) -> f64;
    fn ls_to_look_vector(&self, coordinate:&ImageCoordinate) -> error::Result<LookVector>;
    fn xyz_to_ls(&self, xyz:&Vector, infinity:bool) -> ImageCoordinate;
    fn box_clone(&self) -> Box<dyn CameraModelTrait + 'static>;
    fn c(&self) -> Vector;
    fn a(&self) -> Vector;
    fn h(&self) -> Vector;
    fn v(&self) -> Vector;
    fn o(&self) -> Vector;
    fn r(&self) -> Vector;
    fn e(&self) -> Vector;
}


#[derive(Clone)]
pub struct CameraModel {
    model : Option<Box<dyn CameraModelTrait + 'static>>
}


impl CameraModel {
    
    pub fn new(model:Box<dyn CameraModelTrait + 'static>) -> CameraModel {
        CameraModel{
            model:Some(model)
        }
    }

    pub fn default() -> CameraModel {
        CameraModel{
            model:None
        }
    }

    pub fn is_valid(&self) -> bool {
        match self.model {
            Some(_) => true,
            None => false
        }
    }

    pub fn model_type(&self) -> ModelType {
        match &self.model {
            Some(m) => m.model_type(),
            None => panic!("Camera model is not valid")
        }
    }

    pub fn c(&self) -> Vector {
        match &self.model {
            Some(m) => m.c(),
            None => Vector::default()
        }
    }

    pub fn a(&self) -> Vector {
        match &self.model {
            Some(m) => m.a(),
            None => Vector::default()
        }
    }

    pub fn h(&self) -> Vector {
        match &self.model {
            Some(m) => m.h(),
            None => Vector::default()
        }
    }

    pub fn v(&self) -> Vector {
        match &self.model {
            Some(m) => m.v(),
            None => Vector::default()
        }
    }

    pub fn o(&self) -> Vector {
        match &self.model {
            Some(m) => m.o(),
            None => Vector::default()
        }
    }

    pub fn r(&self) -> Vector {
        match &self.model {
            Some(m) => m.r(),
            None => Vector::default()
        }
    }

    pub fn f(&self) -> f64 {
        match &self.model {
            Some(m) => m.f(),
            None => panic!("Camera model is not valid")
        }
    }

    pub fn pixel_angle_horiz(&self) -> f64 {
        match &self.model {
            Some(m) => m.pixel_angle_horiz(),
            None => panic!("Camera model is not valid")
        }
    }

    pub fn pixel_angle_vert(&self) -> f64 {
        match &self.model {
            Some(m) => m.pixel_angle_vert(),
            None => panic!("Camera model is not valid")
        }
    }

    pub fn ls_to_look_vector(&self, coordinate:&ImageCoordinate) -> error::Result<LookVector> {
        match &self.model {
            Some(m) => m.ls_to_look_vector(&coordinate),
            None => panic!("Camera model is not valid")
        }
    }

    pub fn xyz_to_ls(&self, xyz:&Vector, infinity:bool) -> ImageCoordinate {
        match &self.model {
            Some(m) => m.xyz_to_ls(&xyz, infinity),
            None => panic!("Camera model is not valid")
        }
    }

}

impl Clone for Box<dyn CameraModelTrait + 'static> {
    fn clone(&self) -> Box<dyn CameraModelTrait + 'static> {
        self.box_clone()
    }   
}