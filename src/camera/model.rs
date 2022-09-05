
use crate::{
    error,
    vector::Vector,
    camera::cahv,
    camera::cahvor,
    camera::cahvore
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


impl LookVector {


    pub fn intersect_to_plane(&self, ground:&Vector) -> Option<Vector> {
        let normal = Vector::new(0.0, 0.0, -1.0);
        
        let dot = self.look_direction.dot_product(&normal);
        if dot == 0.0 {
            return Some(self.look_direction.clone());
        }
    
        let ratio = ground.subtract(&self.origin).dot_product(&normal) / dot;
    
        let intersect_point = self.origin.add(&self.look_direction.scale(ratio));
        
        if ratio < 0.0 {
            None
        } else {
            Some(intersect_point)
        }
    
    }

    pub fn intersect_to_sphere(&self, radius:f64) -> Vector {
        self.look_direction.normalized().scale(radius).add(&self.origin)
    }

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
    fn serialize(&self) -> String;
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

    pub fn e(&self) -> Vector {
        match &self.model {
            Some(m) => m.e(),
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

    pub fn linearize(&self, cahvor_width:usize, cahvor_height:usize, cahv_width:usize, cahv_height:usize) -> error::Result<cahv::Cahv> {
        match self.model_type() {
            ModelType::CAHV => {
                if let Some(m) = &self.model {
                    let c = cahv::Cahv{
                        c: m.c(),
                        a: m.a(),
                        h: m.h(),
                        v: m.v()
                    };
                    Ok(c)
                } else {
                    Err("Wut?")
                }
            },
            ModelType::CAHVOR => {

                if let Some(m) = &self.model {
                    let c = cahvor::Cahvor{
                        c: m.c(),
                        a: m.a(),
                        h: m.h(),
                        v: m.v(),
                        o: m.o(),
                        r: m.r()
                    };
                    Ok(cahvor::linearize(&c, cahvor_width, cahvor_height, cahv_width, cahv_height))
                } else {
                    Err("Wut?")
                }

                
            },
            ModelType::CAHVORE => {

                if let Some(m) = &self.model {
                    let c = cahvore::Cahvore{
                        c: m.c(),
                        a: m.a(),
                        h: m.h(),
                        v: m.v(),
                        o: m.o(),
                        r: m.r(),
                        e: m.e(),
                        pupil_type: cahvore::PupilType::General,
                        linearity: cahvore::LINEARITY_FISHEYE
                    };
                    Ok(cahvore::linearize(&c, cahvor_width, cahvor_height, cahv_width, cahv_height))
                } else {
                    Err("Wut?")
                }

                
            }
        }
    }

    pub fn serialize(&self) -> String {
        match &self.model {
            Some(m) => m.serialize(),
            None => panic!("Camera model is not valid")
        }
    }

}

impl Clone for Box<dyn CameraModelTrait + 'static> {
    fn clone(&self) -> Box<dyn CameraModelTrait + 'static> {
        self.box_clone()
    }   
}