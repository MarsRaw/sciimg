use crate::{
    debayer::FilterPattern, error, image::Image, imagebuffer::ImageBuffer, not_implemented,
};

/// Debayers a single channel image buffer using the default (RGGB) filter pattern
///
pub fn debayer(buffer: &ImageBuffer) -> error::Result<Image> {
    debayer_with_pattern(buffer, FilterPattern::RGGB)
}

/// Debayers a single channel image buffer
pub fn debayer_with_pattern(
    buffer: &ImageBuffer,
    filter_pattern: FilterPattern,
) -> error::Result<Image> {
    not_implemented!()
}
