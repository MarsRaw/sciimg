use sciimg::{
    gpu::{gpu_context::GpuContext, image::GpuImage},
    image::Image,
};

const EXAMPLE_IMG: &str = "tests/testdata/MSL_MAHLI_INPAINT_Sol2904_V1.png";

fn main() {
    _ = pretty_env_logger::init();
    let gpu = pollster::block_on(GpuContext::new());
    let start_img = Image::open(&String::from(EXAMPLE_IMG)).unwrap();
    let gpu_img = GpuImage::from_sciimg_rgb(&start_img);

    let radius = 2;
    let sigma = 2.8;
    let (width, height) = (start_img.width, start_img.height);

    let res = gpu.gaussian_blur(&gpu_img, width as u32, height as u32, radius, sigma);

    let res_as_sciimg = res.to_sciimg_rgb(width, height).unwrap();
    res_as_sciimg.save_rgb("gaussian_gpu.png");
}
