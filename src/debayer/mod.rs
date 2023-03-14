#![allow(warnings)]
#![allow(clippy)]
#![allow(unknown_lints)]
mod amaze;
mod malvar;

use crate::error;
use crate::imagebuffer::ImageBuffer;
use crate::rgbimage::RgbImage;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DebayerMethod {
    Malvar,
    AMaZE, // Not ready
}

pub fn debayer(buffer: &ImageBuffer, method: DebayerMethod) -> error::Result<RgbImage> {
    match method {
        DebayerMethod::Malvar => malvar::debayer(&buffer),
        DebayerMethod::AMaZE => amaze::debayer(&buffer),
    }
}
