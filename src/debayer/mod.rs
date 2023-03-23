#[allow(unused_macros)]
#[allow(dead_code)]
mod amaze;

#[allow(dead_code)]
mod malvar;

use std::str::FromStr;

use crate::error::Result;
use crate::imagebuffer::ImageBuffer;
use crate::rgbimage::RgbImage;
use serde::{Deserialize, Serialize};

// Filter patterns 'borrowed' from dcraw:
// https://github.com/Beep6581/RawTherapee/blob/dev/rtengine/dcraw.cc

/// BGGR Pattern
///
///   0 1 2 3 4 5
/// 0 B G B G B G
/// 1 G R G R G R
/// 2 B G B G B G
/// 3 G R G R G R
static FILTER_PATTERN_BGGR: u32 = 0x16161616;

/// GRBG Pattern
///
///  0 1 2 3 4 5
/// 0 G R G R G R
/// 1 B G B G B G
/// 2 G R G R G R
/// 3 B G B G B G
static FILTER_PATTERN_GRBG: u32 = 0x61616161;

/// GBRG Pattern
///
///   0 1 2 3 4 5
/// 0 G B G B G B
/// 1 R G R G R G
/// 2 G B G B G B
/// 3 R G R G R G
static FILTER_PATTERN_GBRG: u32 = 0x49494949;

/// RGGB Pattern
///
///   0 1 2 3 4 5
/// 0 R G R G R G
/// 1 G B G B G B
/// 2 R G R G R G
/// 3 G B G B G B
static FILTER_PATTERN_RGGB: u32 = 0x94949494;

/// Enums for each of the four major bayer grids
#[derive(Copy, Clone)]
pub enum FilterPattern {
    BGGR,
    GRBG,
    GBRG,
    RGGB,
}

impl FilterPattern {
    /// Translate enum to 32-bit filter pattern
    pub fn pattern(self) -> u32 {
        match self {
            Self::BGGR => FILTER_PATTERN_BGGR,
            Self::GBRG => FILTER_PATTERN_GBRG,
            Self::GRBG => FILTER_PATTERN_GRBG,
            Self::RGGB => FILTER_PATTERN_RGGB,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebayerMethod {
    Malvar,
    AMaZE, // Not ready
}

impl FromStr for DebayerMethod {
    fn from_str(s: &str) -> std::result::Result<DebayerMethod, std::string::String> {
        Ok(match s.to_uppercase().as_str() {
            "AMAZE" => DebayerMethod::AMaZE,
            _ => DebayerMethod::Malvar, // "MALVAR"
        })
    }

    type Err = String;
}

/// Debayer a single-channel image with specified algorithm. Defaults to RGGB
/// filter pattern.
pub fn debayer(buffer: &ImageBuffer, method: DebayerMethod) -> Result<RgbImage> {
    debayer_with_pattern(buffer, method, FilterPattern::RGGB)
}

/// Debayer a single-channel image with specified algorithm and filter pattern.
pub fn debayer_with_pattern(
    buffer: &ImageBuffer,
    method: DebayerMethod,
    filter_pattern: FilterPattern,
) -> Result<RgbImage> {
    match method {
        DebayerMethod::Malvar => malvar::debayer_with_pattern(buffer, filter_pattern),
        DebayerMethod::AMaZE => amaze::debayer_with_pattern(buffer, filter_pattern),
    }
}
