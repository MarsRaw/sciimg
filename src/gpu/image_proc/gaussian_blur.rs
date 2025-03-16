//! image processing
#![allow(unused_imports, dead_code)]
use crate::{
    enums,
    gpu::{gpu_context::GpuContext, image::GpuImage},
    image::Image,
    max, min, path, Dn, DnVec, Mask, MaskVec, MaskedDnVec, MinMax, VecMath,
};

use bytemuck::{Pod, Zeroable};

use encase::{
    internal::{ReadFrom, WriteInto},
    ArrayLength, ShaderSize, ShaderType, StorageBuffer,
};
use glam::{Vec3, Vec3A, Vec3Swizzles, Vec4, Vec4Swizzles};
use thiserror;
use wgpu::Features;

#[derive(ShaderType)]
struct GaussianBlurUniform {
    pub radius: u32,
    pub sigma: f32,
    pub width: u32,
    pub height: u32,
}

impl GpuContext {
    pub fn gaussian_blur(&self, img: &GpuImage<Vec3>, radius: u32, sigma: f32) -> GpuImage<Vec3> {
        // Create a uniform struct
        let uniform_data = GaussianBlurUniform {
            radius,
            sigma,
            width: img.width,
            height: img.height,
        };

        let mut input_bytes = Vec::new();
        {
            let mut sbuf = encase::StorageBuffer::new(&mut input_bytes);
            sbuf.write(img).unwrap(); // write GpuImage into the buffer
        }
        let input_size = input_bytes.len() as wgpu::BufferAddress;
        let input_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GaussianBlur Input"),
            size: input_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&input_buffer, 0, &input_bytes);

        // Output buffer:
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GaussianBlur Output"),
            size: input_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Uniform buffer:
        let mut uniform_bytes = Vec::new();
        {
            let mut ubuf = encase::StorageBuffer::new(&mut uniform_bytes);
            ubuf.write(&uniform_data).unwrap();
        }
        let uniform_size = uniform_bytes.len() as wgpu::BufferAddress;
        let uniform_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GaussianBlur Uniform"),
            size: uniform_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&uniform_buffer, 0, &uniform_bytes);

        // 3) Create bind group layouts & bind groups.
        // Convention: uniform is binding=0, storage is binding=100
        let layout0 = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("GaussianBlur layout0"),
                entries: &[
                    // uniform at binding 0
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // input buffer at binding=100
                    wgpu::BindGroupLayoutEntry {
                        binding: 100,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let layout1 = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("GaussianBlur layout1"),
                entries: &[
                    // output buffer also at binding=100
                    wgpu::BindGroupLayoutEntry {
                        binding: 100,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let bind_group0 = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("GaussianBlur bind_group0"),
            layout: &layout0,
            entries: &[
                // uniform
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                // input buffer
                wgpu::BindGroupEntry {
                    binding: 100,
                    resource: input_buffer.as_entire_binding(),
                },
            ],
        });

        let bind_group1 = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("GaussianBlur bind_group1"),
            layout: &layout1,
            entries: &[wgpu::BindGroupEntry {
                binding: 100,
                resource: output_buffer.as_entire_binding(),
            }],
        });

        // 4) Create the compute pipeline
        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("GaussianBlur pipeline layout"),
                bind_group_layouts: &[&layout0, &layout1],
                push_constant_ranges: &[],
            });

        let cs_module = self
            .device
            .create_shader_module(wgpu::include_wgsl!("../shaders/gaussian_blur.wgsl"));
        let pipeline = self.create_compute_pipeline(&pipeline_layout, &cs_module, "main");

        // 5) Dispatch
        let groups_x = (img.width + 15) / 16;
        let groups_y = (img.height + 15) / 16;
        self.run_compute_job(
            &pipeline,
            &[&bind_group0, &bind_group1],
            groups_x,
            groups_y,
            1,
        );

        // 6) Retrieve data to new GpuImage
        let new_image = self.retrieve_storage_data(&output_buffer, input_size);

        new_image
    }
}

#[cfg(test)]
mod test {
    use super::*;
    const INPAINT_TEST_IMAGE: &str = "tests/testdata/MSL_MAHLI_INPAINT_Sol2904_V1.png";

    #[test]
    fn gpu_gaussian_blur() {
        let gpu = pollster::block_on(GpuContext::new());
        let img = Image::open(&String::from(INPAINT_TEST_IMAGE)).unwrap();
        let img = GpuImage::from_sciimg_rgb(&img);

        let radius = 2;
        let sigma = 2.8;

        gpu.gaussian_blur(&img, radius, sigma);
    }
}
