
extern crate lab;
use lab::{
    rgbs_to_labs,
    labs_to_rgbs,
    Lab
};

use crate::{
    rgbimage::RgbImage,
    enums,
    imagebuffer::ImageBuffer,
    blur
};

struct SplitLab {
    l: Vec<u16>,
    a: Vec<u16>,
    b: Vec<u16>
}

fn split_lab_channels(lab_array:&[Lab]) -> SplitLab {
    let mut l: Vec<u16> = Vec::with_capacity(lab_array.len());
    l.resize(lab_array.len(), 0);

    let mut a: Vec<u16> = Vec::with_capacity(lab_array.len());
    a.resize(lab_array.len(), 0);

    let mut b: Vec<u16> = Vec::with_capacity(lab_array.len());
    b.resize(lab_array.len(), 0);

    for i in 0..lab_array.len() {
        l[i] = lab_array[i].l as u16;

        a[i] = lab_array[i].a as u16;

        b[i] = lab_array[i].b as u16;
    }

    SplitLab{l, 
        a, 
        b
    }
}

fn combine_lab_channels(splitlab:&SplitLab) -> Vec<Lab> {

    let mut lab_array:Vec<Lab> = Vec::with_capacity(splitlab.a.len());
    lab_array.resize(splitlab.a.len(), Lab{l:0.0, a:0.0, b:0.0});

    for i in 0..splitlab.a.len() {
        lab_array[i].l = splitlab.l[i] as f32;
        lab_array[i].a = splitlab.a[i] as f32;
        lab_array[i].b = splitlab.b[i] as f32;
    }

    lab_array
}

pub fn color_noise_reduction(image:&mut RgbImage, amount:i32) -> RgbImage {
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
    split_channels.a = blur::blur_vec_u16(&split_channels.a, image.width, image.height, amount as f32);
    split_channels.b = blur::blur_vec_u16(&split_channels.b, image.width, image.height, amount as f32);
    
    let labs_recombined = combine_lab_channels(&split_channels);

    let rgbs = labs_to_rgbs(&labs_recombined);

    let mut red = ImageBuffer::new_with_mask(image.width, image.height, &image.get_band(0).mask).unwrap();
    let mut green = ImageBuffer::new_with_mask(image.width, image.height, &image.get_band(1).mask).unwrap();
    let mut blue = ImageBuffer::new_with_mask(image.width, image.height, &image.get_band(2).mask).unwrap();

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

