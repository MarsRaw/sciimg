use sciimg::{
    gpu::{gpu_context::GpuContext, image::GpuImage},
    image::Image,
};
use std::time::Instant;

const EXAMPLE_IMG: &str =
    "tests/testdata/ZL0_0053_0671642352_402ECM_N0032046ZCAM05025_110085J01.png";

fn main() {
    _ = pretty_env_logger::init();
    println!("Starting GPU context creation...");
    let gpu_start = Instant::now();
    let gpu = pollster::block_on(GpuContext::new());
    let gpu_creation_time = gpu_start.elapsed();
    println!("GPU context created in: {:?}", gpu_creation_time);

    println!("Loading original sciimg...");
    let load_start = Instant::now();
    let start_img = Image::open(&String::from(EXAMPLE_IMG)).unwrap();
    let load_time = load_start.elapsed();
    println!("Original sciimg loaded in: {:?}", load_time);

    println!("Converting sciimg to gpuimg...");
    let conversion_start = Instant::now();
    let gpu_img = GpuImage::from_sciimg_rgb(&start_img);
    let conversion_time = conversion_start.elapsed();
    println!("Conversion to gpuimg complete in: {:?}", conversion_time);

    let radius = 2;
    let sigma = 2.8;
    let (width, height) = (start_img.width, start_img.height);

    println!("Starting Gaussian blur...");
    let blur_start = Instant::now();
    let res = gpu.gaussian_blur(&gpu_img, width as u32, height as u32, radius, sigma);
    let blur_time = blur_start.elapsed();
    println!("Gaussian blur completed in: {:?}", blur_time);

    println!("Saving the processed image...");
    let save_start = Instant::now();
    let res_as_sciimg = res.to_sciimg_rgb(width, height).unwrap();
    res_as_sciimg.save_rgb("gaussian_gpu.png");
    let save_time = save_start.elapsed();
    println!(
        "Processed image saved as 'gaussian_gpu.png' in: {:?}",
        save_time
    );
}
