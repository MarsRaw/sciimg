use crate::color::{Color, ColorConverter};
use crate::matrix::Matrix;

struct Xyz2sRgbConverter {
    m: Matrix,
}

impl Xyz2sRgbConverter {
    pub fn new() -> Self {
        Xyz2RgbConverter {

            // CIE XYZ to sRGB D65 linear transformation matrix
            #[rustfmt::skip]
            m: Matrix::new_from_vec(vec![
                3.2406255,  -1.537208,  -0.4986286, 0.0, 
                -0.9689307,  1.8757561,  0.0415175, 0.0,
                0.0557101,  -0.2040211,  1.0569959, 0.0, 
                0.0,         0.0,        0.0,       1.0,
            ]),
        }
    }
}
