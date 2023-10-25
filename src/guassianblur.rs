use crate::imagebuffer::ImageBuffer;
use itertools::iproduct;

// fn normalize_kernel(in_kernel: &[f32], radius: usize) -> Vec<f32> {
//     let mut kernel = in_kernel.to_vec();
//     let mut sum = kernel[0 + 0 * radius];
//     (1..radius).for_each(|i| {
//         sum += 2.0 * (kernel[i + 0 * radius] + kernel[0 + i * radius]);
//     });

//     iproduct!(1..radius, 1..radius).for_each(|(i, j)| {
//         sum += 4.0 * kernel[i + j * radius];
//     });

//     (0..radius.pow(2)).for_each(|i| {
//         kernel[i] /= sum;
//     });

//     kernel
// }

// fn calculate_gaussian_kernel(len: usize, sigma: f32, normalize: bool) -> Vec<f32> {
//     let mut kernel: Vec<f32> = (0..len).map(|_| 0.0).collect();

//     iproduct!(0..len, 0..len).for_each(|(i, j)| {
//         let fi = i as f32;
//         let fj = j as f32;
//         kernel[i + j * len] = (-(fi.powi(2) + fj.powi(2)) / (2.0 * sigma.powi(2))).exp();
//     });
//     if normalize {
//         kernel = normalize_kernel(&kernel, len);
//     }
//     kernel
// }

fn calculate_gaussian_kernel_projection(
    in_kernel: &[f32],
    radius: i32,
    sigma: f32,
    normalize: bool,
) -> Vec<f32> {
    let mut kernel = in_kernel.to_vec();
    (0..(2 * radius - 1)).for_each(|i| {
        kernel[i as usize] =
            (-((radius as f32 - 1.0) - i as f32).powi(2) / (2.0 * sigma * sigma)).exp();
    });

    if normalize {
        let sum = (0..(2 * radius - 1))
            .map(|i| kernel[i as usize])
            .sum::<f32>();
        (0..(2 * radius - 1)).for_each(|i| {
            kernel[i as usize] /= sum;
        });
    }
    kernel
}

// fn calculate_half_1d_gaussian_kernel(radius: usize, sigma: f32) -> Vec<f32> {
//     let kernel: Vec<f32> = (0..radius as i32)
//         .map(|i: i32| (-i as f32 * i as f32 / (2.0 * sigma * sigma)).exp())
//         .collect();

//     let sum = kernel.clone().into_iter().map(|v| 2.0 * v).sum::<f32>();

//     kernel.into_iter().map(|v| v / sum).collect()
// }

pub fn gaussian_blur_1d_vec(values: &[f32], sigma: f32) -> Vec<f32> {
    let radius: i32 = (3.0 * sigma).ceil() as i32;
    let mut kernel: Vec<f32> = (0..(2 * radius - 1)).map(|_| 0.0).collect();
    kernel = calculate_gaussian_kernel_projection(&kernel, radius, sigma, true);

    let mut result = values.to_vec();

    iproduct!(0..values.len() as i32, 0..(2 * radius - 1)).for_each(|(i, j)| {
        let mut influence_src_idx = i - (radius - 1) + j;

        if influence_src_idx < 0 {
            influence_src_idx = 0;
        } else if influence_src_idx >= values.len() as i32 {
            influence_src_idx = values.len() as i32 - 1;
        }
        let influence_src = values[influence_src_idx as usize];
        result[i as usize] += influence_src * kernel[j as usize];
    });
    result
}

pub fn gaussian_blur_1d(values: &ImageBuffer, sigma: f32) -> ImageBuffer {
    ImageBuffer::from_vec(
        &gaussian_blur_1d_vec(&values.to_vector(), sigma),
        values.width,
        values.height,
    )
    .unwrap()
}

pub fn gaussian_blur_2d(buffer: &ImageBuffer, sigma: f32) -> ImageBuffer {
    let horiz = gaussian_blur_1d(&buffer, sigma);

    let vert = gaussian_blur_1d(&buffer.swap_axis(), sigma);

    horiz
        .scale(0.5)
        .unwrap()
        .add(&vert.swap_axis().scale(0.5).unwrap())
        .unwrap()
}

pub fn gaussian_blur_2d_nbands(buffers: &[ImageBuffer], sigma: f32) -> Vec<ImageBuffer> {
    buffers
        .into_iter()
        .map(|b| gaussian_blur_2d(&b, sigma))
        .collect()
}
