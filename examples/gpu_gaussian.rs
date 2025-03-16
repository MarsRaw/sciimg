use log::info;
use sciimg::{
    enums,
    gpu::{gpu_context::GpuContext, image::GpuImage},
    image::Image,
    prelude::ImageBuffer,
};
use std::time::Instant;

const EXAMPLE_IMG: &str =
    "tests/testdata/ZL0_0053_0671642352_402ECM_N0032046ZCAM05025_110085J01.png";

fn main() -> anyhow::Result<()> {
    _ = pretty_env_logger::init();

    let gpu_start = Instant::now();
    let gpu = pollster::block_on(GpuContext::new());
    let gpu_creation_time = gpu_start.elapsed();
    info!("GPU context created in: {:?}", gpu_creation_time);

    let load_start = Instant::now();
    let mut start_img = Image::open(&String::from(EXAMPLE_IMG)).unwrap();
    let load_time = load_start.elapsed();
    info!("Original sciimg loaded in: {:?}", load_time);

    let conversion_start = Instant::now();
    let gpu_img = GpuImage::from_sciimg_rgb(&start_img);
    let conversion_time = conversion_start.elapsed();
    info!("Conversion to gpuimg complete in: {:?}", conversion_time);

    let radius = 32;
    let sigma = 2.8;
    let (width, height) = (start_img.width, start_img.height);

    let blur_start = Instant::now();
    let res = gpu.gaussian_blur(&gpu_img, width as u32, height as u32, radius, sigma);
    let blur_time = blur_start.elapsed();
    info!("Gaussian blur completed in: {:?}", blur_time);

    info!("Saving the processed image...");
    let save_start = Instant::now();
    let size = gpu_img.data.len();
    let mut red_buf = Vec::with_capacity(size);
    let mut green_buf = Vec::with_capacity(size);
    let mut blue_buf = Vec::with_capacity(size);
    for px in &res.data {
        red_buf.push(px.x);
        green_buf.push(px.y);
        blue_buf.push(px.z);
        // Ignoring alpha for now.
        // Although note for stride reasons we always use vec4<f32> on the GPU side so
        // alpha is ALWAYS there.
    }

    let red_band =
        ImageBuffer::from_vec_as_mode(&red_buf, width, height, enums::ImageMode::U16BIT)?;

    let green_band =
        ImageBuffer::from_vec_as_mode(&green_buf, width, height, enums::ImageMode::U16BIT)?;

    let blue_band =
        ImageBuffer::from_vec_as_mode(&blue_buf, width, height, enums::ImageMode::U16BIT)?;

    start_img.set_band(&red_band, 0);
    start_img.set_band(&green_band, 1);
    start_img.set_band(&blue_band, 2);

    start_img.save_rgba("gaussian_gpu.png");
    let save_time = save_start.elapsed();
    info!(
        "Processed image saved as 'gaussian_gpu.png' in: {:?}",
        save_time
    );

    info!("Total runtime {}", gpu_start.elapsed().as_secs_f32());
    Ok(())
}
