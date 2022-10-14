extern crate lab;
use lab::{labs_to_rgbs, rgbs_to_labs, Lab};

use crate::{blur, enums, imagebuffer::ImageBuffer, rgbimage::RgbImage};

struct SplitLab {
    l: Vec<u8>,
    a: Vec<u8>,
    b: Vec<u8>,
}

fn split_lab_channels(lab_array: &[Lab]) -> SplitLab {
    let mut l: Vec<u8> = vec![0; lab_array.len()];
    let mut a: Vec<u8> = vec![0; lab_array.len()];
    let mut b: Vec<u8> = vec![0; lab_array.len()];

    for i in 0..lab_array.len() {
        l[i] = lab_array[i].l as u8;
        a[i] = lab_array[i].a as u8;
        b[i] = lab_array[i].b as u8;
    }

    SplitLab { l, a, b }
}

fn combine_lab_channels(splitlab: &SplitLab) -> Vec<Lab> {
    let mut lab_array: Vec<Lab> = Vec::with_capacity(splitlab.a.len());
    lab_array.resize(
        splitlab.a.len(),
        Lab {
            l: 0.0,
            a: 0.0,
            b: 0.0,
        },
    );

    for (i, item) in lab_array.iter_mut().enumerate().take(splitlab.a.len()) {
        item.l = splitlab.l[i] as f32;
        item.a = splitlab.a[i] as f32;
        item.b = splitlab.b[i] as f32;
    }

    lab_array
}

pub fn color_noise_reduction(image: &mut RgbImage, amount: i32) -> RgbImage {
    // We're juggling a couple different data structures here so we need to
    // convert the imagebuffer to a vec that's expected by lab and fastblur...
    let mut data: Vec<[u8; 3]> = Vec::with_capacity(image.width * image.height);
    data.resize(image.width * image.height, [0, 0, 0]);

    for y in 0..image.height {
        for x in 0..image.width {
            let r = image.get_band(0).get(x, y).unwrap() as u8;
            let g = image.get_band(1).get(x, y).unwrap() as u8;
            let b = image.get_band(2).get(x, y).unwrap() as u8;
            let idx = (y * image.width) + x;
            data[idx][0] = r;
            data[idx][1] = g;
            data[idx][2] = b;
        }
    }

    let labs = rgbs_to_labs(&data);

    let mut split_channels = split_lab_channels(&labs);

    split_channels.a =
        blur::blur_vec_u8(&split_channels.a, image.width, image.height, amount as f32);
    split_channels.b =
        blur::blur_vec_u8(&split_channels.b, image.width, image.height, amount as f32);

    let labs_recombined = combine_lab_channels(&split_channels);

    let rgbs = labs_to_rgbs(&labs_recombined);

    let mut red =
        ImageBuffer::new_with_mask(image.width, image.height, &image.get_band(0).to_mask())
            .unwrap();
    let mut green =
        ImageBuffer::new_with_mask(image.width, image.height, &image.get_band(1).to_mask())
            .unwrap();
    let mut blue =
        ImageBuffer::new_with_mask(image.width, image.height, &image.get_band(2).to_mask())
            .unwrap();

    for y in 0..image.height {
        for x in 0..image.width {
            let idx = (y * image.width) + x;
            red.put(x, y, rgbs[idx][0] as f32);
            green.put(x, y, rgbs[idx][1] as f32);
            blue.put(x, y, rgbs[idx][2] as f32);
        }
    }

    RgbImage::new_from_buffers_rgb(&red, &green, &blue, enums::ImageMode::U8BIT).unwrap()
}
