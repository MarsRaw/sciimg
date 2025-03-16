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
use wgpu::{BufferUsages, Features};

use super::image::{dimensions::ImgDimensions, Empty, GpuImage, ImageUniform};
/// A `gpu` wrapper, holding all the wgpu goodies we need to get stuff done
// NOTE: You should implement things ON this.
pub struct GpuContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl GpuContext {
    pub async fn new() -> Self {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("SciImg GPU Device"),
                // These features are required to use `binding_array` in your wgsl.
                // Without them your shader may fail to compile.
                required_features: Features::empty(),
                // Features::BUFFER_BINDING_ARRAY
                // | Features::STORAGE_RESOURCE_BINDING_ARRAY
                // | Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
                memory_hints: wgpu::MemoryHints::Performance,
                required_limits: wgpu::Limits {
                    ..Default::default()
                },
                ..Default::default()
            })
            .await
            .unwrap();

        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }

    /// Host to Device.
    pub fn htod<C: ShaderSize + WriteInto + ReadFrom>(
        &self,
        img: &GpuImage<C>,
    ) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
        let buffer_size = (std::mem::size_of::<C>() * img.data.len()) as wgpu::BufferAddress;
        let buffer_desc = wgpu::BufferDescriptor {
            label: Some("GpuImage"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        let storage = self.device.create_buffer(&buffer_desc);
        self.queue.write_buffer(
            &storage,
            0,
            &img.as_wgsl_bytes()
                .expect("Unable to write your GpuImage to GPU buffer."),
        );

        let bind_group_layout =
            self.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        // Storate is (for **THIS** repo by convention up around 100)
                        binding: 100,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("SciImg Storage Bind Group Layout"),
                });

        let (uniform, _uniform_size) = self.add_image_info_to_uniform(&img);

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 100,
                    resource: storage.as_entire_binding(),
                },
            ],
            label: Some("SciImg Storage Bind Group"),
        });

        (storage, bind_group_layout, bind_group)
    }

    pub fn create_compute_pipeline(
        &self,
        layout: &wgpu::PipelineLayout,
        shader_module: &wgpu::ShaderModule,
        entry_point: &str,
    ) -> wgpu::ComputePipeline {
        self.device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Compute Pipeline"),
                layout: Some(layout),
                module: shader_module,
                entry_point: Some(entry_point),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            })
    }

    pub fn run_compute_job(
        &self,
        pipeline: &wgpu::ComputePipeline,
        bind_groups: &[&wgpu::BindGroup],
        workgroup_count_x: u32,
        workgroup_count_y: u32,
        workgroup_count_z: u32,
    ) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(pipeline);

            for (idx, bind_group) in bind_groups.iter().enumerate() {
                compute_pass.set_bind_group(idx as u32, *bind_group, &[]);
            }

            compute_pass.dispatch_workgroups(
                workgroup_count_x,
                workgroup_count_y,
                workgroup_count_z,
            );
        }

        self.queue.submit([encoder.finish()]);

        self.device.poll(wgpu::PollType::Wait).unwrap();
    }

    pub fn retrieve_storage_data(
        &self,
        output_buffer: &wgpu::Buffer,
        buffer_size: u64,
    ) -> GpuImage<Vec3> {
        // Create a staging buffer for reading back data
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Copy from output buffer to staging buffer
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Copy Encoder"),
            });
        encoder.copy_buffer_to_buffer(output_buffer, 0, &staging_buffer, 0, buffer_size);
        self.queue.submit([encoder.finish()]);

        // Map the staging buffer and read the data
        let buffer_slice = staging_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, move |_r| {});
        self.device.poll(wgpu::PollType::Wait).unwrap();

        // Read the mapped range into a Vec<u8>
        let mapped_range = buffer_slice.get_mapped_range();
        let mut byte_data = mapped_range.to_vec();

        // Now we can unmap and drop the mapped range
        drop(mapped_range);
        staging_buffer.unmap();

        // Create a mutable StorageBuffer with our copied data
        let buffer = encase::StorageBuffer::new(&mut byte_data);

        // Create an empty result and read into it
        let mut result: GpuImage<Vec3> = GpuImage::empty();
        buffer
            .read(&mut result)
            .expect("Failed to read from buffer");

        result
    }

    pub fn store_img<T: ShaderType + WriteInto>(
        &self,
        data: &T,
        label: Option<&str>,
        usage: wgpu::BufferUsages,
    ) -> (wgpu::Buffer, u64) {
        // Create buffer to hold our serialized data
        let mut byte_buffer = Vec::new();
        let mut buffer = encase::StorageBuffer::new(&mut byte_buffer);
        buffer.write(data).expect("Failed to write to buffer");

        // Create the GPU buffer with appropriate usage flags
        let gpu_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label,
            size: byte_buffer.len() as u64,
            usage,
            mapped_at_creation: false,
        });

        // Write our serialized data to the GPU buffer
        self.queue.write_buffer(&gpu_buffer, 0, &byte_buffer);

        // Return the buffer and its size
        (gpu_buffer, byte_buffer.len() as u64)
    }

    pub fn add_image_info_to_uniform<C>(&self, img: &GpuImage<C>) -> (wgpu::Buffer, u64)
    where
        C: ShaderSize + WriteInto + ReadFrom,
    {
        let uniform = ImageUniform::from_img(&img);
        let mut buffer = encase::UniformBuffer::new(Vec::new());
        buffer.write(&uniform).unwrap();
        let usage = BufferUsages::UNIFORM | BufferUsages::COPY_DST;
        let byte_buffer = buffer.into_inner();
        let gpu_uniform_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Image Uniform Buffer"),
            size: byte_buffer.len() as u64,
            usage,
            mapped_at_creation: false,
        });

        self.queue
            .write_buffer(&gpu_uniform_buffer, 0, &byte_buffer);
        (gpu_uniform_buffer, byte_buffer.len() as u64)
    }
}
