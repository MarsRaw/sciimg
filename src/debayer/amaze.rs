/**
 * Port of libraw AMaZE demosaicing algorithm from Emil Martinec
 * Source: https://github.com/LibRaw/LibRaw-demosaic-pack-GPL3/blob/master/amaze_demosaic_RT.cc
 */
// NOTE: Clippy is going to have an absolute fielday with this code...
use crate::{
    error, imagebuffer::ImageBuffer, max, min, prelude::ImageMode, rgbimage::RgbImage, DnVec,
};
use std::ops::{Index, IndexMut};

static TS: i32 = 512;
static pre_mul: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

macro_rules! SQR {
    ($x: expr) => {
        ($x) * ($x)
    };
}

macro_rules! fabs {
    ($x: expr) => {
        ($x).abs()
    };
}

macro_rules! LIM {
    ($x: expr, $y:expr, $z:expr) => {{
        max!($y, min!($x, $z))
    }};
}
macro_rules! ULIM {
    ($x: expr, $y:expr, $z:expr) => {{
        if $y < $z {
            LIM!($x, $y, $z)
        } else {
            LIM!($x, $z, $y)
        }
    }};
}

macro_rules! HCLIP {
    ($clip_pt: expr, $x:expr) => {
        min!($clip_pt, $x)
    };
}

macro_rules! FC {
    ($row:expr, $col:expr) => {
        //return (imgdata.idata.filters >> (((row << 1 & 14) | (col & 1)) << 1) & 3);
        (0x94949494_u32 >> (((($row as u32) << 1 & 14) | (($col as u32) & 1)) << 1) & 3) as i32
    };
}

macro_rules! CLIP {
    ($x: expr) => {
        LIM!($x, 0.0, 65535.0_f32)
    };
}

/// Minimally implemented vector to allow for i32 (or other) indexing.
/// Intent is to keep from having `usize` casts all over the place.
/// Not one of my best ideas...
#[derive(Debug, Clone)]
struct Vek<T: Clone> {
    v: Vec<T>,
}

impl<T: Clone> Vek<T> {
    fn with_capacity(size: usize) -> Vek<T> {
        Vek {
            v: Vec::with_capacity(size),
        }
    }

    fn resize(&mut self, new_len: usize, fill_value: T) {
        self.v.resize(new_len, fill_value);
    }

    fn is_empty(&self) -> bool {
        self.v.is_empty()
    }

    fn len(&self) -> usize {
        self.v.len()
    }
}

impl<T: Clone> Index<usize> for Vek<T> {
    type Output = T;
    fn index<'a>(&'a self, i: usize) -> &'a T {
        &self.v[i]
    }
}

impl<T: Clone> IndexMut<usize> for Vek<T> {
    fn index_mut<'a>(&'a mut self, i: usize) -> &'a mut T {
        &mut self.v[i]
    }
}

impl<T: Clone> Index<i32> for Vek<T> {
    type Output = T;
    fn index<'a>(&'a self, i: i32) -> &'a T {
        if i < 0 {
            panic!("Index out of bounds, less than zero: {}", i);
        }
        &self.v[i as usize]
    }
}

impl<T: Clone> IndexMut<i32> for Vek<T> {
    fn index_mut<'a>(&'a mut self, i: i32) -> &'a mut T {
        if i < 0 {
            panic!("Index out of bounds, less than zero: {}", i);
        }
        &mut self.v[i as usize]
    }
}

fn vec_of_size<T: Clone>(size: usize, fill_value: T) -> Vek<T>
where
    T: Clone,
{
    let mut v = Vek::with_capacity(size);
    v.resize(size, fill_value);
    v
}

// macro_rules! vek {
//     ($elem:expr; $n:expr) => {
//         vec_of_size($elem, $n)
//     };
// }

fn imagebuffer_to_vek_array(buffer: &ImageBuffer) -> Vek<Vek<f32>> {
    let mut image = vec_of_size(buffer.width * buffer.height, vec_of_size(3, 0.0_f32));
    for y in (0..buffer.height) {
        for x in (0..buffer.width) {
            image[y * buffer.width + x][0] = buffer.get(x, y).unwrap_or(0.0);
            image[y * buffer.width + x][1] = buffer.get(x, y).unwrap_or(0.0);
            image[y * buffer.width + x][2] = buffer.get(x, y).unwrap_or(0.0);
        }
    }

    image
}

fn vek_array_to_rgbimage(
    v: &Vek<Vek<f32>>,
    width: usize,
    height: usize,
    mode: ImageMode,
) -> RgbImage {
    let mut image = RgbImage::new_with_bands(width, height, 3, mode).unwrap();
    for y in (0..height) {
        for x in (0..width) {
            let indx = y * width + x;
            image.put(x, y, v[indx][0], 0);
            image.put(x, y, v[indx][1], 1);
            image.put(x, y, v[indx][2], 2);
        }
    }
    image
}

pub fn debayer(buffer: &ImageBuffer) -> error::Result<RgbImage> {
    let mut red =
        ImageBuffer::new_with_mask(buffer.width, buffer.height, &buffer.to_mask()).unwrap();
    let mut green =
        ImageBuffer::new_with_mask(buffer.width, buffer.height, &buffer.to_mask()).unwrap();
    let mut blue =
        ImageBuffer::new_with_mask(buffer.width, buffer.height, &buffer.to_mask()).unwrap();

    // placeholder. NOT CORRECT!!!!
    //let mut image = vec_of_size(buffer.height, vec_of_size(buffer.width as usize, 0.0_f32));

    //let mut image = vec_of_size(buffer.width * buffer.height, vec_of_size(3, 0.0_f32));
    let mut image = imagebuffer_to_vek_array(buffer);

    let mut rgb = vec_of_size((TS * TS) as usize, vec_of_size(3, 0.0_f32));

    let mut delh = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut delv = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut delhsq = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut delvsq = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut dirwts = vec_of_size((TS * TS) as usize, vec_of_size(2, 0.0_f32));
    let mut vcd = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut hcd = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut vcdalt = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut hcdalt = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut vcdsq = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut hcdsq = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut cddiffsq = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut hvwt = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut Dgrb = vec_of_size((TS * TS) as usize, vec_of_size(2, 0.0_f32));
    let mut delp = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut delm = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut rbint = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut Dgrbh2 = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut Dgrbv2 = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut dgintv = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut dginth = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut Dgrbp1 = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut Dgrbm1 = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut Dgrbpsq1 = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut Dgrbmsq1 = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut cfa = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut pmwt = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut rbp = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut rbm = vec_of_size((TS * TS) as usize, 0.0_f32);

    let mut nyquist = vec_of_size((TS * TS) as usize, 0);

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

    let eps: f32 = 1e-5;
    let epssq: f32 = 1e-10;

    let arthresh: f32 = 0.75;
    let nyqthresh: f32 = 0.5;
    let pmthresh: f32 = 0.25;
    let lbd: f32 = 1.0;
    let ubd: f32 = 1.0;

    let gaussodd: [f32; 4] = [
        0.14659727707323927,
        0.103592713382435,
        0.0732036125103057,
        0.0365543548389495,
    ];

    let gaussgrad: [f32; 6] = [
        0.07384411893421103,
        0.06207511968171489,
        0.0521818194747806,
        0.03687419286733595,
        0.03099732204057846,
        0.018413194161458882,
    ];

    let gauss1: [f32; 3] = [0.3376688223162362, 0.12171198028231786, 0.04387081413862306];

    let gausseven: [f32; 2] = [0.13719494435797422, 0.05640252782101291];

    let gquinc: [f32; 4] = [0.169917, 0.108947, 0.069855, 0.0287182];

    // Start Parallizable:
    // In other words, this is where Emil enables OpenMP parallel if LIBRAW_USE_OPENMP is 1

    //let mut top = 0;
    //let mut left = 0;

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
                    let mut c: i32 = 0;

                    // pointer counters within the tile
                    let mut indx = 0;
                    let mut indx1 = 0;

                    // direction counter for nbrs[]
                    let mut dir = 0;

                    // dummy indices
                    let mut i = 0;
                    let mut j = 0;

                    // +1 or -1
                    let mut sgn: f32 = 0.0;

                    //color ratios in up/down/left/right directions
                    let mut cru: f32 = 0.0;
                    let mut crd: f32 = 0.0;
                    let mut crl: f32 = 0.0;
                    let mut crr: f32 = 0.0;

                    //adaptive weights for vertical/horizontal/plus/minus directions
                    let mut vwt: f32 = 0.0;
                    let mut hwt: f32 = 0.0;
                    let mut pwt: f32 = 0.0;
                    let mut mwt: f32 = 0.0;

                    //vertical and horizontal G interpolations
                    let mut Gintv: f32 = 0.0;
                    let mut Ginth: f32 = 0.0;

                    //G interpolated in vert/hor directions using adaptive ratios
                    let mut guar: f32 = 0.0;
                    let mut gdar: f32 = 0.0;
                    let mut glar: f32 = 0.0;
                    let mut grar: f32 = 0.0;

                    //G interpolated in vert/hor directions using Hamilton-Adams method
                    let mut guha: f32 = 0.0;
                    let mut gdha: f32 = 0.0;
                    let mut glha: f32 = 0.0;
                    let mut grha: f32 = 0.0;

                    //interpolated G from fusing left/right or up/down
                    let mut Ginthar: f32 = 0.0;
                    let mut Ginthha: f32 = 0.0;
                    let mut Gintvar: f32 = 0.0;
                    let mut Gintvha: f32 = 0.0;

                    //color difference (G-R or G-B) variance in up/down/left/right directions
                    let mut Dgrbvvaru: f32 = 0.0;
                    let mut Dgrbvvard: f32 = 0.0;
                    let mut Dgrbhvarl: f32 = 0.0;
                    let mut Dgrbhvarr: f32 = 0.0;

                    //gradients in various directions
                    let mut gradp: f32 = 0.0;
                    let mut gradm: f32 = 0.0;
                    let mut gradv: f32 = 0.0;
                    let mut gradh: f32 = 0.0;
                    let mut gradpm: f32 = 0.0;
                    let mut gradhv: f32 = 0.0;

                    //color difference variances in vertical and horizontal directions
                    let mut vcdvar: f32 = 0.0;
                    let mut hcdvar: f32 = 0.0;
                    let mut vcdvar1: f32 = 0.0;
                    let mut hcdvar1: f32 = 0.0;
                    let mut hcdltvar: f32 = 0.0;
                    let mut vcdaltvar: f32 = 0.0;
                    let mut hcdaltvar: f32 = 0.0;

                    //adaptive interpolation weight using variance of color differences
                    let mut varwt: f32 = 0.0;

                    //adaptive interpolation weight using difference of left-right and up-down G interpolations
                    let mut diffwt: f32 = 0.0;

                    //alternative adaptive weight for combining horizontal/vertical interpolations
                    let mut hvwtalt: f32 = 0.0;

                    //temporary variables for combining interpolation weights at R and B sites
                    let mut vo: f32 = 0.0;
                    let mut ve: f32 = 0.0;

                    //interpolation of G in four directions
                    let mut gu: f32 = 0.0;
                    let mut gd: f32 = 0.0;
                    let mut gl: f32 = 0.0;
                    let mut gr: f32 = 0.0;

                    //variance of G in vertical/horizontal directions
                    let mut gvarh: f32 = 0.0;
                    let mut gvarv: f32 = 0.0;

                    //Nyquist texture test
                    let mut nyqtest: f32 = 0.0;

                    //accumulators for Nyquist texture interpolation
                    let mut sumh: f32 = 0.0;
                    let mut sumv: f32 = 0.0;
                    let mut sumsqh: f32 = 0.0;
                    let mut sumsqv: f32 = 0.0;
                    let mut areawt: f32 = 0.0;

                    //color ratios in diagonal directions
                    let mut crse: f32 = 0.0;
                    let mut crnw: f32 = 0.0;
                    let mut crne: f32 = 0.0;
                    let mut crsw: f32 = 0.0;

                    //color differences in diagonal directions
                    let mut rbse: f32 = 0.0;
                    let mut rbnw: f32 = 0.0;
                    let mut rbne: f32 = 0.0;
                    let mut rbsw: f32 = 0.0;

                    //adaptive weights for combining diagonal interpolations
                    let mut wtse: f32 = 0.0;
                    let mut wtnw: f32 = 0.0;
                    let mut wtsw: f32 = 0.0;
                    let mut wtne: f32 = 0.0;

                    //alternate weight for combining diagonal interpolations
                    let mut pmwtalt: f32 = 0.0;

                    //variance of R-B in plus/minus directions
                    let mut rbvarp: f32 = 0.0;
                    let mut rbvarm: f32 = 0.0;

                    // rgb from input CFA data
                    // rgb values should be floating point number between 0 and 1
                    // after white balance multipliers are applied
                    // a 16 pixel border is added to each side of the image

                    // bookkeeping for borders
                    rrmin = if top < winy { 16 } else { 0 };

                    ccmin = if left < winx { 16 } else { 0 };

                    rrmax = if bottom > (winy + height) {
                        winy + height - top
                    } else {
                        rr1
                    };

                    ccmax = if right > (winx + width) {
                        winx + width - left
                    } else {
                        cc1
                    };

                    for rr in (rrmin..rrmax) {
                        row = rr + top;
                        cc = ccmin;
                        while cc < ccmax {
                            col = cc + left;
                            c = FC!(rr, cc);

                            indx1 = rr * TS + cc;
                            indx = row * width + col;
                            rgb[indx1][c] = image[indx][c] / 65535.0_f32;
                            cfa[indx1] = rgb[indx1][c];
                            cc += 1;
                        }
                    }

                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
                    //fill borders
                    if rrmax > 0 {
                        for rr in (0..16) {
                            for cc in (ccmin..ccmax) {
                                c = FC!(rr, cc);
                                rgb[rr * TS + cc][c] = rgb[(32 - rr) * TS + cc][c];
                                cfa[rr * TS + cc] = rgb[rr * TS + cc][c];
                            }
                        }
                    }
                    if rrmax < rr1 {
                        for rr in (0..16) {
                            for cc in (ccmin..ccmax) {
                                c = FC!(rr, cc);
                                rgb[(rrmax + rr) * TS + cc][c] =
                                    (image[(height - rr - 2) * width + left + cc][c]) / 65535.0_f32;
                                cfa[(rrmax + rr) * TS + cc] = rgb[(rrmax + rr) * TS + cc][c];
                            }
                        }
                    }
                    if ccmin > 0 {
                        for rr in (rrmin..rrmax) {
                            for cc in (0..16) {
                                c = FC!(rr, cc);
                                rgb[rr * TS + cc][c] = rgb[rr * TS + 32 - cc][c];
                                cfa[rr * TS + cc] = rgb[rr * TS + cc][c];
                            }
                        }
                    }
                    if ccmax < cc1 {
                        for rr in (rrmin..rrmax) {
                            for cc in (0..16) {
                                c = FC!(rr, cc);
                                rgb[rr * TS + ccmax + cc][c] =
                                    (image[(top + rr) * width + (width - cc - 2)][c]) / 65535.0_f32;
                                cfa[rr * TS + ccmax + cc] = rgb[rr * TS + ccmax + cc][c];
                            }
                        }
                    }

                    //also, fill the image corners
                    if rrmin > 0 && ccmin > 0 {
                        for rr in (0..16) {
                            for cc in (0..16) {
                                c = FC!(rr, cc);
                                rgb[(rr) * TS + cc][c] = (rgb[(32 - rr) * TS + (32 - cc)][c]);
                                cfa[(rr) * TS + cc] = rgb[(rr) * TS + cc][c];
                            }
                        }
                    }
                    if rrmax < rr1 && ccmax < cc1 {
                        for rr in (0..16) {
                            c = FC!(rr, cc);
                            rgb[(rrmax + rr) * TS + ccmax + cc][c] = (image
                                [(height - rr - 2) * width + (width - cc - 2)][c])
                                / 65535.0_f32;
                            cfa[(rrmax + rr) * TS + ccmax + cc] =
                                rgb[(rrmax + rr) * TS + ccmax + cc][c];
                        }
                    }
                    if rrmin > 0 && ccmax < cc1 {
                        for rr in (0..16) {
                            for cc in (0..16) {
                                c = FC!(rr, cc);
                                rgb[(rr) * TS + ccmax + cc][c] =
                                    (image[(32 - rr) * width + (width - cc - 2)][c]) / 65535.0_f32;
                                cfa[(rr) * TS + ccmax + cc] = rgb[(rr) * TS + ccmax + cc][c];
                            }
                        }
                    }
                    if rrmax < rr1 && ccmin > 0 {
                        for rr in (0..16) {
                            for cc in (0..16) {
                                c = FC!(rr, cc);
                                rgb[(rrmax + rr) * TS + cc][c] =
                                    (image[(height - rr - 2) * width + (32 - cc)][c]) / 65535.0_f32;
                                cfa[(rrmax + rr) * TS + cc] = rgb[(rrmax + rr) * TS + cc][c];
                            }
                        }
                    }

                    //end of border fill
                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

                    for rr in (1..rr1 - 1) {
                        cc = 1;
                        indx = rr * TS + cc;
                        while cc < cc1 - 1 {
                            // Pick up on line 436
                            delh[indx] = (cfa[indx + 1] as f32 - cfa[indx - 1] as f32).abs();
                            delv[indx] = (cfa[indx + v1] as f32 - cfa[indx - v1] as f32).abs();
                            delhsq[indx] = SQR!(delh[indx]);
                            delvsq[indx] = SQR!(delv[indx]);
                            delp[indx] = (cfa[indx + p1] as f32 - cfa[indx - p1] as f32).abs();
                            delm[indx] = (cfa[indx + m1] as f32 - cfa[indx - m1] as f32).abs();
                            cc += 1;
                            indx += 1;
                        }
                    }

                    for rr in (2..rr1 - 2) {
                        cc = 2;
                        indx = rr * TS + cc;

                        while cc < cc1 - 2 {
                            dirwts[indx][0] = eps + delv[indx + v1] + delv[indx - v1] + delv[indx];
                            dirwts[indx][1] = eps + delh[indx + 1] + delh[indx - 1] + delh[indx];

                            if FC!(rr, cc) & 1 >= 1 {
                                //for later use in diagonal interpolation
                                Dgrbpsq1[indx] = (SQR!(cfa[indx] - cfa[indx - p1])
                                    + SQR!(cfa[indx] - cfa[indx + p1]));
                                Dgrbmsq1[indx] = (SQR!(cfa[indx] - cfa[indx - m1])
                                    + SQR!(cfa[indx] - cfa[indx + m1]));
                            }
                            cc += 1;
                            indx += 1;
                        }
                    }
                    // end of tile initialization
                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

                    //interpolate vertical and horizontal color differences
                    for rr in (4..(rr1 - 4)) {
                        cc = 4;
                        indx = rr * TS + cc;
                        while cc < cc1 - 4 {
                            c = FC!(rr, cc);

                            //initialization of nyquist test
                            nyquist[indx] = 0;

                            //preparation for diag interp
                            rbint[indx] = 0.0;

                            //color ratios in each cardinal direction
                            cru = cfa[indx - v1] * (dirwts[indx - v2][0] + dirwts[indx][0])
                                / (dirwts[indx - v2][0] * (eps + cfa[indx])
                                    + dirwts[indx][0] * (eps + cfa[indx - v2]));
                            crd = cfa[indx + v1] * (dirwts[indx + v2][0] + dirwts[indx][0])
                                / (dirwts[indx + v2][0] * (eps + cfa[indx])
                                    + dirwts[indx][0] * (eps + cfa[indx + v2]));
                            crl = cfa[indx - 1] * (dirwts[indx - 2][1] + dirwts[indx][1])
                                / (dirwts[indx - 2][1] * (eps + cfa[indx])
                                    + dirwts[indx][1] * (eps + cfa[indx - 2]));
                            crr = cfa[indx + 1] * (dirwts[indx + 2][1] + dirwts[indx][1])
                                / (dirwts[indx + 2][1] * (eps + cfa[indx])
                                    + dirwts[indx][1] * (eps + cfa[indx + 2]));

                            guha =
                                min!(clip_pt, cfa[indx - v1]) + 0.5 * (cfa[indx] - cfa[indx - v2]);
                            gdha =
                                min!(clip_pt, cfa[indx + v1]) + 0.5 * (cfa[indx] - cfa[indx + v2]);
                            glha = min!(clip_pt, cfa[indx - 1]) + 0.5 * (cfa[indx] - cfa[indx - 2]);
                            grha = min!(clip_pt, cfa[indx + 1]) + 0.5 * (cfa[indx] - cfa[indx + 2]);

                            guar = if fabs!(1.0 - cru) < arthresh {
                                cfa[indx] * cru
                            } else {
                                guha
                            };

                            gdar = if fabs!(1.0 - crd) < arthresh {
                                cfa[indx] * crd
                            } else {
                                gdha
                            };

                            glar = if fabs!(1.0 - crl) < arthresh {
                                cfa[indx] * crl
                            } else {
                                glha
                            };

                            grar = if fabs!(1.0 - crr) < arthresh {
                                cfa[indx] * crr
                            } else {
                                grha
                            };

                            hwt = dirwts[indx - 1][1] / (dirwts[indx - 1][1] + dirwts[indx + 1][1]);
                            vwt = dirwts[indx - v1][0]
                                / (dirwts[indx + v1][0] + dirwts[indx - v1][0]);

                            //interpolated G via adaptive weights of cardinal evaluations
                            Gintvar = vwt * gdar + (1.0 - vwt) * guar;
                            Ginthar = hwt * grar + (1.0 - hwt) * glar;
                            Gintvha = vwt * gdha + (1.0 - vwt) * guha;
                            Ginthha = hwt * grha + (1.0 - hwt) * glha;

                            //interpolated color differences
                            vcd[indx] = sgn * (Gintvar - cfa[indx]);
                            hcd[indx] = sgn * (Ginthar - cfa[indx]);
                            vcdalt[indx] = sgn * (Gintvha - cfa[indx]);
                            hcdalt[indx] = sgn * (Ginthha - cfa[indx]);

                            if cfa[indx] > 0.8 * clip_pt
                                || Gintvha > 0.8 * clip_pt
                                || Ginthha > 0.8 * clip_pt
                            {
                                guar = guha;
                                gdar = gdha;
                                glar = glha;
                                grar = grha;
                                vcd[indx] = vcdalt[indx];
                                hcd[indx] = hcdalt[indx];
                            }

                            dgintv[indx] = min!(SQR!(guha - gdha), SQR!(guar - gdar));
                            dginth[indx] = min!(SQR!(glha - grha), SQR!(glar - grar));

                            cc += 1;
                            indx += 1;
                        }
                    }

                    for rr in (4..(rr1 - 4)) {
                        cc = 4;
                        indx = rr * TS + cc;
                        while cc < cc1 - 4 {
                            hcdvar = 3.0
                                * (SQR!(hcd[indx - 2]) + SQR!(hcd[indx]) + SQR!(hcd[indx + 2]))
                                - SQR!(hcd[indx - 2] + hcd[indx] + hcd[indx + 2]);
                            hcdaltvar = 3.0
                                * (SQR!(hcdalt[indx - 2])
                                    + SQR!(hcdalt[indx])
                                    + SQR!(hcdalt[indx + 2]))
                                - SQR!(hcdalt[indx - 2] + hcdalt[indx] + hcdalt[indx + 2]);
                            vcdvar = 3.0
                                * (SQR!(vcd[indx - v2]) + SQR!(vcd[indx]) + SQR!(vcd[indx + v2]))
                                - SQR!(vcd[indx - v2] + vcd[indx] + vcd[indx + v2]);
                            vcdaltvar = 3.0
                                * (SQR!(vcdalt[indx - v2])
                                    + SQR!(vcdalt[indx])
                                    + SQR!(vcdalt[indx + v2]))
                                - SQR!(vcdalt[indx - v2] + vcdalt[indx] + vcdalt[indx + v2]);

                            //choose the smallest variance; this yields a smoother interpolation
                            if hcdaltvar < hcdvar {
                                hcd[indx] = hcdalt[indx];
                            }

                            if vcdaltvar < vcdvar {
                                vcd[indx] = vcdalt[indx]
                            }
                            if c & 1 >= 1 {
                                Ginth = -hcd[indx] + cfa[indx]; //R or B
                                Gintv = -vcd[indx] + cfa[indx]; //B or R

                                if hcd[indx] > 0.0 {
                                    if 3.0 * hcd[indx] > (Ginth + cfa[indx]) {
                                        hcd[indx] =
                                            -ULIM!(Ginth, cfa[indx - 1], cfa[indx + 1]) + cfa[indx];
                                    } else {
                                        hwt = 1.0 - 3.0 * hcd[indx] / (eps + Ginth + cfa[indx]);
                                        hcd[indx] = hwt * hcd[indx]
                                            + (1.0 - hwt)
                                                * (-ULIM!(Ginth, cfa[indx - 1], cfa[indx + 1])
                                                    + cfa[indx]);
                                    }
                                }

                                if vcd[indx] > 0.0 {
                                    if 3.0 * vcd[indx] > (Gintv + cfa[indx]) {
                                        vcd[indx] = -ULIM!(Gintv, cfa[indx - v1], cfa[indx + v1])
                                            + cfa[indx];
                                    } else {
                                        vwt = 1.0 - 3.0 * vcd[indx] / (eps + Gintv + cfa[indx]);
                                        vcd[indx] = vwt * vcd[indx]
                                            + (1.0 - vwt)
                                                * (-ULIM!(Gintv, cfa[indx - v1], cfa[indx + v1])
                                                    + cfa[indx]);
                                    }
                                }

                                if Ginth > clip_pt {
                                    hcd[indx] =
                                        -ULIM!(Ginth, cfa[indx - 1], cfa[indx + 1]) + cfa[indx];
                                } //for RT implementation
                                if Gintv > clip_pt {
                                    vcd[indx] =
                                        -ULIM!(Gintv, cfa[indx - v1], cfa[indx + v1]) + cfa[indx];
                                }
                            } else {
                                // R or B site
                                Ginth = hcd[indx] + cfa[indx]; //interpolated G
                                Gintv = vcd[indx] + cfa[indx];

                                if hcd[indx] < 0.0 {
                                    if 3.0 * hcd[indx] < -(Ginth + cfa[indx]) {
                                        hcd[indx] =
                                            ULIM!(Ginth, cfa[indx - 1], cfa[indx + 1]) - cfa[indx];
                                    } else {
                                        hwt = 1.0 + 3.0 * hcd[indx] / (eps + Ginth + cfa[indx]);
                                        hcd[indx] = hwt * hcd[indx]
                                            + (1.0 - hwt)
                                                * (ULIM!(Ginth, cfa[indx - 1], cfa[indx + 1])
                                                    - cfa[indx]);
                                    }
                                }
                                if vcd[indx] < 0.0 {
                                    if 3.0 * vcd[indx] < -(Gintv + cfa[indx]) {
                                        vcd[indx] = ULIM!(Gintv, cfa[indx - v1], cfa[indx + v1])
                                            - cfa[indx];
                                    } else {
                                        vwt = 1.0 + 3.0 * vcd[indx] / (eps + Gintv + cfa[indx]);
                                        vcd[indx] = vwt * vcd[indx]
                                            + (1.0 - vwt)
                                                * (ULIM!(Gintv, cfa[indx - v1], cfa[indx + v1])
                                                    - cfa[indx]);
                                    }
                                }
                                if Ginth > clip_pt {
                                    hcd[indx] =
                                        ULIM!(Ginth, cfa[indx - 1], cfa[indx + 1]) - cfa[indx];
                                } //for RT implementation
                                if Gintv > clip_pt {
                                    vcd[indx] =
                                        ULIM!(Gintv, cfa[indx - v1], cfa[indx + v1]) - cfa[indx];
                                }
                            }

                            vcdsq[indx] = SQR!(vcd[indx]);
                            hcdsq[indx] = SQR!(hcd[indx]);
                            cddiffsq[indx] = SQR!(vcd[indx] - hcd[indx]);

                            cc += 1;
                            indx += 1;
                        }
                    }

                    for rr in (6..(rr1 - 6)) {
                        cc = 6 + (FC!(rr, 2) & 1);
                        indx = rr * TS + cc;

                        while cc < cc1 - 6 {
                            Dgrbvvaru = 4.0
                                * (vcdsq[indx]
                                    + vcdsq[indx - v1]
                                    + vcdsq[indx - v2]
                                    + vcdsq[indx - v3])
                                - SQR!(
                                    vcd[indx] + vcd[indx - v1] + vcd[indx - v2] + vcd[indx - v3]
                                );
                            Dgrbvvard = 4.0
                                * (vcdsq[indx]
                                    + vcdsq[indx + v1]
                                    + vcdsq[indx + v2]
                                    + vcdsq[indx + v3])
                                - SQR!(
                                    vcd[indx] + vcd[indx + v1] + vcd[indx + v2] + vcd[indx + v3]
                                );
                            Dgrbhvarl = 4.0
                                * (hcdsq[indx]
                                    + hcdsq[indx - 1]
                                    + hcdsq[indx - 2]
                                    + hcdsq[indx - 3])
                                - SQR!(hcd[indx] + hcd[indx - 1] + hcd[indx - 2] + hcd[indx - 3]);
                            Dgrbhvarr = 4.0
                                * (hcdsq[indx]
                                    + hcdsq[indx + 1]
                                    + hcdsq[indx + 2]
                                    + hcdsq[indx + 3])
                                - SQR!(hcd[indx] + hcd[indx + 1] + hcd[indx + 2] + hcd[indx + 3]);

                            hwt = dirwts[indx - 1][1] / (dirwts[indx - 1][1] + dirwts[indx + 1][1]);
                            vwt = dirwts[indx - v1][0]
                                / (dirwts[indx + v1][0] + dirwts[indx - v1][0]);

                            vcdvar = epssq + vwt * Dgrbvvard + (1.0 - vwt) * Dgrbvvaru;
                            hcdvar = epssq + hwt * Dgrbhvarr + (1.0 - hwt) * Dgrbhvarl;

                            Dgrbvvaru = (dgintv[indx]) + (dgintv[indx - v1]) + (dgintv[indx - v2]);
                            Dgrbvvard = (dgintv[indx]) + (dgintv[indx + v1]) + (dgintv[indx + v2]);
                            Dgrbhvarl = (dginth[indx]) + (dginth[indx - 1]) + (dginth[indx - 2]);
                            Dgrbhvarr = (dginth[indx]) + (dginth[indx + 1]) + (dginth[indx + 2]);

                            vcdvar1 = epssq + vwt * Dgrbvvard + (1.0 - vwt) * Dgrbvvaru;
                            hcdvar1 = epssq + hwt * Dgrbhvarr + (1.0 - hwt) * Dgrbhvarl;

                            //determine adaptive weights for G interpolation
                            varwt = hcdvar / (vcdvar + hcdvar);
                            diffwt = hcdvar1 / (vcdvar1 + hcdvar1);

                            //if both agree on interpolation direction, choose the one with strongest directional discrimination;
                            //otherwise, choose the u/d and l/r difference fluctuation weights
                            hvwt[indx] = if (0.5 - varwt) * (0.5 - diffwt) > 0.0
                                && fabs!(0.5 - diffwt) < fabs!(0.5 - varwt)
                            {
                                varwt
                            } else {
                                diffwt
                            };

                            cc += 2;
                            indx += 2;
                        }
                    }

                    for rr in (6..(rr1 - 6)) {
                        cc = 6 + (FC!(rr, 2) & 1);
                        indx = rr * TS + cc;
                        while cc < cc1 - 6 {
                            //nyquist texture test: ask if difference of vcd compared to hcd is larger or smaller than RGGB gradients
                            nyqtest = (gaussodd[0] * cddiffsq[indx]
                                + gaussodd[1]
                                    * (cddiffsq[indx - m1]
                                        + cddiffsq[indx + p1]
                                        + cddiffsq[indx - p1]
                                        + cddiffsq[indx + m1])
                                + gaussodd[2]
                                    * (cddiffsq[indx - v2]
                                        + cddiffsq[indx - 2]
                                        + cddiffsq[indx + 2]
                                        + cddiffsq[indx + v2])
                                + gaussodd[3]
                                    * (cddiffsq[indx - m2]
                                        + cddiffsq[indx + p2]
                                        + cddiffsq[indx - p2]
                                        + cddiffsq[indx + m2]));

                            nyqtest -= nyqthresh
                                * (gaussgrad[0] * (delhsq[indx] + delvsq[indx])
                                    + gaussgrad[1]
                                        * (delhsq[indx - v1]
                                            + delvsq[indx - v1]
                                            + delhsq[indx + 1]
                                            + delvsq[indx + 1]
                                            + delhsq[indx - 1]
                                            + delvsq[indx - 1]
                                            + delhsq[indx + v1]
                                            + delvsq[indx + v1])
                                    + gaussgrad[2]
                                        * (delhsq[indx - m1]
                                            + delvsq[indx - m1]
                                            + delhsq[indx + p1]
                                            + delvsq[indx + p1]
                                            + delhsq[indx - p1]
                                            + delvsq[indx - p1]
                                            + delhsq[indx + m1]
                                            + delvsq[indx + m1])
                                    + gaussgrad[3]
                                        * (delhsq[indx - v2]
                                            + delvsq[indx - v2]
                                            + delhsq[indx - 2]
                                            + delvsq[indx - 2]
                                            + delhsq[indx + 2]
                                            + delvsq[indx + 2]
                                            + delhsq[indx + v2]
                                            + delvsq[indx + v2])
                                    + gaussgrad[4]
                                        * (delhsq[indx - 2 * TS - 1]
                                            + delvsq[indx - 2 * TS - 1]
                                            + delhsq[indx - 2 * TS + 1]
                                            + delvsq[indx - 2 * TS + 1]
                                            + delhsq[indx - TS - 2]
                                            + delvsq[indx - TS - 2]
                                            + delhsq[indx - TS + 2]
                                            + delvsq[indx - TS + 2]
                                            + delhsq[indx + TS - 2]
                                            + delvsq[indx + TS - 2]
                                            + delhsq[indx + TS + 2]
                                            + delvsq[indx - TS + 2]
                                            + delhsq[indx + 2 * TS - 1]
                                            + delvsq[indx + 2 * TS - 1]
                                            + delhsq[indx + 2 * TS + 1]
                                            + delvsq[indx + 2 * TS + 1])
                                    + gaussgrad[5]
                                        * (delhsq[indx - m2]
                                            + delvsq[indx - m2]
                                            + delhsq[indx + p2]
                                            + delvsq[indx + p2]
                                            + delhsq[indx - p2]
                                            + delvsq[indx - p2]
                                            + delhsq[indx + m2]
                                            + delvsq[indx + m2]));
                            if nyqtest > 0.0 {
                                nyquist[indx] = 1;
                            }
                            cc += 2;
                            indx += 2;
                        }
                    }

                    for rr in (8..(rr1 - 8)) {
                        cc = 8 + (FC!(rr, 2) & 1);
                        indx = rr * TS + cc;
                        while cc < cc1 - 8 {
                            areawt = (nyquist[indx - v2]
                                + nyquist[indx - m1]
                                + nyquist[indx + p1]
                                + nyquist[indx - 2]
                                + nyquist[indx]
                                + nyquist[indx + 2]
                                + nyquist[indx - p1]
                                + nyquist[indx + m1]
                                + nyquist[indx + v2]) as f32;
                            //if most of your neighbors are named Nyquist, it's likely that you're one too
                            if areawt > 4.0 {
                                nyquist[indx] = 1;
                            }
                            //or not
                            if areawt < 4.0 {
                                nyquist[indx] = 0;
                            }
                            cc += 2;
                            indx += 2;
                        }
                    }
                    // end of Nyquist test
                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
                    // in areas of Nyquist texture, do area interpolation

                    for rr in (8..rr1 - 8) {
                        cc = 8 + (FC!(rr, 2) & 1);
                        indx = rr * TS + cc;
                        while cc < cc1 - 8 {
                            if nyquist[indx] >= 1 {
                                // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
                                // area interpolation
                                sumh = 0.0;
                                sumv = 0.0;
                                sumsqh = 0.0;
                                sumsqv = 0.0;
                                areawt = 0.0;
                                for i in (-6..7).step_by(2) {
                                    for j in (-6..7).step_by(2) {
                                        indx1 = (rr + i) * TS + cc + j;
                                        if nyquist[indx1] >= 1 {
                                            sumh += cfa[indx1]
                                                - 0.5 * (cfa[indx1 - 1] + cfa[indx1 + 1]);
                                            sumv += cfa[indx1]
                                                - 0.5 * (cfa[indx1 - v1] + cfa[indx1 + v1]);
                                            sumsqh += 0.5
                                                * (SQR!(cfa[indx1] - cfa[indx1 - 1])
                                                    + SQR!(cfa[indx1] - cfa[indx1 + 1]));
                                            sumsqv += 0.5
                                                * (SQR!(cfa[indx1] - cfa[indx1 - v1])
                                                    + SQR!(cfa[indx1] - cfa[indx1 + v1]));
                                            areawt += 1.0;
                                        }
                                    }
                                }
                                //horizontal and vertical color differences, and adaptive weight
                                hcdvar = epssq + max!(0.0, areawt * sumsqh - sumh * sumh);
                                vcdvar = epssq + max!(0.0, areawt * sumsqv - sumv * sumv);
                                hvwt[indx] = hcdvar / (vcdvar + hcdvar);

                                // end of area interpolation
                                // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
                            }
                            cc += 2;
                            indx += 2;
                        }
                    }
                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
                    //populate G at R/B sites
                    for rr in (8..rr1 - 8) {
                        cc = 8 + (FC!(rr, 2) & 1);
                        indx = rr * TS + cc;
                        while cc < cc1 - 8 {
                            hvwtalt = 0.25
                                * (hvwt[indx - m1]
                                    + hvwt[indx + p1]
                                    + hvwt[indx - p1]
                                    + hvwt[indx + m1]);
                            vo = fabs!(0.5 - hvwt[indx]);
                            ve = fabs!(0.5 - hvwtalt);

                            if vo < ve {
                                hvwt[indx] = hvwtalt;
                            } //a better result was obtained from the neighbors

                            Dgrb[indx][0] =
                                (hcd[indx] * (1.0 - hvwt[indx]) + vcd[indx] * hvwt[indx]); //evaluate color differences
                            rgb[indx][1] = cfa[indx] + Dgrb[indx][0]; //evaluate G (finally!)

                            if nyquist[indx] >= 1 {
                                Dgrbh2[indx] = SQR!(
                                    rgb[indx][1] - 0.5_f32 * (rgb[indx - 1][1] + rgb[indx + 1][1])
                                );
                                Dgrbv2[indx] = SQR!(
                                    rgb[indx][1]
                                        - 0.5_f32 * (rgb[indx - v1][1] + rgb[indx + v1][1])
                                );
                            } else {
                                Dgrbh2[indx] = 0.0;
                                Dgrbv2[indx] = 0.0;
                            }

                            cc += 2;
                            indx += 2;
                        }
                    }

                    //end of standard interpolation
                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
                    // refine Nyquist areas using G curvatures
                    for rr in (8..rr1 - 8) {
                        cc = 8 + (FC!(rr, 2) & 1);
                        indx = rr * TS + cc;
                        while cc < cc1 - 8 {
                            if nyquist[indx] > 1 {
                                //local averages (over Nyquist pixels only) of G curvature squared
                                gvarh = epssq
                                    + (gquinc[0] * Dgrbh2[indx]
                                        + gquinc[1]
                                            * (Dgrbh2[indx - m1]
                                                + Dgrbh2[indx + p1]
                                                + Dgrbh2[indx - p1]
                                                + Dgrbh2[indx + m1])
                                        + gquinc[2]
                                            * (Dgrbh2[indx - v2]
                                                + Dgrbh2[indx - 2]
                                                + Dgrbh2[indx + 2]
                                                + Dgrbh2[indx + v2])
                                        + gquinc[3]
                                            * (Dgrbh2[indx - m2]
                                                + Dgrbh2[indx + p2]
                                                + Dgrbh2[indx - p2]
                                                + Dgrbh2[indx + m2]));
                                gvarv = epssq
                                    + (gquinc[0] * Dgrbv2[indx]
                                        + gquinc[1]
                                            * (Dgrbv2[indx - m1]
                                                + Dgrbv2[indx + p1]
                                                + Dgrbv2[indx - p1]
                                                + Dgrbv2[indx + m1])
                                        + gquinc[2]
                                            * (Dgrbv2[indx - v2]
                                                + Dgrbv2[indx - 2]
                                                + Dgrbv2[indx + 2]
                                                + Dgrbv2[indx + v2])
                                        + gquinc[3]
                                            * (Dgrbv2[indx - m2]
                                                + Dgrbv2[indx + p2]
                                                + Dgrbv2[indx - p2]
                                                + Dgrbv2[indx + m2]));
                                //use the results as weights for refined G interpolation
                                Dgrb[indx][0] =
                                    (hcd[indx] * gvarv + vcd[indx] * gvarh) / (gvarv + gvarh);
                                rgb[indx][1] = cfa[indx] + Dgrb[indx][0];
                            }
                            cc += 2;
                            indx += 2;
                        }
                    }
                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
                    // diagonal interpolation correction
                    for rr in (8..rr1 - 8) {
                        cc = 8 + (FC!(rr, 2) & 1);
                        indx = rr * TS + cc;
                        while cc < cc1 - 8 {
                            rbvarp = epssq
                                + (gausseven[0]
                                    * (Dgrbpsq1[indx - v1]
                                        + Dgrbpsq1[indx - 1]
                                        + Dgrbpsq1[indx + 1]
                                        + Dgrbpsq1[indx + v1])
                                    + gausseven[1]
                                        * (Dgrbpsq1[indx - v2 - 1]
                                            + Dgrbpsq1[indx - v2 + 1]
                                            + Dgrbpsq1[indx - 2 - v1]
                                            + Dgrbpsq1[indx + 2 - v1]
                                            + Dgrbpsq1[indx - 2 + v1]
                                            + Dgrbpsq1[indx + 2 + v1]
                                            + Dgrbpsq1[indx + v2 - 1]
                                            + Dgrbpsq1[indx + v2 + 1]));
                            rbvarm = epssq
                                + (gausseven[0]
                                    * (Dgrbmsq1[indx - v1]
                                        + Dgrbmsq1[indx - 1]
                                        + Dgrbmsq1[indx + 1]
                                        + Dgrbmsq1[indx + v1])
                                    + gausseven[1]
                                        * (Dgrbmsq1[indx - v2 - 1]
                                            + Dgrbmsq1[indx - v2 + 1]
                                            + Dgrbmsq1[indx - 2 - v1]
                                            + Dgrbmsq1[indx + 2 - v1]
                                            + Dgrbmsq1[indx - 2 + v1]
                                            + Dgrbmsq1[indx + 2 + v1]
                                            + Dgrbmsq1[indx + v2 - 1]
                                            + Dgrbmsq1[indx + v2 + 1]));
                            // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

                            //diagonal color ratios
                            crse = 2.0 * (cfa[indx + m1]) / (eps + cfa[indx] + (cfa[indx + m2]));
                            crnw = 2.0 * (cfa[indx - m1]) / (eps + cfa[indx] + (cfa[indx - m2]));
                            crne = 2.0 * (cfa[indx + p1]) / (eps + cfa[indx] + (cfa[indx + p2]));
                            crsw = 2.0 * (cfa[indx - p1]) / (eps + cfa[indx] + (cfa[indx - p2]));

                            //assign B/R at R/B sites
                            rbse = if fabs!(1.0 - crse) < arthresh {
                                cfa[indx] * crse
                            }
                            //use this if more precise diag interp is necessary
                            else {
                                (cfa[indx + m1]) + 0.5 * (cfa[indx] - cfa[indx + m2])
                            };

                            rbnw = if fabs!(1.0 - crnw) < arthresh {
                                cfa[indx] * crnw
                            } else {
                                (cfa[indx - m1]) + 0.5 * (cfa[indx] - cfa[indx - m2])
                            };

                            rbne = if fabs!(1.0 - crne) < arthresh {
                                cfa[indx] * crne
                            } else {
                                (cfa[indx + p1]) + 0.5 * (cfa[indx] - cfa[indx + p2])
                            };

                            rbsw = if fabs!(1.0 - crsw) < arthresh {
                                cfa[indx] * crsw
                            } else {
                                (cfa[indx - p1]) + 0.5 * (cfa[indx] - cfa[indx - p2])
                            };

                            wtse = eps + delm[indx] + delm[indx + m1] + delm[indx + m2]; //same as for wtu,wtd,wtl,wtr
                            wtnw = eps + delm[indx] + delm[indx - m1] + delm[indx - m2];
                            wtne = eps + delp[indx] + delp[indx + p1] + delp[indx + p2];
                            wtsw = eps + delp[indx] + delp[indx - p1] + delp[indx - p2];

                            rbm[indx] = (wtse * rbnw + wtnw * rbse) / (wtse + wtnw);
                            rbp[indx] = (wtne * rbsw + wtsw * rbne) / (wtne + wtsw);

                            pmwt[indx] = rbvarm / (rbvarp + rbvarm);

                            // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
                            //bound the interpolation in regions of high saturation

                            if rbp[indx] < cfa[indx] {
                                rbp[indx] = if 2.0 * rbp[indx] < cfa[indx] {
                                    ULIM!(rbp[indx], cfa[indx - p1], cfa[indx + p1])
                                } else {
                                    pwt = 2.0 * (cfa[indx] - rbp[indx])
                                        / (eps + rbp[indx] + cfa[indx]);
                                    pwt * rbp[indx]
                                        + (1.0 - pwt)
                                            * ULIM!(rbp[indx], cfa[indx - p1], cfa[indx + p1])
                                };
                            }

                            if rbm[indx] < cfa[indx] {
                                rbm[indx] = if 2.0 * rbm[indx] < cfa[indx] {
                                    ULIM!(rbm[indx], cfa[indx - m1], cfa[indx + m1])
                                } else {
                                    mwt = 2.0 * (cfa[indx] - rbm[indx])
                                        / (eps + rbm[indx] + cfa[indx]);
                                    mwt * rbm[indx]
                                        + (1.0 - mwt)
                                            * ULIM!(rbm[indx], cfa[indx - m1], cfa[indx + m1])
                                }
                            }

                            if rbp[indx] > clip_pt {
                                rbp[indx] = ULIM!(rbp[indx], cfa[indx - p1], cfa[indx + p1]);
                            } //for RT implementation
                            if rbm[indx] > clip_pt {
                                rbm[indx] = ULIM!(rbm[indx], cfa[indx - m1], cfa[indx + m1]);
                            }

                            cc += 2;
                            indx += 2;
                        }
                    }

                    for rr in (10..rr1 - 10) {
                        cc = 10 + (FC!(rr, 2) & 1);
                        indx = rr * TS + cc;
                        while cc < cc1 - 10 {
                            pmwtalt = 0.25
                                * (pmwt[indx - m1]
                                    + pmwt[indx + p1]
                                    + pmwt[indx - p1]
                                    + pmwt[indx + m1]);
                            vo = fabs!(0.5 - pmwt[indx]);
                            ve = fabs!(0.5 - pmwtalt);

                            if vo < ve {
                                pmwt[indx] = pmwtalt;
                            } //a better result was obtained from the neighbors
                            rbint[indx] = 0.5
                                * (cfa[indx]
                                    + rbm[indx] * (1.0 - pmwt[indx])
                                    + rbp[indx] * pmwt[indx]); //this is R+B, interpolated

                            cc += 2;
                            indx += 2;
                        }
                    }

                    for rr in (12..rr1 - 12) {
                        cc = 12 + (FC!(rr, 2) & 1);
                        indx = rr * TS + cc;
                        while cc < cc1 - 12 {
                            // if fabs!(0.5 - pmwt[indx]) < fabs!(0.5 - hvwt[indx]) {
                            //     cc += 2;
                            //     indx += 2;
                            //     continue;
                            // }

                            //now interpolate G vertically/horizontally using R+B values
                            //unfortunately, since G interpolation cannot be done diagonally this may lead to color shifts
                            //color ratios for G interpolation

                            cru = cfa[indx - v1] * 2.0 / (eps + rbint[indx] + rbint[indx - v2]);
                            crd = cfa[indx + v1] * 2.0 / (eps + rbint[indx] + rbint[indx + v2]);
                            crl = cfa[indx - 1] * 2.0 / (eps + rbint[indx] + rbint[indx - 2]);
                            crr = cfa[indx + 1] * 2.0 / (eps + rbint[indx] + rbint[indx + 2]);

                            gu = if fabs!(1.0 - cru) < arthresh {
                                rbint[indx] * cru
                            } else {
                                cfa[indx - v1] + 0.5 * (rbint[indx] - rbint[indx - v2])
                            };
                            gd = if fabs!(1.0 - crd) < arthresh {
                                rbint[indx] * crd
                            } else {
                                cfa[indx + v1] + 0.5 * (rbint[indx] - rbint[indx + v2])
                            };
                            gl = if fabs!(1.0 - crl) < arthresh {
                                rbint[indx] * crl
                            } else {
                                cfa[indx - 1] + 0.5 * (rbint[indx] - rbint[indx - 2])
                            };
                            gr = if fabs!(1.0 - crr) < arthresh {
                                rbint[indx] * crr
                            } else {
                                cfa[indx + 1] + 0.5 * (rbint[indx] - rbint[indx + 2])
                            };

                            //interpolated G via adaptive weights of cardinal evaluations
                            Gintv = (dirwts[indx - v1][0] * gd + dirwts[indx + v1][0] * gu)
                                / (dirwts[indx + v1][0] + dirwts[indx - v1][0]);
                            Ginth = (dirwts[indx - 1][1] * gr + dirwts[indx + 1][1] * gl)
                                / (dirwts[indx - 1][1] + dirwts[indx + 1][1]);

                            // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
                            //bound the interpolation in regions of high saturation
                            if Gintv < rbint[indx] {
                                Gintv = if 2.0 * Gintv < rbint[indx] {
                                    ULIM!(Gintv, cfa[indx - v1], cfa[indx + v1])
                                } else {
                                    vwt = 2.0 * (rbint[indx] - Gintv) / (eps + Gintv + rbint[indx]);
                                    vwt * Gintv
                                        + (1.0 - vwt) * ULIM!(Gintv, cfa[indx - v1], cfa[indx + v1])
                                };
                            }
                            if Ginth < rbint[indx] {
                                Ginth = if 2.0 * Ginth < rbint[indx] {
                                    ULIM!(Ginth, cfa[indx - 1], cfa[indx + 1])
                                } else {
                                    hwt = 2.0 * (rbint[indx] - Ginth) / (eps + Ginth + rbint[indx]);
                                    hwt * Ginth
                                        + (1.0 - hwt) * ULIM!(Ginth, cfa[indx - 1], cfa[indx + 1])
                                }
                            }

                            if Ginth > clip_pt {
                                Ginth = ULIM!(Ginth, cfa[indx - 1], cfa[indx + 1]);
                            } //for RT implementation
                            if Gintv > clip_pt {
                                Gintv = ULIM!(Gintv, cfa[indx - v1], cfa[indx + v1]);
                            }

                            rgb[indx][1] = Ginth * (1.0 - hvwt[indx]) + Gintv * hvwt[indx];
                            Dgrb[indx][0] = rgb[indx][1] - cfa[indx];

                            cc += 2;
                            indx += 2;
                        }
                    }
                    //end of diagonal interpolation correction
                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

                    //fancy chrominance interpolation
                    //(ey,ex) is location of R site
                    for rr in (13 - ey..rr1 - 12).step_by(2) {
                        cc = 13 - ex;
                        indx = rr * TS + cc;
                        while cc < cc1 - 12 {
                            Dgrb[indx][1] = Dgrb[indx][0]; //split out G-B from G-R
                            Dgrb[indx][0] = 0.0_f32;
                            cc += 2;
                            indx += 2;
                        }
                    }

                    for rr in (12..rr1 - 12) {
                        cc = 12 + (FC!(rr, 2) & 1);
                        indx = rr * TS + cc;
                        c = 1 - FC!(rr, cc) / 2;
                        while cc < cc1 - 12 {
                            wtnw = 1.0
                                / (eps
                                    + fabs!(Dgrb[indx - m1][c] - Dgrb[indx + m1][c])
                                    + fabs!(Dgrb[indx - m1][c] - Dgrb[indx - m3][c])
                                    + fabs!(Dgrb[indx + m1][c] - Dgrb[indx - m3][c]));
                            wtne = 1.0
                                / (eps
                                    + fabs!(Dgrb[indx + p1][c] - Dgrb[indx - p1][c])
                                    + fabs!(Dgrb[indx + p1][c] - Dgrb[indx + p3][c])
                                    + fabs!(Dgrb[indx - p1][c] - Dgrb[indx + p3][c]));
                            wtsw = 1.0
                                / (eps
                                    + fabs!(Dgrb[indx - p1][c] - Dgrb[indx + p1][c])
                                    + fabs!(Dgrb[indx - p1][c] - Dgrb[indx + m3][c])
                                    + fabs!(Dgrb[indx + p1][c] - Dgrb[indx - p3][c]));
                            wtse = 1.0
                                / (eps
                                    + fabs!(Dgrb[indx + m1][c] - Dgrb[indx - m1][c])
                                    + fabs!(Dgrb[indx + m1][c] - Dgrb[indx - p3][c])
                                    + fabs!(Dgrb[indx - m1][c] - Dgrb[indx + m3][c]));

                            Dgrb[indx][c] = (wtnw
                                * (1.325 * Dgrb[indx - m1][c]
                                    - 0.175 * Dgrb[indx - m3][c]
                                    - 0.075 * Dgrb[indx - m1 - 2][c]
                                    - 0.075 * Dgrb[indx - m1 - v2][c])
                                + wtne
                                    * (1.325 * Dgrb[indx + p1][c]
                                        - 0.175 * Dgrb[indx + p3][c]
                                        - 0.075 * Dgrb[indx + p1 + 2][c]
                                        - 0.075 * Dgrb[indx + p1 + v2][c])
                                + wtsw
                                    * (1.325 * Dgrb[indx - p1][c]
                                        - 0.175 * Dgrb[indx - p3][c]
                                        - 0.075 * Dgrb[indx - p1 - 2][c]
                                        - 0.075 * Dgrb[indx - p1 - v2][c])
                                + wtse
                                    * (1.325 * Dgrb[indx + m1][c]
                                        - 0.175 * Dgrb[indx + m3][c]
                                        - 0.075 * Dgrb[indx + m1 + 2][c]
                                        - 0.075 * Dgrb[indx + m1 + v2][c]))
                                / (wtnw + wtne + wtsw + wtse);

                            cc += 2;
                            indx += 2;
                        }
                    }

                    for rr in (12..rr1 - 12) {
                        cc = 12 + (FC!(rr, 1) & 1);
                        indx = rr * TS + cc;
                        //c = FC!(rr, cc+1) / 2;
                        while cc < cc1 - 12 {
                            for c in 0..2 {
                                Dgrb[indx][c] = ((hvwt[indx - v1]) * Dgrb[indx - v1][c]
                                    + (1.0 - hvwt[indx + 1]) * Dgrb[indx + 1][c]
                                    + (1.0 - hvwt[indx - 1]) * Dgrb[indx - 1][c]
                                    + (hvwt[indx + v1]) * Dgrb[indx + v1][c])
                                    / ((hvwt[indx - v1])
                                        + (1.0 - hvwt[indx + 1])
                                        + (1.0 - hvwt[indx - 1])
                                        + (hvwt[indx + v1]));
                            }
                            cc += 2;
                            indx += 2;
                        }
                    }

                    for rr in (12..rr1 - 12) {
                        cc = 12;
                        indx = rr * TS + cc;
                        while cc < cc1 - 12 {
                            rgb[indx][0] = (rgb[indx][1] - Dgrb[indx][0]);
                            rgb[indx][2] = (rgb[indx][1] - Dgrb[indx][1]);
                            cc += 1;
                            indx += 1;
                        }
                    }

                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

                    // copy smoothed results back to image matrix
                    for rr in (16..rr1 - 16) {
                        row = rr + top;
                        cc = 16;
                        while cc < cc1 - 16 {
                            col = cc + left;
                            indx = row * width + col;
                            for c in 0..3 {
                                image[indx][c] =
                                    CLIP!(65535.0_f32 * rgb[rr * TS + cc][c] + 0.5_f32);
                            }
                            cc += 1
                        }
                    }
                    //end of main loop
                });
        });

    //let newimage = RgbImage::new_from_buffers_rgb(&red, &green, &blue, buffer.mode).unwrap();
    //Ok(newimage)
    Ok(vek_array_to_rgbimage(
        &image,
        buffer.width,
        buffer.height,
        buffer.mode,
    ))
}
