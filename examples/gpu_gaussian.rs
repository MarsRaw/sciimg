use sciimg::{gpu::image::GpuImage, image::Image};

const INPAINT_TEST_IMAGE: &str = "tests/testdata/MSL_MAHLI_INPAINT_Sol2904_V1.png";

fn main() {
    let gpu = pollster::block_on(sciimg::gpu::gpu_context::GpuContext::new());
    let img = Image::open(&String::from(INPAINT_TEST_IMAGE)).unwrap();
    let total: usize = img
        .buffers()
        .iter()
        .enumerate()
        .map(|(e, b)| {
            //
            println!("Band: {} contains {}", e, b.buffer.len());
            //
            b.buffer.len()
        })
        .sum();
    println!("Total: {}", total);
    println!("Num bands: {} ", img.num_bands());
    println!(
        "width:{} * height:{} = {}",
        img.width,
        img.height,
        img.width * img.height
    );

    let img = GpuImage::from_sciimg_rgb(&img);
    println!(
        "width:{} * height:{} = {}",
        img.width,
        img.height,
        img.width * img.height
    );

    let radius = 2;
    let sigma = 2.8;

    let res = gpu.gaussian_blur(&img, radius, sigma);

    println!(
        "RES\nwidth:{} * height:{} = {}",
        res.width,
        res.height,
        res.width * res.height
    );

    let res_as_sciimg = res.to_sciimg_rgb().unwrap();
    res_as_sciimg.save_rgb("gaussian_gpu.png");
}
