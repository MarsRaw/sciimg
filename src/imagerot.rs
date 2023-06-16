use crate::{enums, imagebuffer};

use anyhow::Result;
use image::Rgba;
use imageproc::geometric_transformations::{rotate_about_center, Interpolation};

pub fn rotate_8(buffer: &imagebuffer::ImageBuffer, theta: f32) -> Result<imagebuffer::ImageBuffer> {
    let image = buffer.buffer_to_image_8bit();
    let default_pixel = Rgba([0, 0, 0, 0]);
    let rotated = rotate_about_center(&image, theta, Interpolation::Bicubic, default_pixel);

    imagebuffer::ImageBuffer::from_image_u8(&rotated)
}

pub fn rotate_16(
    buffer: &imagebuffer::ImageBuffer,
    theta: f32,
) -> Result<imagebuffer::ImageBuffer> {
    let image = buffer.buffer_to_image_16bit();
    let default_pixel = Rgba([0, 0, 0, 0]);
    let rotated = rotate_about_center(&image, theta, Interpolation::Bicubic, default_pixel);

    imagebuffer::ImageBuffer::from_image_u16(&rotated)
}

pub fn rotate(buffer: &imagebuffer::ImageBuffer, theta: f32) -> Result<imagebuffer::ImageBuffer> {
    match buffer.mode {
        enums::ImageMode::U8BIT => rotate_8(buffer, theta),
        enums::ImageMode::U12BIT => rotate_16(buffer, theta),
        enums::ImageMode::U16BIT => rotate_16(buffer, theta),
    }
}
