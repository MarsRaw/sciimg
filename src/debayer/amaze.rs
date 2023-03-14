/**
 * Port of libraw AMaZE demosaicing algorithm from Emil Martinec
 * Source: https://github.com/LibRaw/LibRaw-demosaic-pack-GPL3/blob/master/amaze_demosaic_RT.cc
 */
use crate::{error, imagebuffer::ImageBuffer, max, min, rgbimage::RgbImage};

static TS: i32 = 512;
static pre_mul: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

macro_rules! SQR {
    ($x: expr) => {
        ($x) * ($x)
    };
}

macro_rules! LIM {
    ($x: expr, $y:expr, $z:expr) => {{
        max!(y, min!(x, z))
    }};
}
macro_rules! ULIM {
    ($x: expr, $y:expr, $z:expr) => {{
        if $y < $z {
            LIM!(x, y, z)
        } else {
            LIM!(x, z, y)
        }
    }};
}

macro_rules! HCLIP {
    ($clip_pt: expr, $x:expr) => {
        min!(clip_pt, x)
    };
}

macro_rules! FC {
    ($row:expr, $col:expr) => {
        //return (imgdata.idata.filters >> (((row << 1 & 14) | (col & 1)) << 1) & 3);
        (0x16161616 >> ((($row << 1 & 14) | ($col & 1)) << 1) & 3)
    };
}

pub fn debayer(buffer: &ImageBuffer) -> error::Result<RgbImage> {
    let mut red =
        ImageBuffer::new_with_mask(buffer.width, buffer.height, &buffer.to_mask()).unwrap();
    let mut green =
        ImageBuffer::new_with_mask(buffer.width, buffer.height, &buffer.to_mask()).unwrap();
    let mut blue =
        ImageBuffer::new_with_mask(buffer.width, buffer.height, &buffer.to_mask()).unwrap();

    let height = buffer.height as i32;
    let width = buffer.width as i32;

    let clip_pt = min!(min!(pre_mul[0], pre_mul[1]), pre_mul[2]);
    let mut winx = 0;
    let mut winy = 0;
    let winw = width;
    let winh = height;

    let mut ex = 0;
    let mut ey = 0;

    let v1 = TS;
    let v2 = 2 * TS;
    let v3 = 3 * TS;
    let p1 = -TS + 1;
    let p2 = -2 * TS + 2;
    let p3 = -3 * TS + 3;
    let m1 = TS + 1;
    let m2 = 2 * TS + 2;
    let m3 = 3 * TS + 3;

    let nbr: [i32; 5] = [-v2, -2, 2, v2, 0];

    let eps = 1e-5;
    let epssq = 1e-10;

    let arthresh = 0.75;
    let nyqthresh = 0.5;
    let pmthresh = 0.25;
    let lbd = 1.0;
    let ubd = 1.0;

    let gaussodd = [
        0.14659727707323927,
        0.103592713382435,
        0.0732036125103057,
        0.0365543548389495,
    ];

    let gaussgrad = [
        0.07384411893421103,
        0.06207511968171489,
        0.0521818194747806,
        0.03687419286733595,
        0.03099732204057846,
        0.018413194161458882,
    ];

    let gauss1 = [0.3376688223162362, 0.12171198028231786, 0.04387081413862306];

    let gausseven = [0.13719494435797422, 0.05640252782101291];

    let gquinc = [0.169917, 0.108947, 0.069855, 0.0287182];

    // Start Parallizable:
    // In other words, this is where Emil enables OpenMP parallel if LIBRAW_USE_OPENMP is 1

    let mut top = 0;
    let mut left = 0;

    // Honestly this won't change ever while we hardcode it above
    // as RGGB
    if FC!(0, 0) == 1 {
        (ey, ex) = if FC!(0, 1) == 0 { (0, 1) } else { (1, 0) }
    } else {
        (ey, ex) = if FC!(0, 0) == 0 { (0, 0) } else { (1, 1) }
    }

    ((winy - 16)..(winy + height))
        .step_by(TS as usize - 32)
        .for_each(|top| {
            ((winx - 16)..(winx + width))
                .step_by(TS as usize - 32)
                .for_each(|left| {
                    // location of tile edge
                    let bottom = min!(top + TS, winy + height + 16);

                    // location of tile right edge
                    let right = min!(left + TS, winx + width + 16);

                    // tile width (=TS except for right edge of image)
                    let rr1 = bottom - top;

                    // tile height (=TS except for bottom edge of image)
                    let cc1 = right - left;

                    // tile vars
                    // counters for pixel location of the image
                    let mut row = 0;
                    let mut col = 0;

                    // min and max row/column in the tile
                    let mut rrmin = 0;
                    let mut rrmax = 0;
                    let mut ccmin = 0;
                    let mut ccmax = 0;

                    // counters for pixel location within the tile
                    let mut rr = 0;
                    let mut cc = 0;

                    // color index 0=R, 1=G, 2=B
                    let mut c = 0;

                    // pointer counters within the tile
                    let mut indx = 0;
                    let mut indx1 = 0;

                    // direction counter for nbrs[]
                    let mut dir = 0;

                    // dummy indices
                    let mut i = 0;
                    let mut j = 0;

                    // +1 or -1
                    let mut sgn = 0;

                    // TODO: Pick up on line 281...
                });
        });

    let newimage = RgbImage::new_from_buffers_rgb(&red, &green, &blue, buffer.mode).unwrap();
    Ok(newimage)
}
