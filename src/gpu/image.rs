//! The GPU version of a `sciimg::Image`
#![allow(unused_imports, dead_code)]
use crate::{
    enums, image::Image, max, min, path, prelude::ImageBuffer, Dn, DnVec, Mask, MaskVec,
    MaskedDnVec, MinMax, VecMath,
};

use bytemuck::{Pod, Zeroable};
use dimensions::ImgDimensions;
use encase::{
    internal::{ReadFrom, WriteInto},
    ArrayLength, ShaderSize, ShaderType, StorageBuffer,
};
use glam::{Vec3, Vec3A, Vec3Swizzles, Vec4, Vec4Swizzles};
use thiserror;
use wgpu::Features;

#[derive(ShaderType)]
pub struct ImageUniform {
    pub width: u32,
    pub height: u32,
    // pub alpha: bool,
}
impl ImageUniform {
    pub fn from(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(ShaderType)]
pub struct GpuImage {
    length: ArrayLength,
    #[size(runtime)]
    pub data: Vec<Vec4>,
}

impl GpuImage {
    pub fn from_data(data: Vec<Vec4>) -> Self {
        Self {
            length: ArrayLength,
            data,
        }
    }
    pub fn as_wgsl_uniform_buffer_bytes(&self) -> encase::internal::Result<Vec<u8>> {
        let mut buffer = encase::UniformBuffer::new(Vec::new());
        buffer.write(self)?;
        Ok(buffer.into_inner())
    }
}

impl GpuImage {
    pub fn from_sciimg_rgb(img: &Image) -> Self {
        let size = img.width * img.height;
        let mut data = Vec::with_capacity(size);

        assert!(img.num_bands() == 3);
        assert!(img.buffers().iter().all(|b| b.buffer.len() == size));

        for i in 0..size {
            data.push(Vec4::new(
                img.get_band(0).buffer[i],
                img.get_band(1).buffer[i],
                img.get_band(2).buffer[i],
                1.0,
            ));
        }

        Self {
            length: ArrayLength,
            data,
        }
    }

    pub fn to_sciimg_rgb(&self, width: usize, height: usize) -> anyhow::Result<Image> {
        let size = self.data.len();

        // Split Vec3 channels
        let mut red_buf = Vec::with_capacity(size);
        let mut green_buf = Vec::with_capacity(size);
        let mut blue_buf = Vec::with_capacity(size);
        for px in &self.data {
            red_buf.push(px.x);
            green_buf.push(px.y);
            blue_buf.push(px.z);
            // Ignoring alpha for now.
        }

        let red_band = ImageBuffer::from_vec_as_mode(
            &red_buf,
            width,
            height,
            enums::ImageMode::U16BIT, // or U8BIT, etc
        )?;

        let green_band =
            ImageBuffer::from_vec_as_mode(&green_buf, width, height, enums::ImageMode::U16BIT)?;

        let blue_band =
            ImageBuffer::from_vec_as_mode(&blue_buf, width, height, enums::ImageMode::U16BIT)?;

        Image::new_from_buffers_rgb(&red_band, &green_band, &blue_band, enums::ImageMode::U16BIT)
    }

    pub fn from_sciimg_with_alpha(img: &Image) -> Self {
        let size = img.width * img.height;
        let mut data = Vec::with_capacity(size);
        for i in 0..img.width * img.height {
            data.push(Vec4::new(
                img.get_band(0).buffer[i],
                img.get_band(1).buffer[i],
                img.get_band(2).buffer[i],
                if img.is_using_alpha() {
                    if img.get_alpha_at(i % img.width, i / img.width) {
                        1.0
                    } else {
                        0.0
                    }
                } else {
                    1.0
                },
            ));
        }

        Self {
            length: ArrayLength,
            data,
        }
    }
}

pub trait Empty {
    fn empty() -> Self;
}

impl Empty for GpuImage {
    fn empty() -> Self {
        Self {
            length: ArrayLength,
            data: Vec::new(),
        }
    }
}

pub mod dimensions {
    use glam::{Vec3, Vec4};

    use super::GpuImage;

    pub trait ImgDimensions {
        fn dimensions(&self) -> (u32, u32);
        fn width(&self) -> u32;
        fn height(&self) -> u32;
    }
}
