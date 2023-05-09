use crate::{debayer::FilterPattern, image::Image, imagebuffer::ImageBuffer, not_implemented};
use anyhow::Result;
/// Debayers a single channel image buffer using the default (RGGB) filter pattern
///
pub fn debayer(buffer: &ImageBuffer) -> Result<Image> {
    debayer_with_pattern(buffer, FilterPattern::RGGB)
}

/// Debayers a single channel image buffer
pub fn debayer_with_pattern(buffer: &ImageBuffer, filter_pattern: FilterPattern) -> Result<Image> {
    not_implemented!()
}
