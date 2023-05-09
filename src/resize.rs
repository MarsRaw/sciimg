use crate::{enums::ImageMode, imagebuffer};
use anyhow::Result;
use image::imageops::resize;
use image::imageops::FilterType;

pub fn resize_to_8(
    buffer: &imagebuffer::ImageBuffer,
    to_width: usize,
    to_height: usize,
) -> Result<imagebuffer::ImageBuffer> {
    let image = buffer.buffer_to_image_8bit();
    let result = resize(
        &image,
        to_width as u32,
        to_height as u32,
        FilterType::Lanczos3,
    );
    imagebuffer::ImageBuffer::from_image_u8(&result)
}

pub fn resize_to_16(
    buffer: &imagebuffer::ImageBuffer,
    to_width: usize,
    to_height: usize,
) -> Result<imagebuffer::ImageBuffer> {
    let image = buffer.buffer_to_image_16bit();
    let result = resize(
        &image,
        to_width as u32,
        to_height as u32,
        FilterType::Lanczos3,
    );
    imagebuffer::ImageBuffer::from_image_u16(&result)
}

pub fn resize_to(
    buffer: &imagebuffer::ImageBuffer,
    to_width: usize,
    to_height: usize,
) -> Result<imagebuffer::ImageBuffer> {
    match buffer.mode {
        ImageMode::U8BIT => resize_to_8(buffer, to_width, to_height),
        ImageMode::U12BIT => resize_to_16(buffer, to_width, to_height),
        ImageMode::U16BIT => resize_to_16(buffer, to_width, to_height),
    }
}
