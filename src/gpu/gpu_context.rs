//! The GPU version of a `sciimg::Image`
use super::image::{dimensions::ImgDimensions, Empty, GpuImage, ImageUniform};
use crate::{
    enums, image::Image, max, min, path, Dn, DnVec, Mask, MaskVec, MaskedDnVec, MinMax, VecMath,
};
use encase::{
    internal::{ReadFrom, WriteInto},
    ArrayLength, ShaderSize, ShaderType, StorageBuffer,
};
use glam::{Vec3, Vec3A, Vec3Swizzles, Vec4, Vec4Swizzles};
use wgpu::{BufferUsages, Features};

/// A `gpu` wrapper, holding all the wgpu goodies we need to get stuff done
// NOTE: You should implement things ON this.
pub struct GpuContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    //TODO: Maybe store a compute pipeline in here? the only thing we're gonna be swapping in and out are shaders/entry points...
}

impl GpuContext {
    //TODO: Errors
    pub async fn new() -> Self {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("SciImg GPU Device"),
                required_features: Features::empty(),
                memory_hints: wgpu::MemoryHints::Performance,
                required_limits: wgpu::Limits::downlevel_defaults(),
                trace: wgpu::Trace::Off,
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

    pub fn create_compute_pipeline(
        &self,
        layout: &wgpu::PipelineLayout,
        shader_module: &wgpu::ShaderModule,
        entry_point: &str,
    ) -> wgpu::ComputePipeline {
        self.device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Sciimg Compute Pipeline"),
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
                label: Some("Sciimg Compute Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Sciimg Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(pipeline);

            bind_groups
                .iter()
                .enumerate()
                .for_each(|(idx, bind_group)| {
                    compute_pass.set_bind_group(idx as u32, *bind_group, &[]);
                });

            compute_pass.dispatch_workgroups(
                workgroup_count_x,
                workgroup_count_y,
                workgroup_count_z,
            );
        }

        self.queue.submit([encoder.finish()]);

        self.device.poll(wgpu::PollType::Wait).unwrap();
    }
}
