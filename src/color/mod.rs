use crate::matrix::Matrix;
use crate::vector::Vector;
use anyhow::{anyhow, Result};

#[allow(non_camel_case_types)]
#[derive(Debug, Eq, PartialEq)]
pub enum ColorSpaceType {
    iRGB,
    sRGB,
    pRGB,
    XYZ,
    RAD,
    xyY,
}

pub struct Color {
    value: Vector,
    space: ColorSpaceType,
}

pub trait ColorConverter {
    fn convert(&self, in_color: &Color) -> Result<Color>;
}

// Note: These conversions would nominally have camera-specific matricies. Hard-coding for now, but consider
// using .parms files once available for MSL & M20

///////////////////////////////
/// XYZ to sRGB
///////////////////////////////

pub struct Xyz2sRgbConverter {
    m: Matrix,
}

impl Xyz2sRgbConverter {
    pub fn new() -> Self {
        Xyz2sRgbConverter {
            // CIE XYZ to sRGB D65 linear transformation matrix
            m: Matrix::new_from_vec(&vec![
                3.2406255, -1.537208, -0.4986286, 0.0, -0.9689307, 1.8757561, 0.0415175, 0.0,
                0.0557101, -0.2040211, 1.0569959, 0.0, 0.0, 0.0, 0.0, 1.0,
            ])
            .unwrap(),
        }
    }
}

impl Default for Xyz2sRgbConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorConverter for Xyz2sRgbConverter {
    fn convert(&self, in_color: &Color) -> Result<Color> {
        if in_color.space != ColorSpaceType::XYZ {
            Err(anyhow!(
                "Cannot convert to sRGB, invalid input colorspace: {:?}",
                in_color.space
            ))
        } else {
            Ok(Color {
                value: self.m.multiply_vector(&in_color.value),
                space: ColorSpaceType::sRGB,
            })
        }
    }
}

///////////////////////////////
/// RAD to iRGB
///////////////////////////////

pub struct Rad2iRgbConverter {
    v: Vector,
}

impl Rad2iRgbConverter {
    pub fn new() -> Self {
        //strcat(pre, "RAD_to_iRGB_vector = %lf %lf %lf");
        Rad2iRgbConverter {
            v: Vector {
                x: 8.613539e-07,
                y: 1.091394e-06,
                z: 1.664399e-06,
            },
        }
    }
}

impl Default for Rad2iRgbConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorConverter for Rad2iRgbConverter {
    fn convert(&self, in_color: &Color) -> Result<Color> {
        if in_color.space != ColorSpaceType::RAD {
            Err(anyhow!(
                "Cannot convert to iRGB, invalid input colorspace: {:?}",
                in_color.space
            ))
        } else if self.v.x == 0.0 || self.v.y == 0.0 || self.v.z == 0.0 {
            Err(anyhow!(
                "RAD->iRGB converter vector contains a zero. Cannot divide by zero."
            ))
        } else {
            Ok(Color {
                value: Vector::new(
                    in_color.value.x / self.v.x,
                    in_color.value.y / self.v.y,
                    in_color.value.z / self.v.z,
                ),
                space: ColorSpaceType::iRGB,
            })
        }
    }
}

///////////////////////////////
/// XYZ to xyY
///////////////////////////////

pub struct Xyz2xyYConverter {}

impl Xyz2xyYConverter {
    pub fn new() -> Self {
        Xyz2xyYConverter {}
    }
}

impl Default for Xyz2xyYConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorConverter for Xyz2xyYConverter {
    fn convert(&self, in_color: &Color) -> Result<Color> {
        if in_color.space != ColorSpaceType::XYZ {
            Err(anyhow!(
                "Cannot convert to xyY, invalid input colorspace: {:?}",
                in_color.space
            ))
        } else {
            let (x, y) = if in_color.value.x + in_color.value.y + in_color.value.z == 0.0 {
                // Note: possible floating point error
                (0.0, 0.0)
            } else {
                (
                    in_color.value.x / (in_color.value.x + in_color.value.y + in_color.value.z),
                    in_color.value.y / (in_color.value.x + in_color.value.y + in_color.value.z),
                )
            };

            Ok(Color {
                value: Vector::new(x, y, in_color.value.y),
                space: ColorSpaceType::xyY,
            })
        }
    }
}

///////////////////////////////
/// iRGB to XYZ
///////////////////////////////

pub struct IRgb2XyzConverter {
    m: Matrix,
}

impl IRgb2XyzConverter {
    pub fn new() -> Self {
        IRgb2XyzConverter {
            m: Matrix::new_from_vec(&vec![
                1.0875708,
                -1.4314745,
                3.2392806,
                0.0,
                0.17009690,
                0.93876829,
                0.37937771,
                0.0,
                -0.62922341,
                -4.3906116,
                15.291394,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
            ])
            .unwrap(),
        }
    }
}

impl Default for IRgb2XyzConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorConverter for IRgb2XyzConverter {
    fn convert(&self, in_color: &Color) -> Result<Color> {
        if in_color.space != ColorSpaceType::iRGB {
            Err(anyhow!(
                "Cannot convert to XYZ, invalid input colorspace: {:?}",
                in_color.space
            ))
        } else {
            Ok(Color {
                value: self.m.multiply_vector(&in_color.value),
                space: ColorSpaceType::XYZ,
            })
        }
    }
}

///////////////////////////////
/// XYZ to pRGB
///////////////////////////////

pub struct Xyz2pRgbConverter {
    m: Matrix,
}

impl Xyz2pRgbConverter {
    pub fn new() -> Self {
        Xyz2pRgbConverter {
            m: Matrix::identity(),
        }
    }
}

impl Default for Xyz2pRgbConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorConverter for Xyz2pRgbConverter {
    fn convert(&self, in_color: &Color) -> Result<Color> {
        if in_color.space != ColorSpaceType::XYZ {
            Err(anyhow!(
                "Cannot convert to pRGB, invalid input colorspace: {:?}",
                in_color.space
            ))
        } else {
            Ok(Color {
                value: self.m.multiply_vector(&in_color.value),
                space: ColorSpaceType::pRGB,
            })
        }
    }
}

///////////////////////////////
/// sRGB to pRGB
///////////////////////////////

pub struct SRgb2pRgbConverter {
    m: Matrix,
}

impl SRgb2pRgbConverter {
    pub fn new() -> Self {
        SRgb2pRgbConverter {
            m: Matrix::new_from_vec(&vec![
                0.7742, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.9156, 0.0, 0.0, 0.0, 0.0,
                1.0,
            ])
            .unwrap(),
        }
    }
}

impl Default for SRgb2pRgbConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorConverter for SRgb2pRgbConverter {
    fn convert(&self, in_color: &Color) -> Result<Color> {
        if in_color.space != ColorSpaceType::sRGB {
            Err(anyhow!(
                "Cannot convert to pRGB, invalid input colorspace: {:?}",
                in_color.space
            ))
        } else {
            Ok(Color {
                value: self.m.multiply_vector(&in_color.value),
                space: ColorSpaceType::pRGB,
            })
        }
    }
}
