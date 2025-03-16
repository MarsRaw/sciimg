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

pub mod gpu_context;
pub mod image;
pub mod image_proc {
    //! image processing
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

    use super::{gpu_context::GpuContext, image::GpuImage};

    impl GpuContext {
        pub fn gaussianblur<C>(&self, img: &GpuImage<C>) -> GpuImage<C>
        where
            C: ShaderSize + WriteInto + ReadFrom,
        {
            todo!()
        }
    }
}
