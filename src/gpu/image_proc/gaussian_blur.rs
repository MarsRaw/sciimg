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
    internal::{CreateFrom, ReadFrom, WriteInto},
    ArrayLength, ShaderSize, ShaderType, StorageBuffer,
};
use glam::{Vec4, Vec4Swizzles};
use thiserror;
use wgpu::Features;

#[derive(ShaderType)]
struct GaussianBlurUniform {
    pub radius: u32,
    pub sigma: f32,
    pub width: u32,
    pub height: u32,
}
// #[derive(ShaderType)]
// pub struct GpuImage {
//     length: ArrayLength,
//     #[size(runtime)]
//     pub data: Vec<Vec4>,
// }
impl GpuContext {
    pub fn gaussian_blur(
        &self,
        img: &GpuImage,
        width: u32,
        height: u32,
        radius: u32,
        sigma: f32,
    ) -> GpuImage {
        let uniform_data = GaussianBlurUniform {
            radius,
            sigma,
            width,
            height,
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

        // 2a) Output buffer
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GaussianBlur Output"),
            size: input_buffer.size(), // Must be the same
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        // We don't write to this in Rust code, because the shader writes to it in the shader code.

        // 2b) we make a final buffer that can be read FROM the cpu
        let readback_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GaussianBlur Staging"),
            size: input_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // 3) Uniform buffer
        let mut uniform_bytes = Vec::new();
        {
            let mut ubuf = encase::UniformBuffer::new(&mut uniform_bytes);
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
        // @group(0)
        let layout0 = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("GaussianBlur Inputs & Uniforms"),
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
                        binding: 1,
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

        // @group(1)
        let layout1 = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("GaussianBlur ReadBack"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
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
                    binding: 1,
                    resource: input_buffer.as_entire_binding(),
                },
            ],
        });
        let bind_group1 = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("GaussianBlur bind_group1"),
            layout: &layout1,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
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

        // 7) Bind things TO that Pipeline
        let bind_groups: &[&wgpu::BindGroup] = &[&bind_group0, &bind_group1];
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Sciimg Compute Encoder"),
            });

        // 8) Build a compute pass for the Pipeline, bind the binds to it.
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Sciimg Compute Pass"),
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(&pipeline);
        // @group(0..n) binding(0..n) are set for the ComputePass
        bind_groups
            .iter()
            .enumerate()
            .for_each(|(idx, bind_group)| {
                compute_pass.set_bind_group(idx as u32, *bind_group, &[]);
            });

        // 9) Dispatch the work! ** Actually run shit on the GPU **
        let gx = (width + 15) / 16;
        let gy = (height + 15) / 16;
        compute_pass.dispatch_workgroups(gx, gy, 1);
        drop(compute_pass);

        // 10) enque a copy Device -> Host & submit it.
        encoder.copy_buffer_to_buffer(&output_buffer, 0, &readback_buffer, 0, output_buffer.size());
        let submit = encoder.finish();
        self.queue.submit([submit]);

        // 11) Read stuff back n wait...
        let buffer_slice = readback_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |_| ());
        self.device.poll(wgpu::PollType::Wait).unwrap();

        // 12) Map (DtoH) results
        let mapped_range = buffer_slice.get_mapped_range();
        let mut final_bytes = mapped_range.to_vec();
        drop(mapped_range);
        readback_buffer.unmap();
        assert!(
            !final_bytes.is_empty(),
            "Failed to copy any data back from the Shader's output..."
        );

        // 13) Decode
        let sbuf = encase::StorageBuffer::new(&mut final_bytes);
        let mut new_image: GpuImage = GpuImage::empty();
        match sbuf.read(&mut new_image) {
            Ok(_) => {
                assert!(!new_image.data.is_empty());
                println!("Successfully read data: {} elements", new_image.data.len())
            }
            Err(e) => panic!("Failed to deserialize buffer: {:?}", e),
        };

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
        _ = pretty_env_logger::init();
        let gpu = pollster::block_on(GpuContext::new());
        let start_img = Image::open(&String::from(INPAINT_TEST_IMAGE)).unwrap();
        let gpu_img = GpuImage::from_sciimg_rgb(&start_img);

        let radius = 2;
        let sigma = 2.8;
        let (width, height) = (start_img.width, start_img.height);

        let res = gpu.gaussian_blur(&gpu_img, width as u32, height as u32, radius, sigma);
        assert_eq!(start_img.get_band(0).buffer.len(), res.data.len(),);
        let res_as_sciimg = res.to_sciimg_rgb(width, height).unwrap();

        assert_eq!(
            start_img.get_band(0).buffer.len(),
            res_as_sciimg.get_band(0).buffer.len(),
        )
    }
}
