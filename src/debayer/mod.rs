#![allow(warnings)]
#![allow(clippy)]
#![allow(unknown_lints)]
mod amaze;
mod malvar;

use crate::error::Result;
use crate::imagebuffer::ImageBuffer;
use crate::rgbimage::RgbImage;
use serde::{Deserialize, Serialize};
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebayerMethod {
    Malvar,
    AMaZE, // Not ready
}

impl DebayerMethod {
    pub fn from_str(s: &String) -> Result<DebayerMethod> {
        Ok(match s.to_uppercase().as_str() {
            "AMAZE" => DebayerMethod::AMaZE,
            "MALVAR" | _ => DebayerMethod::Malvar,
        })
    }
}

pub fn debayer(buffer: &ImageBuffer, method: DebayerMethod) -> Result<RgbImage> {
    match method {
        DebayerMethod::Malvar => malvar::debayer(&buffer),
        DebayerMethod::AMaZE => amaze::debayer(&buffer),
    }
}
