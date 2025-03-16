//! image processing
#![allow(unused_imports, dead_code)]
use crate::{
    enums,
    gpu::{
        gpu_context::GpuContext,
        image::{Empty, GpuImage},
    },
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
        let uniform_data = GaussianBlurUniform {
            radius,
            sigma,
            width: img.width,
            height: img.height,
        };

        // 1) Create & fill input buffer
        let mut input_bytes = Vec::new();
        {
            let mut sbuf = encase::StorageBuffer::new(&mut input_bytes);
            sbuf.write(img).unwrap();
        }
        let input_size = input_bytes.len() as wgpu::BufferAddress;
        let input_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GaussianBlur Input"),
            size: input_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&input_buffer, 0, &input_bytes);

        // 2) Output buffer
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GaussianBlur Output"),
            size: input_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // 3) Uniform buffer
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

        // 4) Bind group layouts
        let layout0 = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("GaussianBlur layout0"),
                entries: &[
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
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 100,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // 5) Bind groups
        let bind_group0 = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("GaussianBlur bind_group0"),
            layout: &layout0,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
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

        // 6) Pipeline
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

        // 7) Dispatch job.
        let gx = (img.width + 15) / 16;
        let gy = (img.height + 15) / 16;
        self.run_compute_job(&pipeline, &[&bind_group0, &bind_group1], gx, gy, 1);

        // 8) Readback DtoH results
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GaussianBlur Staging"),
            size: input_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        {
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, input_size);
            self.queue.submit([encoder.finish()]);
        }
        self.device.poll(wgpu::PollType::Wait).unwrap();

        // Map (DtoH) results
        let buffer_slice = staging_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |_| ());
        self.device.poll(wgpu::PollType::Wait).unwrap();
        let mapped_range = buffer_slice.get_mapped_range();
        let mut final_bytes = mapped_range.to_vec();
        drop(mapped_range);
        staging_buffer.unmap();

        // Decode
        let sbuf = encase::StorageBuffer::new(&mut final_bytes);
        let mut new_image = GpuImage::empty();
        sbuf.read(&mut new_image).unwrap();

        new_image
    }
}

#[cfg(test)]
mod test {
    use super::*;
    const INPAINT_TEST_IMAGE: &str =
        "tests/testdata/ZL0_0038_0670307360_057ECM_N0031392ZCAM08007_1100LUJ.png";

    #[test]
    fn gpu_gaussian_blur() {
        let gpu = pollster::block_on(GpuContext::new());
        let img = Image::open(&String::from(INPAINT_TEST_IMAGE)).unwrap();
        let img = GpuImage::from_sciimg_rgb(&img);

        let radius = 2;
        let sigma = 2.8;

        let res = gpu.gaussian_blur(&img, radius, sigma);

        let res_as_sciimg = res.to_sciimg_rgb().unwrap();
    }
}
