use sciimg::{gpu::image::GpuImage, image::Image};

const INPAINT_TEST_IMAGE: &str = "tests/testdata/MSL_MAHLI_INPAINT_Sol2904_V1.png";

fn main() {
    let gpu = pollster::block_on(sciimg::gpu::gpu_context::GpuContext::new());
    let img = Image::open(&String::from(INPAINT_TEST_IMAGE)).unwrap();
    let img = GpuImage::from_sciimg_rgb(&img);

    let radius = 2;
    let sigma = 2.8;

    let result = gpu.gaussian_blur(&img, radius, sigma);
    
}
