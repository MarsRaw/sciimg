use crate::vector::Vector;

pub static EPSILON: f64 = 1.0e-15;
pub static CONV: f64 = 1.0e-6;
pub static MAXITER: u8 = 20;

#[derive(PartialEq, Eq)]
pub enum ModelType {
    CAHV,
    CAHVOR,
    CAHVORE,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ImageCoordinate {
    pub line: f64,
    pub sample: f64,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LookVector {
    pub origin: Vector,
    pub look_direction: Vector,
}

impl LookVector {
    pub fn intersect_to_plane(&self, ground: &Vector) -> Option<Vector> {
        let normal = Vector::new(0.0, 0.0, -1.0);

        let dot = self.look_direction.dot_product(&normal);
        if dot == 0.0 {
            return Some(self.look_direction);
        }

        let ratio = ground.subtract(&self.origin).dot_product(&normal) / dot;

        let intersect_point = self.origin.add(&self.look_direction.scale(ratio));

        if ratio < 0.0 {
            None
        } else {
            Some(intersect_point)
        }
    }

    pub fn intersect_to_sphere(&self, radius: f64) -> Vector {
        self.look_direction
            .normalized()
            .scale(radius)
            .add(&self.origin)
    }
}
