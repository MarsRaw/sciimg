use crate::{
    debayer::FilterPattern, error, imagebuffer::ImageBuffer, not_implemented, rgbimage::RgbImage,
};

/// Debayers a single channel image buffer using the default (RGGB) filter pattern
///
pub fn debayer(buffer: &ImageBuffer) -> error::Result<RgbImage> {
    debayer_with_pattern(buffer, FilterPattern::RGGB)
}

/// Debayers a single channel image buffer
pub fn debayer_with_pattern(
    buffer: &ImageBuffer,
    filter_pattern: FilterPattern,
) -> error::Result<RgbImage> {
    not_implemented!()
}
