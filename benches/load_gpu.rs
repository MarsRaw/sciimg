use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pollster::block_on;
use sciimg::{
    gpu::{gpu_context::GpuContext, image::GpuImage},
    image::Image,
};
use std::time::Duration;

fn setup_benchmark_data() -> Image {
    const INPAINT_TEST_IMAGE: &str = "tests/testdata/MSL_MAHLI_INPAINT_Sol2904_V1.png";
    Image::open(&String::from(INPAINT_TEST_IMAGE)).unwrap()
}

fn bench_gpu_processing(c: &mut Criterion) {
    // Set up a benchmark group with longer sample time
    let mut group = c.benchmark_group("gpu_image_processing");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10); // Reduce sample size for GPU benchmarks

    // Load test image
    let test_image = setup_benchmark_data();

    group.bench_function("gpu_init_compute_then_retrieve", |b| {
        // Initialize GPU context - this is EXPLICITLY here to show how long the startup time is.
        let gpu_context = block_on(GpuContext::new());

        // Convert image to GPU format
        let gpu_image = GpuImage::from_sciimg_rgb(&test_image);

        b.iter(|| {
            // Create and use storage buffer directly
            let (buffer, buffer_size) = gpu_context.create_storage_buffer(
                &gpu_image,
                Some("Benchmark Input Buffer"),
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
            );

            // Retrieve the data back
            let result = black_box(gpu_context.retrieve_storage_data(&buffer, buffer_size));

            // Verify dimensions to ensure the test is valid
            assert_eq!(result.dimensions(), gpu_image.dimensions());
        });
    });

    // Add a benchmark for just the conversion to GPU format
    group.bench_function("convert_to_gpu_format", |b| {
        b.iter(|| {
            let gpu_image = black_box(GpuImage::from_sciimg_rgb(&test_image));
            assert_eq!(
                gpu_image.dimensions(),
                (test_image.width as u32, test_image.height as u32)
            );
        });
    });

    group.finish();
}

// If you want to test a specific compute operation, add another benchmark like this:
fn bench_compute_operation(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_compute_operations");
    group.measurement_time(Duration::from_secs(5));

    // Setup data
    let test_image = setup_benchmark_data();
    // Note how the GPU setup is OUTISDE the loop here :wink
    let gpu_context = block_on(GpuContext::new());
    let gpu_image = GpuImage::from_sciimg_rgb(&test_image);

    // Test specific compute operation
    group.bench_function("simple_compute_job", |b| {
        // Create input storage buffer
        let (input_buffer, input_size) = gpu_context.create_storage_buffer(
            &gpu_image,
            Some("Compute Input Buffer"),
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        );

        // Create input bind group layout
        let input_bind_group_layout =
            gpu_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("Input Bind Group Layout"),
                });

        // Create input bind group
        let input_bind_group = gpu_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &input_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: input_buffer.as_entire_binding(),
                }],
                label: Some("Input Bind Group"),
            });

        // Create output buffer
        let output_buffer = gpu_context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: input_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Create output bind group layout
        let output_bind_group_layout =
            gpu_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    label: Some("Output Bind Group Layout"),
                });

        // Create output bind group
        let output_bind_group = gpu_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &output_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: output_buffer.as_entire_binding(),
                }],
                label: Some("Output Bind Group"),
            });

        // Create pipeline layout
        let pipeline_layout =
            gpu_context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Pipeline Layout"),
                    bind_group_layouts: &[&input_bind_group_layout, &output_bind_group_layout],
                    push_constant_ranges: &[],
                });

        // let cs = gpu_context
        //     .device
        //     .create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        // Create shader module with a simple copy operation
        // Using the image dimensions for proper indexing
        let width = gpu_image.width;
        let shader_src = format!(
            r#"
            @group(0) @binding(0) var<storage, read> input_data: array<vec3<f32>>;
            @group(1) @binding(0) var<storage, read_write> output_data: array<vec3<f32>>;
            
            @compute @workgroup_size(16, 16)
            fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
                let width = {width}u;
                let index = global_id.x + global_id.y * width;
                if (index >= arrayLength(&input_data)) {{
                    return;
                }}
                
                // Simple operation: copy with slight modification
                output_data[index] = input_data[index] * 1.1;
            }}
        "#
        );

        let cs = gpu_context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Compute Shader"),
                source: wgpu::ShaderSource::Wgsl(shader_src.into()),
            });

        let compute_pipeline = gpu_context.create_compute_pipeline(&pipeline_layout, &cs, "main");

        // Benchmark the compute operation
        b.iter(|| {
            // Run compute job
            gpu_context.run_compute_job(
                &compute_pipeline,
                &[&input_bind_group, &output_bind_group],
                (gpu_image.width as u32 + 15) / 16, // Ceiling division for workgroup count
                (gpu_image.height as u32 + 15) / 16,
                1,
            );

            // Retrieve results (optional, if you want to verify)
            if false {
                // Set to true to verify during development
                let result =
                    black_box(gpu_context.retrieve_storage_data(&output_buffer, input_size));
                // assert_eq!(result.dimensions(), gpu_image.dimensions());
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_gpu_processing, bench_compute_operation);
criterion_main!(benches);
