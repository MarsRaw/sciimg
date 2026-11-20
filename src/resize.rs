use crate::{enums::ImageMode, imagebuffer};
use anyhow::Result;
use image::imageops::FilterType;
use image::imageops::resize;

/// These directly translate to the image crates FilterType enum
#[derive(Debug, Copy, Clone)]
pub enum ResizeFilter {
    Nearest,
    Triangle,
    CatmullRom,
    Gaussian,
    Lanczos3,
}

impl ResizeFilter {
    pub fn to_filter_type(&self) -> FilterType {
        match self {
            Self::Nearest => FilterType::Nearest,
            Self::Triangle => FilterType::Triangle,
            Self::CatmullRom => FilterType::CatmullRom,
            Self::Gaussian => FilterType::Gaussian,
            Self::Lanczos3 => FilterType::Lanczos3,
        }
    }
}

pub fn resize_to_8(
    buffer: &imagebuffer::ImageBuffer,
    to_width: usize,
    to_height: usize,
    filter: ResizeFilter,
) -> Result<imagebuffer::ImageBuffer> {
    let image = buffer.buffer_to_image_8bit();
    let result = resize(
        &image,
        to_width as u32,
        to_height as u32,
        filter.to_filter_type(),
    );
    imagebuffer::ImageBuffer::from_image_u8(&result)
}

pub fn resize_to_16(
    buffer: &imagebuffer::ImageBuffer,
    to_width: usize,
    to_height: usize,
    filter: ResizeFilter,
) -> Result<imagebuffer::ImageBuffer> {
    let image = buffer.buffer_to_image_16bit();
    let result = resize(
        &image,
        to_width as u32,
        to_height as u32,
        filter.to_filter_type(),
    );
    imagebuffer::ImageBuffer::from_image_u16(&result)
}

pub fn resize_to(
    buffer: &imagebuffer::ImageBuffer,
    to_width: usize,
    to_height: usize,
    filter: ResizeFilter,
) -> Result<imagebuffer::ImageBuffer> {
    match buffer.mode {
        ImageMode::U8BIT => resize_to_8(buffer, to_width, to_height, filter),
        ImageMode::U12BIT => resize_to_16(buffer, to_width, to_height, filter),
        ImageMode::U16BIT => resize_to_16(buffer, to_width, to_height, filter),
    }
}
