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
pub type RGB = Vec3;
pub type RGBA = Vec4;

#[derive(ShaderType)]
pub struct ImageUniform {
    pub width: u32,
    pub height: u32,
    // pub alpha: bool,
}
impl ImageUniform {
    pub fn from_img<C>(img: &GpuImage<C>) -> Self
    where
        C: ShaderSize + WriteInto + ReadFrom,
    {
        let (width, height) = (img.width, img.height);

        Self { width, height }
    }
}
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

    pub fn to_sciimg_rgb(&self) -> Image {
        let size = (self.width * self.height) as usize;

        // Make band buffers
        let mut red_buf = Vec::with_capacity(size);
        let mut green_buf = Vec::with_capacity(size);
        let mut blue_buf = Vec::with_capacity(size);

        // Split out the Vec3 channels into separate bands
        for px in &self.data {
            red_buf.push(px.x);
            green_buf.push(px.y);
            blue_buf.push(px.z);
        }

        // Wrap them in MaskedDnVec (assuming some float-based DnVec).
        // In practice, adapt if your DnVec can be I16, etc.
        let mask = MaskVec::new(size, false); // no masked-out pixels
        let dn_red = MaskedDnVec {
            vec: DnVec::F32(red_buf),
            mask: mask.clone(),
            null: 0.0,
        };
        let dn_green = MaskedDnVec {
            vec: DnVec::F32(green_buf),
            mask: mask.clone(),
            null: 0.0,
        };
        let dn_blue = MaskedDnVec {
            vec: DnVec::F32(blue_buf),
            mask: mask,
            null: 0.0,
        };

        let band_r = ImageBuffer {
            buffer: dn_red,
            width: self.width as usize,
            height: self.height as usize,
            empty: false,
            mode: enums::ImageMode::U8BIT, // or whatever fits your data
        };
        let band_g = ImageBuffer {
            buffer: dn_green,
            width: self.width as usize,
            height: self.height as usize,
            empty: false,
            mode: enums::ImageMode::U8BIT,
        };
        let band_b = ImageBuffer {
            buffer: dn_blue,
            width: self.width as usize,
            height: self.height as usize,
            empty: false,
            mode: enums::ImageMode::U8BIT,
        };

        Image {
            bands: vec![band_r, band_g, band_b],
            alpha: MaskVec::new(size, false),
            uses_alpha: false,
            width: self.width as usize,
            height: self.height as usize,
            mode: enums::ImageMode::U8BIT,
            empty: false,
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

pub mod dimensions {
    use glam::{Vec3, Vec4};

    use super::GpuImage;

    pub trait ImgDimensions {
        fn dimensions(&self) -> (u32, u32);
        fn width(&self) -> u32;
        fn height(&self) -> u32;
    }
}
