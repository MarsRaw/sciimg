use crate::{error, imagebuffer};

use image::imageops::resize;
use image::imageops::FilterType;

pub fn resize_to(
    buffer: &imagebuffer::ImageBuffer,
    to_width:usize,
    to_height:usize
) -> error::Result<imagebuffer::ImageBuffer> {
    let image = buffer.buffer_to_image_16bit();
    let result = resize(&image, to_width as u32, to_height as u32, FilterType::Lanczos3);
    imagebuffer::ImageBuffer::from_image_u16(&result)
}