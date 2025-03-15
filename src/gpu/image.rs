//! The GPU version of a `sciimg::Image`
#![allow(unused_imports, dead_code)]
use crate::{
    enums, image::Image, max, min, path, Dn, DnVec, Mask, MaskVec, MaskedDnVec, MinMax, VecMath,
};

use bytemuck::{Pod, Zeroable};

use encase::{
    internal::{ReadFrom, WriteInto},
    ArrayLength, ShaderSize, ShaderType, StorageBuffer,
};
use glam::{Vec3, Vec3A, Vec3Swizzles, Vec4, Vec4Swizzles};
use thiserror;
use wgpu::Features;
pub type RGB = Vec3;
pub type RGBA = Vec4;

// A simple image raster buffer.
#[derive(ShaderType)]
pub struct GpuImage<C: ShaderSize + WriteInto + ReadFrom> {
    pub width: u32,
    pub height: u32,

    /// WGPU requires this, it is the length of the below .data
    length: ArrayLength,

    /// The Vec<ImageBuffer> is inappropraite for GPUs.
    /// So we use a `Vec<Vec3>` or, a `Vec4` when there's an alpha channel.
    #[size(runtime)]
    pub data: Vec<C>,
}

impl<C: ShaderSize + WriteInto + ReadFrom> GpuImage<C> {
    pub fn new(width: u32, height: u32, data: Vec<C>) -> Self {
        Self {
            width,
            height,
            length: ArrayLength,
            data,
        }
    }
    pub fn as_wgsl_bytes(&self) -> encase::internal::Result<Vec<u8>> {
        let mut buffer = encase::UniformBuffer::new(Vec::new());
        buffer.write(self)?;
        Ok(buffer.into_inner())
    }
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl GpuImage<Vec3> {
    pub fn from_sciimg_rgb(img: &Image) -> Self {
        let size = img.width * img.height;
        let mut data = Vec::with_capacity(size);
        for i in 0..size {
            data.push(Vec3::new(
                img.get_band(0).buffer[i],
                img.get_band(1).buffer[i],
                img.get_band(2).buffer[i],
            ));
        }

        Self {
            width: img.width as u32,
            height: img.height as u32,
            length: ArrayLength,
            data,
        }
    }
}

impl GpuImage<Vec4> {
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
            width: img.width as u32,
            height: img.height as u32,
            length: ArrayLength,
            data,
        }
    }
}

pub trait Empty {
    fn empty() -> Self;
}

impl Empty for GpuImage<Vec3> {
    fn empty() -> Self {
        Self {
            width: 0,
            height: 0,
            length: ArrayLength,
            data: Vec::new(),
        }
    }
}
impl Empty for GpuImage<Vec4> {
    fn empty() -> Self {
        Self {
            width: 0,
            height: 0,
            length: ArrayLength,
            data: Vec::new(),
        }
    }
}
