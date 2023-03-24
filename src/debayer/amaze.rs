/**
 * Port of libraw AMaZE demosaicing algorithm from Emil Martinec
 * Source: https://github.com/LibRaw/LibRaw-demosaic-pack-GPL3/blob/master/amaze_demosaic_RT.cc
 */
use crate::{
    debayer::FilterPattern, error, imagebuffer::ImageBuffer, max, min, prelude::ImageMode,
    rgbimage::RgbImage,
};
use std::ops::{Index, IndexMut};

static TS: i32 = 512;

macro_rules! FC {
    ($row:expr, $col:expr, $pattern:expr) => {
        ($pattern.fc($row, $col))
    };
}

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

macro_rules! CLIP {
    ($x: expr) => {
        LIM!($x, 0.0, 65535.0_f32)
    };
}

/// Minimally implemented vector to allow for i32 (or other) indexing.
/// Intent is to keep from having `usize` casts all over the place. Will panic if
/// you attempt to index with a negative number.
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

    pub fn resize(&mut self, new_len: usize, fill_value: T) {
        self.v.resize(new_len, fill_value);
    }

    // pub fn is_empty(&self) -> bool {
    //     self.v.is_empty()
    // }

    // pub fn len(&self) -> usize {
    //     self.v.len()
    // }
}

impl<T: Clone> Index<usize> for Vek<T> {
    type Output = T;
    fn index(&self, i: usize) -> &T {
        &self.v[i]
    }
}

impl<T: Clone> IndexMut<usize> for Vek<T> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        &mut self.v[i]
    }
}

impl<T: Clone> Index<i32> for Vek<T> {
    type Output = T;
    fn index(&self, i: i32) -> &T {
        if i < 0 {
            panic!("Index out of bounds, less than zero: {}", i);
        }
        &self.v[i as usize]
    }
}

impl<T: Clone> IndexMut<i32> for Vek<T> {
    fn index_mut(&mut self, i: i32) -> &mut T {
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

fn imagebuffer_to_vek_array(buffer: &ImageBuffer) -> Vek<Vek<f32>> {
    let mut image = vec_of_size(buffer.width * buffer.height, vec_of_size(3, 0.0_f32));
    for y in 0..buffer.height {
        for x in 0..buffer.width {
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
    for y in 0..height {
        for x in 0..width {
            let indx = y * width + x;
            image.put(x, y, v[indx][0], 0);
            image.put(x, y, v[indx][1], 1);
            image.put(x, y, v[indx][2], 2);
        }
    }
    image
}

/// Debayers a single channel image buffer using the default (RGGB) filter pattern
///
pub fn debayer(buffer: &ImageBuffer) -> error::Result<RgbImage> {
    debayer_with_pattern(buffer, FilterPattern::RGGB)
}

/// Debayers a single channel image buffer
pub fn debayer_with_pattern(
    buffer: &ImageBuffer,
    filter_pattern: FilterPattern,
) -> error::Result<RgbImage> {
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
    let mut dgrb = vec_of_size((TS * TS) as usize, vec_of_size(2, 0.0_f32));
    let mut delp = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut delm = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut rbint = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut dgrbh2 = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut dgrbv2 = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut dgintv = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut dginth = vec_of_size((TS * TS) as usize, 0.0_f32);
    // let mut Dgrbp1 = vec_of_size((TS * TS) as usize, 0.0_f32);
    // let mut Dgrbm1 = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut dgrbpsq1 = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut dgrbmsq1 = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut cfa = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut pmwt = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut rbp = vec_of_size((TS * TS) as usize, 0.0_f32);
    let mut rbm = vec_of_size((TS * TS) as usize, 0.0_f32);

    let mut nyquist = vec_of_size((TS * TS) as usize, 0);

    let height = buffer.height as i32;
    let width = buffer.width as i32;

    let pixel_value_max: f32 = match buffer.mode {
        ImageMode::U12BIT | ImageMode::U16BIT => 65535.0,
        ImageMode::U8BIT => 255.0,
    };

    // Hardcoded to 1 since we're not worring about pre-multiplied values for now
    let clip_pt = 1.0;

    let winx = 0;
    let winy = 0;
    // let winw = width;
    // let winh = height;

    let v1 = TS;
    let v2 = 2 * TS;
    let v3 = 3 * TS;
    let p1 = -TS + 1;
    let p2 = -2 * TS + 2;
    let p3 = -3 * TS + 3;
    let m1 = TS + 1;
    let m2 = 2 * TS + 2;
    let m3 = 3 * TS + 3;

    // let nbr: [i32; 5] = [-v2, -2, 2, v2, 0];

    let eps: f32 = 1e-5;
    let epssq: f32 = 1e-10;

    let arthresh: f32 = 0.75;
    let nyqthresh: f32 = 0.5;
    // let pmthresh: f32 = 0.25;
    // let lbd: f32 = 1.0;
    // let ubd: f32 = 1.0;

    let gaussodd: [f32; 4] = [0.146_597_28, 0.103_592_716, 0.073_203_616, 0.036_554_355];

    let gaussgrad: [f32; 6] = [
        0.073_844_12,
        0.062_075_12,
        0.052_181_82,
        0.036_874_194,
        0.030_997_323,
        0.018_413_194,
    ];

    // let gauss1: [f32; 3] = [0.3376688223162362, 0.12171198028231786, 0.04387081413862306];

    let gausseven: [f32; 2] = [0.137_194_95, 0.056_402_527];

    let gquinc: [f32; 4] = [0.169917, 0.108947, 0.069855, 0.0287182];

    let (ey, ex) = if FC!(0, 0, filter_pattern) == 1 && FC!(0, 1, filter_pattern) == 0 {
        (0, 1)
    } else if FC!(0, 0, filter_pattern) == 1 && FC!(0, 1, filter_pattern) != 0 {
        (1, 0)
    } else if FC!(0, 0, filter_pattern) != 1 && FC!(0, 0, filter_pattern) == 0 {
        (0, 0)
    } else {
        (1, 1)
    };

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
                    //let mut row = 0;
                    //let mut col = 0;

                    // min and max row/column in the tile
                    //let mut rrmin = 0;
                    // let mut rrmax = 0;
                    // let mut ccmin = 0;
                    // let mut ccmax = 0;

                    // counters for pixel location within the tile
                    // let mut rr = 0;
                    // let mut cc = 0;

                    // color index 0=R, 1=G, 2=B
                    //let mut c: i32 = 0;

                    // pointer counters within the tile
                    //let mut indx = 0;
                    //let mut indx1 = 0;

                    // direction counter for nbrs[]
                    //let mut dir = 0;

                    // dummy indices
                    //let mut i = 0;
                    //let mut j = 0;

                    // +1 or -1
                    //let mut sgn: f32 = 0.0;

                    //color ratios in up/down/left/right directions
                    // let mut cru: f32 = 0.0;
                    // let mut crd: f32 = 0.0;
                    // let mut crl: f32 = 0.0;
                    // let mut crr: f32 = 0.0;

                    //adaptive weights for vertical/horizontal/plus/minus directions
                    // let mut vwt: f32 = 0.0;
                    // let mut hwt: f32 = 0.0;
                    // let mut pwt: f32 = 0.0;
                    // let mut mwt: f32 = 0.0;

                    //vertical and horizontal G interpolations
                    // let mut Gintv: f32 = 0.0;
                    // let mut Ginth: f32 = 0.0;

                    //G interpolated in vert/hor directions using adaptive ratios
                    // let mut guar: f32 = 0.0;
                    // let mut gdar: f32 = 0.0;
                    // let mut glar: f32 = 0.0;
                    // let mut grar: f32 = 0.0;

                    //G interpolated in vert/hor directions using Hamilton-Adams method
                    // let mut guha: f32 = 0.0;
                    // let mut gdha: f32 = 0.0;
                    // let mut glha: f32 = 0.0;
                    // let mut grha: f32 = 0.0;

                    //interpolated G from fusing left/right or up/down
                    // let mut Ginthar: f32 = 0.0;
                    // let mut Ginthha: f32 = 0.0;
                    // let mut Gintvar: f32 = 0.0;
                    // let mut Gintvha: f32 = 0.0;

                    //color difference (G-R or G-B) variance in up/down/left/right directions
                    // let mut Dgrbvvaru: f32 = 0.0;
                    // let mut Dgrbvvard: f32 = 0.0;
                    // let mut Dgrbhvarl: f32 = 0.0;
                    // let mut Dgrbhvarr: f32 = 0.0;

                    //gradients in various directions
                    // let mut gradp: f32 = 0.0;
                    // let mut gradm: f32 = 0.0;
                    // let mut gradv: f32 = 0.0;
                    // let mut gradh: f32 = 0.0;
                    // let mut gradpm: f32 = 0.0;
                    // let mut gradhv: f32 = 0.0;

                    //color difference variances in vertical and horizontal directions
                    // let mut vcdvar: f32 = 0.0;
                    // let mut hcdvar: f32 = 0.0;
                    // let mut vcdvar1: f32 = 0.0;
                    // let mut hcdvar1: f32 = 0.0;
                    // let mut hcdltvar: f32 = 0.0;
                    // let mut vcdaltvar: f32 = 0.0;
                    // let mut hcdaltvar: f32 = 0.0;

                    //adaptive interpolation weight using variance of color differences
                    // let mut varwt: f32 = 0.0;

                    //adaptive interpolation weight using difference of left-right and up-down G interpolations
                    // let mut diffwt: f32 = 0.0;

                    //alternative adaptive weight for combining horizontal/vertical interpolations
                    // let mut hvwtalt: f32 = 0.0;

                    //temporary variables for combining interpolation weights at R and B sites
                    // let mut vo: f32 = 0.0;
                    // let mut ve: f32 = 0.0;

                    //interpolation of G in four directions
                    // let mut gu: f32 = 0.0;
                    // let mut gd: f32 = 0.0;
                    // let mut gl: f32 = 0.0;
                    // let mut gr: f32 = 0.0;

                    //variance of G in vertical/horizontal directions
                    // let mut gvarh: f32 = 0.0;
                    // let mut gvarv: f32 = 0.0;

                    //Nyquist texture test
                    // let mut nyqtest: f32 = 0.0;

                    //accumulators for Nyquist texture interpolation
                    // let mut sumh: f32 = 0.0;
                    // let mut sumv: f32 = 0.0;
                    // let mut sumsqh: f32 = 0.0;
                    // let mut sumsqv: f32 = 0.0;
                    // let mut areawt: f32 = 0.0;

                    //color ratios in diagonal directions
                    // let mut crse: f32 = 0.0;
                    // let mut crnw: f32 = 0.0;
                    // let mut crne: f32 = 0.0;
                    // let mut crsw: f32 = 0.0;

                    //color differences in diagonal directions
                    // let mut rbse: f32 = 0.0;
                    // let mut rbnw: f32 = 0.0;
                    // let mut rbne: f32 = 0.0;
                    // let mut rbsw: f32 = 0.0;

                    //adaptive weights for combining diagonal interpolations
                    // let mut wtse: f32 = 0.0;
                    // let mut wtnw: f32 = 0.0;
                    // let mut wtsw: f32 = 0.0;
                    // let mut wtne: f32 = 0.0;

                    //alternate weight for combining diagonal interpolations
                    // let mut pmwtalt: f32 = 0.0;

                    //variance of R-B in plus/minus directions
                    // let mut rbvarp: f32 = 0.0;
                    // let mut rbvarm: f32 = 0.0;

                    // rgb from input CFA data
                    // rgb values should be floating point number between 0 and 1
                    // after white balance multipliers are applied
                    // a 16 pixel border is added to each side of the image

                    // bookkeeping for borders
                    let rrmin = if top < winy { 16 } else { 0 };

                    let ccmin = if left < winx { 16 } else { 0 };

                    let rrmax = if bottom > (winy + height) {
                        winy + height - top
                    } else {
                        rr1
                    };

                    let ccmax = if right > (winx + width) {
                        winx + width - left
                    } else {
                        cc1
                    };

                    for rr in rrmin..rrmax {
                        let row = rr + top;
                        for cc in ccmin..ccmax {
                            let col = cc + left;
                            let c = FC!(rr, cc, filter_pattern);

                            let indx1 = rr * TS + cc;
                            let indx = row * width + col;
                            rgb[indx1][c] = image[indx][c] / pixel_value_max;
                            cfa[indx1] = rgb[indx1][c];
                        }
                    }

                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
                    //fill borders
                    if rrmax > 0 {
                        for rr in 0..16 {
                            for cc in ccmin..ccmax {
                                let c = FC!(rr, cc, filter_pattern);
                                rgb[rr * TS + cc][c] = rgb[(32 - rr) * TS + cc][c];
                                cfa[rr * TS + cc] = rgb[rr * TS + cc][c];
                            }
                        }
                    }
                    if rrmax < rr1 {
                        for rr in 0..16 {
                            for cc in ccmin..ccmax {
                                let c = FC!(rr, cc, filter_pattern);
                                rgb[(rrmax + rr) * TS + cc][c] = (image
                                    [(height - rr - 2) * width + left + cc][c])
                                    / pixel_value_max;
                                cfa[(rrmax + rr) * TS + cc] = rgb[(rrmax + rr) * TS + cc][c];
                            }
                        }
                    }
                    if ccmin > 0 {
                        for rr in rrmin..rrmax {
                            for cc in 0..16 {
                                let c = FC!(rr, cc, filter_pattern);
                                rgb[rr * TS + cc][c] = rgb[rr * TS + 32 - cc][c];
                                cfa[rr * TS + cc] = rgb[rr * TS + cc][c];
                            }
                        }
                    }
                    if ccmax < cc1 {
                        for rr in rrmin..rrmax {
                            for cc in 0..16 {
                                let c = FC!(rr, cc, filter_pattern);
                                rgb[rr * TS + ccmax + cc][c] = (image
                                    [(top + rr) * width + (width - cc - 2)][c])
                                    / pixel_value_max;
                                cfa[rr * TS + ccmax + cc] = rgb[rr * TS + ccmax + cc][c];
                            }
                        }
                    }

                    //also, fill the image corners
                    if rrmin > 0 && ccmin > 0 {
                        for rr in 0..16 {
                            for cc in 0..16 {
                                let c = FC!(rr, cc, filter_pattern);
                                rgb[(rr) * TS + cc][c] = rgb[(32 - rr) * TS + (32 - cc)][c];
                                cfa[(rr) * TS + cc] = rgb[(rr) * TS + cc][c];
                            }
                        }
                    }
                    if rrmax < rr1 && ccmax < cc1 {
                        for rr in 0..16 {
                            for cc in 0..16 {
                                let c = FC!(rr, cc, filter_pattern);
                                rgb[(rrmax + rr) * TS + ccmax + cc][c] = (image
                                    [(height - rr - 2) * width + (width - cc - 2)][c])
                                    / pixel_value_max;
                                cfa[(rrmax + rr) * TS + ccmax + cc] =
                                    rgb[(rrmax + rr) * TS + ccmax + cc][c];
                            }
                        }
                    }
                    if rrmin > 0 && ccmax < cc1 {
                        for rr in 0..16 {
                            for cc in 0..16 {
                                let c = FC!(rr, cc, filter_pattern);
                                rgb[(rr) * TS + ccmax + cc][c] = (image
                                    [(32 - rr) * width + (width - cc - 2)][c])
                                    / pixel_value_max;
                                cfa[(rr) * TS + ccmax + cc] = rgb[(rr) * TS + ccmax + cc][c];
                            }
                        }
                    }
                    if rrmax < rr1 && ccmin > 0 {
                        for rr in 0..16 {
                            for cc in 0..16 {
                                let c = FC!(rr, cc, filter_pattern);
                                rgb[(rrmax + rr) * TS + cc][c] = (image
                                    [(height - rr - 2) * width + (32 - cc)][c])
                                    / pixel_value_max;
                                cfa[(rrmax + rr) * TS + cc] = rgb[(rrmax + rr) * TS + cc][c];
                            }
                        }
                    }

                    //end of border fill
                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

                    for rr in 1..rr1 - 1 {
                        let mut cc = 1;
                        let mut indx = rr * TS + cc;
                        while cc < cc1 - 1 {
                            // Pick up on line 436
                            delh[indx] = (cfa[indx + 1] - cfa[indx - 1]).abs();
                            delv[indx] = (cfa[indx + v1] - cfa[indx - v1]).abs();
                            delhsq[indx] = SQR!(delh[indx]);
                            delvsq[indx] = SQR!(delv[indx]);
                            delp[indx] = (cfa[indx + p1] - cfa[indx - p1]).abs();
                            delm[indx] = (cfa[indx + m1] - cfa[indx - m1]).abs();
                            cc += 1;
                            indx += 1;
                        }
                    }

                    for rr in 2..rr1 - 2 {
                        let mut cc = 2;
                        let mut indx = rr * TS + cc;

                        while cc < cc1 - 2 {
                            dirwts[indx][0] = eps + delv[indx + v1] + delv[indx - v1] + delv[indx];
                            dirwts[indx][1] = eps + delh[indx + 1] + delh[indx - 1] + delh[indx];

                            if FC!(rr, cc, filter_pattern) & 1 >= 1 {
                                //for later use in diagonal interpolation
                                dgrbpsq1[indx] = SQR!(cfa[indx] - cfa[indx - p1])
                                    + SQR!(cfa[indx] - cfa[indx + p1]);
                                dgrbmsq1[indx] = SQR!(cfa[indx] - cfa[indx - m1])
                                    + SQR!(cfa[indx] - cfa[indx + m1]);
                            }
                            cc += 1;
                            indx += 1;
                        }
                    }
                    // end of tile initialization
                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

                    //interpolate vertical and horizontal color differences
                    for rr in 4..rr1 - 4 {
                        let mut cc = 4;
                        let mut indx = rr * TS + cc;
                        while cc < cc1 - 4 {
                            let c = FC!(rr, cc, filter_pattern);
                            let sgn = if c & 1 >= 1 { -1.0 } else { 1.0 };

                            //initialization of nyquist test
                            nyquist[indx] = 0;

                            //preparation for diag interp
                            rbint[indx] = 0.0;

                            //color ratios in each cardinal direction
                            let cru: f32 = cfa[indx - v1]
                                * (dirwts[indx - v2][0] + dirwts[indx][0])
                                / (dirwts[indx - v2][0] * (eps + cfa[indx])
                                    + dirwts[indx][0] * (eps + cfa[indx - v2]));
                            let crd: f32 = cfa[indx + v1]
                                * (dirwts[indx + v2][0] + dirwts[indx][0])
                                / (dirwts[indx + v2][0] * (eps + cfa[indx])
                                    + dirwts[indx][0] * (eps + cfa[indx + v2]));
                            let crl: f32 = cfa[indx - 1] * (dirwts[indx - 2][1] + dirwts[indx][1])
                                / (dirwts[indx - 2][1] * (eps + cfa[indx])
                                    + dirwts[indx][1] * (eps + cfa[indx - 2]));
                            let crr: f32 = cfa[indx + 1] * (dirwts[indx + 2][1] + dirwts[indx][1])
                                / (dirwts[indx + 2][1] * (eps + cfa[indx])
                                    + dirwts[indx][1] * (eps + cfa[indx + 2]));

                            let guha: f32 =
                                min!(clip_pt, cfa[indx - v1]) + 0.5 * (cfa[indx] - cfa[indx - v2]);
                            let gdha: f32 =
                                min!(clip_pt, cfa[indx + v1]) + 0.5 * (cfa[indx] - cfa[indx + v2]);
                            let glha: f32 =
                                min!(clip_pt, cfa[indx - 1]) + 0.5 * (cfa[indx] - cfa[indx - 2]);
                            let grha: f32 =
                                min!(clip_pt, cfa[indx + 1]) + 0.5 * (cfa[indx] - cfa[indx + 2]);

                            let mut guar: f32 = if fabs!(1.0_f32 - cru) < arthresh {
                                cfa[indx] * cru
                            } else {
                                guha
                            };

                            let mut gdar: f32 = if fabs!(1.0 - crd) < arthresh {
                                cfa[indx] * crd
                            } else {
                                gdha
                            };

                            let mut glar: f32 = if fabs!(1.0 - crl) < arthresh {
                                cfa[indx] * crl
                            } else {
                                glha
                            };

                            let mut grar: f32 = if fabs!(1.0 - crr) < arthresh {
                                cfa[indx] * crr
                            } else {
                                grha
                            };

                            let hwt: f32 =
                                dirwts[indx - 1][1] / (dirwts[indx - 1][1] + dirwts[indx + 1][1]);
                            let vwt: f32 = dirwts[indx - v1][0]
                                / (dirwts[indx + v1][0] + dirwts[indx - v1][0]);

                            //interpolated G via adaptive weights of cardinal evaluations
                            let gintvar: f32 = vwt * gdar + (1.0 - vwt) * guar;
                            let ginthar: f32 = hwt * grar + (1.0 - hwt) * glar;
                            let gintvha: f32 = vwt * gdha + (1.0 - vwt) * guha;
                            let ginthha: f32 = hwt * grha + (1.0 - hwt) * glha;

                            //interpolated color differences
                            vcd[indx] = sgn * (gintvar - cfa[indx]);
                            hcd[indx] = sgn * (ginthar - cfa[indx]);
                            vcdalt[indx] = sgn * (gintvha - cfa[indx]);
                            hcdalt[indx] = sgn * (ginthha - cfa[indx]);

                            if cfa[indx] > 0.8 * clip_pt
                                || gintvha > 0.8 * clip_pt
                                || ginthha > 0.8 * clip_pt
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

                    for rr in 4..rr1 - 4 {
                        let mut cc = 4;
                        let mut indx = rr * TS + cc;
                        while cc < cc1 - 4 {
                            let c = FC!(rr, cc, filter_pattern);
                            let hcdvar: f32 = 3.0
                                * (SQR!(hcd[indx - 2]) + SQR!(hcd[indx]) + SQR!(hcd[indx + 2]))
                                - SQR!(hcd[indx - 2] + hcd[indx] + hcd[indx + 2]);
                            let hcdaltvar: f32 = 3.0
                                * (SQR!(hcdalt[indx - 2])
                                    + SQR!(hcdalt[indx])
                                    + SQR!(hcdalt[indx + 2]))
                                - SQR!(hcdalt[indx - 2] + hcdalt[indx] + hcdalt[indx + 2]);
                            let vcdvar: f32 = 3.0
                                * (SQR!(vcd[indx - v2]) + SQR!(vcd[indx]) + SQR!(vcd[indx + v2]))
                                - SQR!(vcd[indx - v2] + vcd[indx] + vcd[indx + v2]);
                            let vcdaltvar: f32 = 3.0
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
                                let ginth: f32 = -hcd[indx] + cfa[indx]; //R or B
                                let gintv: f32 = -vcd[indx] + cfa[indx]; //B or R

                                if hcd[indx] > 0.0 {
                                    if 3.0 * hcd[indx] > (ginth + cfa[indx]) {
                                        hcd[indx] =
                                            -ULIM!(ginth, cfa[indx - 1], cfa[indx + 1]) + cfa[indx];
                                    } else {
                                        let hwt: f32 =
                                            1.0 - 3.0 * hcd[indx] / (eps + ginth + cfa[indx]);
                                        hcd[indx] = hwt * hcd[indx]
                                            + (1.0 - hwt)
                                                * (-ULIM!(ginth, cfa[indx - 1], cfa[indx + 1])
                                                    + cfa[indx]);
                                    }
                                }

                                if vcd[indx] > 0.0 {
                                    if 3.0 * vcd[indx] > (gintv + cfa[indx]) {
                                        vcd[indx] = -ULIM!(gintv, cfa[indx - v1], cfa[indx + v1])
                                            + cfa[indx];
                                    } else {
                                        let vwt: f32 =
                                            1.0 - 3.0 * vcd[indx] / (eps + gintv + cfa[indx]);
                                        vcd[indx] = vwt * vcd[indx]
                                            + (1.0 - vwt)
                                                * (-ULIM!(gintv, cfa[indx - v1], cfa[indx + v1])
                                                    + cfa[indx]);
                                    }
                                }

                                if ginth > clip_pt {
                                    hcd[indx] =
                                        -ULIM!(ginth, cfa[indx - 1], cfa[indx + 1]) + cfa[indx];
                                } //for RT implementation
                                if gintv > clip_pt {
                                    vcd[indx] =
                                        -ULIM!(gintv, cfa[indx - v1], cfa[indx + v1]) + cfa[indx];
                                }
                            } else {
                                // R or B site
                                let ginth: f32 = hcd[indx] + cfa[indx]; //interpolated G
                                let gintv: f32 = vcd[indx] + cfa[indx];

                                if hcd[indx] < 0.0 {
                                    if 3.0 * hcd[indx] < -(ginth + cfa[indx]) {
                                        hcd[indx] =
                                            ULIM!(ginth, cfa[indx - 1], cfa[indx + 1]) - cfa[indx];
                                    } else {
                                        let hwt: f32 =
                                            1.0 + 3.0 * hcd[indx] / (eps + ginth + cfa[indx]);
                                        hcd[indx] = hwt * hcd[indx]
                                            + (1.0 - hwt)
                                                * (ULIM!(ginth, cfa[indx - 1], cfa[indx + 1])
                                                    - cfa[indx]);
                                    }
                                }
                                if vcd[indx] < 0.0 {
                                    if 3.0 * vcd[indx] < -(gintv + cfa[indx]) {
                                        vcd[indx] = ULIM!(gintv, cfa[indx - v1], cfa[indx + v1])
                                            - cfa[indx];
                                    } else {
                                        let vwt: f32 =
                                            1.0 + 3.0 * vcd[indx] / (eps + gintv + cfa[indx]);
                                        vcd[indx] = vwt * vcd[indx]
                                            + (1.0 - vwt)
                                                * (ULIM!(gintv, cfa[indx - v1], cfa[indx + v1])
                                                    - cfa[indx]);
                                    }
                                }
                                if ginth > clip_pt {
                                    hcd[indx] =
                                        ULIM!(ginth, cfa[indx - 1], cfa[indx + 1]) - cfa[indx];
                                } //for RT implementation
                                if gintv > clip_pt {
                                    vcd[indx] =
                                        ULIM!(gintv, cfa[indx - v1], cfa[indx + v1]) - cfa[indx];
                                }
                            }

                            vcdsq[indx] = SQR!(vcd[indx]);
                            hcdsq[indx] = SQR!(hcd[indx]);
                            cddiffsq[indx] = SQR!(vcd[indx] - hcd[indx]);

                            cc += 1;
                            indx += 1;
                        }
                    }

                    for rr in 6..rr1 - 6 {
                        let mut cc = 6 + (FC!(rr, 2, filter_pattern) & 1);
                        let mut indx = rr * TS + cc;

                        while cc < cc1 - 6 {
                            let mut dgrbvvaru: f32 = 4.0
                                * (vcdsq[indx]
                                    + vcdsq[indx - v1]
                                    + vcdsq[indx - v2]
                                    + vcdsq[indx - v3])
                                - SQR!(
                                    vcd[indx] + vcd[indx - v1] + vcd[indx - v2] + vcd[indx - v3]
                                );
                            let mut dgrbvvard: f32 = 4.0
                                * (vcdsq[indx]
                                    + vcdsq[indx + v1]
                                    + vcdsq[indx + v2]
                                    + vcdsq[indx + v3])
                                - SQR!(
                                    vcd[indx] + vcd[indx + v1] + vcd[indx + v2] + vcd[indx + v3]
                                );
                            let mut dgrbhvarl: f32 = 4.0
                                * (hcdsq[indx]
                                    + hcdsq[indx - 1]
                                    + hcdsq[indx - 2]
                                    + hcdsq[indx - 3])
                                - SQR!(hcd[indx] + hcd[indx - 1] + hcd[indx - 2] + hcd[indx - 3]);
                            let mut dgrbhvarr: f32 = 4.0
                                * (hcdsq[indx]
                                    + hcdsq[indx + 1]
                                    + hcdsq[indx + 2]
                                    + hcdsq[indx + 3])
                                - SQR!(hcd[indx] + hcd[indx + 1] + hcd[indx + 2] + hcd[indx + 3]);

                            let hwt: f32 =
                                dirwts[indx - 1][1] / (dirwts[indx - 1][1] + dirwts[indx + 1][1]);
                            let vwt: f32 = dirwts[indx - v1][0]
                                / (dirwts[indx + v1][0] + dirwts[indx - v1][0]);

                            let vcdvar: f32 = epssq + vwt * dgrbvvard + (1.0 - vwt) * dgrbvvaru;
                            let hcdvar: f32 = epssq + hwt * dgrbhvarr + (1.0 - hwt) * dgrbhvarl;

                            dgrbvvaru = (dgintv[indx]) + (dgintv[indx - v1]) + (dgintv[indx - v2]);
                            dgrbvvard = (dgintv[indx]) + (dgintv[indx + v1]) + (dgintv[indx + v2]);
                            dgrbhvarl = (dginth[indx]) + (dginth[indx - 1]) + (dginth[indx - 2]);
                            dgrbhvarr = (dginth[indx]) + (dginth[indx + 1]) + (dginth[indx + 2]);

                            let vcdvar1: f32 = epssq + vwt * dgrbvvard + (1.0 - vwt) * dgrbvvaru;
                            let hcdvar1: f32 = epssq + hwt * dgrbhvarr + (1.0 - hwt) * dgrbhvarl;

                            //determine adaptive weights for G interpolation
                            let varwt = hcdvar / (vcdvar + hcdvar);
                            let diffwt = hcdvar1 / (vcdvar1 + hcdvar1);

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

                    for rr in 6..rr1 - 6 {
                        let mut cc = 6 + (FC!(rr, 2, filter_pattern) & 1);
                        let mut indx = rr * TS + cc;
                        while cc < cc1 - 6 {
                            //nyquist texture test: ask if difference of vcd compared to hcd is larger or smaller than RGGB gradients
                            let mut nyqtest = gaussodd[0] * cddiffsq[indx]
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
                                        + cddiffsq[indx + m2]);

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

                    for rr in 8..rr1 - 8 {
                        let mut cc = 8 + (FC!(rr, 2, filter_pattern) & 1);
                        let mut indx = rr * TS + cc;
                        while cc < cc1 - 8 {
                            let areawt = (nyquist[indx - v2]
                                + nyquist[indx - m1]
                                + nyquist[indx + p1]
                                + nyquist[indx - 2]
                                + nyquist[indx]
                                + nyquist[indx + 2]
                                + nyquist[indx - p1]
                                + nyquist[indx + m1]
                                + nyquist[indx + v2])
                                as f32;
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

                    for rr in 8..rr1 - 8 {
                        let mut cc = 8 + (FC!(rr, 2, filter_pattern) & 1);
                        let mut indx = rr * TS + cc;
                        while cc < cc1 - 8 {
                            if nyquist[indx] >= 1 {
                                // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
                                // area interpolation
                                let mut sumh = 0.0;
                                let mut sumv = 0.0;
                                let mut sumsqh = 0.0;
                                let mut sumsqv = 0.0;
                                let mut areawt = 0.0;
                                for i in (-6..7).step_by(2) {
                                    for j in (-6..7).step_by(2) {
                                        let indx1 = (rr + i) * TS + cc + j;
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
                                let hcdvar: f32 = epssq + max!(0.0, areawt * sumsqh - sumh * sumh);
                                let vcdvar: f32 = epssq + max!(0.0, areawt * sumsqv - sumv * sumv);
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
                    for rr in 8..rr1 - 8 {
                        let mut cc = 8 + (FC!(rr, 2, filter_pattern) & 1);
                        let mut indx = rr * TS + cc;
                        while cc < cc1 - 8 {
                            let hvwtalt: f32 = 0.25
                                * (hvwt[indx - m1]
                                    + hvwt[indx + p1]
                                    + hvwt[indx - p1]
                                    + hvwt[indx + m1]);
                            let vo = fabs!(0.5 - hvwt[indx]);
                            let ve = fabs!(0.5 - hvwtalt);

                            if vo < ve {
                                hvwt[indx] = hvwtalt;
                            } //a better result was obtained from the neighbors

                            dgrb[indx][0] = hcd[indx] * (1.0 - hvwt[indx]) + vcd[indx] * hvwt[indx]; //evaluate color differences
                            rgb[indx][1] = cfa[indx] + dgrb[indx][0]; //evaluate G (finally!)

                            if nyquist[indx] >= 1 {
                                dgrbh2[indx] = SQR!(
                                    rgb[indx][1] - 0.5_f32 * (rgb[indx - 1][1] + rgb[indx + 1][1])
                                );
                                dgrbv2[indx] = SQR!(
                                    rgb[indx][1]
                                        - 0.5_f32 * (rgb[indx - v1][1] + rgb[indx + v1][1])
                                );
                            } else {
                                dgrbh2[indx] = 0.0;
                                dgrbv2[indx] = 0.0;
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
                    for rr in 8..rr1 - 8 {
                        let mut cc = 8 + (FC!(rr, 2, filter_pattern) & 1);
                        let mut indx = rr * TS + cc;
                        while cc < cc1 - 8 {
                            if nyquist[indx] > 1 {
                                //local averages (over Nyquist pixels only) of G curvature squared
                                let gvarh = epssq
                                    + (gquinc[0] * dgrbh2[indx]
                                        + gquinc[1]
                                            * (dgrbh2[indx - m1]
                                                + dgrbh2[indx + p1]
                                                + dgrbh2[indx - p1]
                                                + dgrbh2[indx + m1])
                                        + gquinc[2]
                                            * (dgrbh2[indx - v2]
                                                + dgrbh2[indx - 2]
                                                + dgrbh2[indx + 2]
                                                + dgrbh2[indx + v2])
                                        + gquinc[3]
                                            * (dgrbh2[indx - m2]
                                                + dgrbh2[indx + p2]
                                                + dgrbh2[indx - p2]
                                                + dgrbh2[indx + m2]));
                                let gvarv = epssq
                                    + (gquinc[0] * dgrbv2[indx]
                                        + gquinc[1]
                                            * (dgrbv2[indx - m1]
                                                + dgrbv2[indx + p1]
                                                + dgrbv2[indx - p1]
                                                + dgrbv2[indx + m1])
                                        + gquinc[2]
                                            * (dgrbv2[indx - v2]
                                                + dgrbv2[indx - 2]
                                                + dgrbv2[indx + 2]
                                                + dgrbv2[indx + v2])
                                        + gquinc[3]
                                            * (dgrbv2[indx - m2]
                                                + dgrbv2[indx + p2]
                                                + dgrbv2[indx - p2]
                                                + dgrbv2[indx + m2]));
                                //use the results as weights for refined G interpolation
                                dgrb[indx][0] =
                                    (hcd[indx] * gvarv + vcd[indx] * gvarh) / (gvarv + gvarh);
                                rgb[indx][1] = cfa[indx] + dgrb[indx][0];
                            }
                            cc += 2;
                            indx += 2;
                        }
                    }
                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
                    // diagonal interpolation correction
                    for rr in 8..rr1 - 8 {
                        let mut cc = 8 + (FC!(rr, 2, filter_pattern) & 1);
                        let mut indx = rr * TS + cc;
                        while cc < cc1 - 8 {
                            let rbvarp = epssq
                                + (gausseven[0]
                                    * (dgrbpsq1[indx - v1]
                                        + dgrbpsq1[indx - 1]
                                        + dgrbpsq1[indx + 1]
                                        + dgrbpsq1[indx + v1])
                                    + gausseven[1]
                                        * (dgrbpsq1[indx - v2 - 1]
                                            + dgrbpsq1[indx - v2 + 1]
                                            + dgrbpsq1[indx - 2 - v1]
                                            + dgrbpsq1[indx + 2 - v1]
                                            + dgrbpsq1[indx - 2 + v1]
                                            + dgrbpsq1[indx + 2 + v1]
                                            + dgrbpsq1[indx + v2 - 1]
                                            + dgrbpsq1[indx + v2 + 1]));
                            let rbvarm = epssq
                                + (gausseven[0]
                                    * (dgrbmsq1[indx - v1]
                                        + dgrbmsq1[indx - 1]
                                        + dgrbmsq1[indx + 1]
                                        + dgrbmsq1[indx + v1])
                                    + gausseven[1]
                                        * (dgrbmsq1[indx - v2 - 1]
                                            + dgrbmsq1[indx - v2 + 1]
                                            + dgrbmsq1[indx - 2 - v1]
                                            + dgrbmsq1[indx + 2 - v1]
                                            + dgrbmsq1[indx - 2 + v1]
                                            + dgrbmsq1[indx + 2 + v1]
                                            + dgrbmsq1[indx + v2 - 1]
                                            + dgrbmsq1[indx + v2 + 1]));
                            // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

                            //diagonal color ratios
                            let crse =
                                2.0 * (cfa[indx + m1]) / (eps + cfa[indx] + (cfa[indx + m2]));
                            let crnw =
                                2.0 * (cfa[indx - m1]) / (eps + cfa[indx] + (cfa[indx - m2]));
                            let crne =
                                2.0 * (cfa[indx + p1]) / (eps + cfa[indx] + (cfa[indx + p2]));
                            let crsw =
                                2.0 * (cfa[indx - p1]) / (eps + cfa[indx] + (cfa[indx - p2]));

                            //assign B/R at R/B sites
                            let rbse = if fabs!(1.0 - crse) < arthresh {
                                cfa[indx] * crse
                            }
                            //use this if more precise diag interp is necessary
                            else {
                                (cfa[indx + m1]) + 0.5 * (cfa[indx] - cfa[indx + m2])
                            };

                            let rbnw = if fabs!(1.0 - crnw) < arthresh {
                                cfa[indx] * crnw
                            } else {
                                (cfa[indx - m1]) + 0.5 * (cfa[indx] - cfa[indx - m2])
                            };

                            let rbne = if fabs!(1.0 - crne) < arthresh {
                                cfa[indx] * crne
                            } else {
                                (cfa[indx + p1]) + 0.5 * (cfa[indx] - cfa[indx + p2])
                            };

                            let rbsw = if fabs!(1.0 - crsw) < arthresh {
                                cfa[indx] * crsw
                            } else {
                                (cfa[indx - p1]) + 0.5 * (cfa[indx] - cfa[indx - p2])
                            };

                            let wtse = eps + delm[indx] + delm[indx + m1] + delm[indx + m2]; //same as for wtu,wtd,wtl,wtr
                            let wtnw = eps + delm[indx] + delm[indx - m1] + delm[indx - m2];
                            let wtne = eps + delp[indx] + delp[indx + p1] + delp[indx + p2];
                            let wtsw = eps + delp[indx] + delp[indx - p1] + delp[indx - p2];

                            rbm[indx] = (wtse * rbnw + wtnw * rbse) / (wtse + wtnw);
                            rbp[indx] = (wtne * rbsw + wtsw * rbne) / (wtne + wtsw);

                            pmwt[indx] = rbvarm / (rbvarp + rbvarm);

                            // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
                            //bound the interpolation in regions of high saturation

                            if rbp[indx] < cfa[indx] {
                                rbp[indx] = if 2.0 * rbp[indx] < cfa[indx] {
                                    ULIM!(rbp[indx], cfa[indx - p1], cfa[indx + p1])
                                } else {
                                    let pwt: f32 = 2.0 * (cfa[indx] - rbp[indx])
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
                                    let mwt: f32 = 2.0 * (cfa[indx] - rbm[indx])
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

                    for rr in 10..rr1 - 10 {
                        let mut cc = 10 + (FC!(rr, 2, filter_pattern) & 1);
                        let mut indx = rr * TS + cc;
                        while cc < cc1 - 10 {
                            let pmwtalt = 0.25
                                * (pmwt[indx - m1]
                                    + pmwt[indx + p1]
                                    + pmwt[indx - p1]
                                    + pmwt[indx + m1]);
                            let vo = fabs!(0.5 - pmwt[indx]);
                            let ve = fabs!(0.5 - pmwtalt);

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

                    for rr in 12..rr1 - 12 {
                        let mut cc = 12 + (FC!(rr, 2, filter_pattern) & 1);
                        let mut indx = rr * TS + cc;
                        while cc < cc1 - 12 {
                            // if fabs!(0.5 - pmwt[indx]) < fabs!(0.5 - hvwt[indx]) {
                            //     cc += 2;
                            //     indx += 2;
                            //     continue;
                            // }

                            //now interpolate G vertically/horizontally using R+B values
                            //unfortunately, since G interpolation cannot be done diagonally this may lead to color shifts
                            //color ratios for G interpolation

                            let cru: f32 =
                                cfa[indx - v1] * 2.0 / (eps + rbint[indx] + rbint[indx - v2]);
                            let crd: f32 =
                                cfa[indx + v1] * 2.0 / (eps + rbint[indx] + rbint[indx + v2]);
                            let crl: f32 =
                                cfa[indx - 1] * 2.0 / (eps + rbint[indx] + rbint[indx - 2]);
                            let crr: f32 =
                                cfa[indx + 1] * 2.0 / (eps + rbint[indx] + rbint[indx + 2]);

                            let gu = if fabs!(1.0 - cru) < arthresh {
                                rbint[indx] * cru
                            } else {
                                cfa[indx - v1] + 0.5 * (rbint[indx] - rbint[indx - v2])
                            };
                            let gd = if fabs!(1.0 - crd) < arthresh {
                                rbint[indx] * crd
                            } else {
                                cfa[indx + v1] + 0.5 * (rbint[indx] - rbint[indx + v2])
                            };
                            let gl = if fabs!(1.0 - crl) < arthresh {
                                rbint[indx] * crl
                            } else {
                                cfa[indx - 1] + 0.5 * (rbint[indx] - rbint[indx - 2])
                            };
                            let gr = if fabs!(1.0 - crr) < arthresh {
                                rbint[indx] * crr
                            } else {
                                cfa[indx + 1] + 0.5 * (rbint[indx] - rbint[indx + 2])
                            };

                            //interpolated G via adaptive weights of cardinal evaluations
                            let mut gintv: f32 = (dirwts[indx - v1][0] * gd
                                + dirwts[indx + v1][0] * gu)
                                / (dirwts[indx + v1][0] + dirwts[indx - v1][0]);
                            let mut ginth: f32 = (dirwts[indx - 1][1] * gr
                                + dirwts[indx + 1][1] * gl)
                                / (dirwts[indx - 1][1] + dirwts[indx + 1][1]);

                            // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
                            //bound the interpolation in regions of high saturation
                            if gintv < rbint[indx] {
                                gintv = if 2.0 * gintv < rbint[indx] {
                                    ULIM!(gintv, cfa[indx - v1], cfa[indx + v1])
                                } else {
                                    let vwt: f32 =
                                        2.0 * (rbint[indx] - gintv) / (eps + gintv + rbint[indx]);
                                    vwt * gintv
                                        + (1.0 - vwt) * ULIM!(gintv, cfa[indx - v1], cfa[indx + v1])
                                };
                            }
                            if ginth < rbint[indx] {
                                ginth = if 2.0 * ginth < rbint[indx] {
                                    ULIM!(ginth, cfa[indx - 1], cfa[indx + 1])
                                } else {
                                    let hwt: f32 =
                                        2.0 * (rbint[indx] - ginth) / (eps + ginth + rbint[indx]);
                                    hwt * ginth
                                        + (1.0 - hwt) * ULIM!(ginth, cfa[indx - 1], cfa[indx + 1])
                                }
                            }

                            if ginth > clip_pt {
                                ginth = ULIM!(ginth, cfa[indx - 1], cfa[indx + 1]);
                            } //for RT implementation
                            if gintv > clip_pt {
                                gintv = ULIM!(gintv, cfa[indx - v1], cfa[indx + v1]);
                            }

                            rgb[indx][1] = ginth * (1.0 - hvwt[indx]) + gintv * hvwt[indx];
                            dgrb[indx][0] = rgb[indx][1] - cfa[indx];

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
                        let mut cc = 13 - ex;
                        let mut indx = rr * TS + cc;
                        while cc < cc1 - 12 {
                            dgrb[indx][1] = dgrb[indx][0]; //split out G-B from G-R
                            dgrb[indx][0] = 0.0_f32;
                            cc += 2;
                            indx += 2;
                        }
                    }

                    for rr in 12..rr1 - 12 {
                        let mut cc = 12 + (FC!(rr, 2, filter_pattern) & 1);
                        let mut indx = rr * TS + cc;
                        let c = 1 - FC!(rr, cc, filter_pattern) / 2;
                        while cc < cc1 - 12 {
                            let wtnw = 1.0
                                / (eps
                                    + fabs!(dgrb[indx - m1][c] - dgrb[indx + m1][c])
                                    + fabs!(dgrb[indx - m1][c] - dgrb[indx - m3][c])
                                    + fabs!(dgrb[indx + m1][c] - dgrb[indx - m3][c]));
                            let wtne = 1.0
                                / (eps
                                    + fabs!(dgrb[indx + p1][c] - dgrb[indx - p1][c])
                                    + fabs!(dgrb[indx + p1][c] - dgrb[indx + p3][c])
                                    + fabs!(dgrb[indx - p1][c] - dgrb[indx + p3][c]));
                            let wtsw = 1.0
                                / (eps
                                    + fabs!(dgrb[indx - p1][c] - dgrb[indx + p1][c])
                                    + fabs!(dgrb[indx - p1][c] - dgrb[indx + m3][c])
                                    + fabs!(dgrb[indx + p1][c] - dgrb[indx - p3][c]));
                            let wtse = 1.0
                                / (eps
                                    + fabs!(dgrb[indx + m1][c] - dgrb[indx - m1][c])
                                    + fabs!(dgrb[indx + m1][c] - dgrb[indx - p3][c])
                                    + fabs!(dgrb[indx - m1][c] - dgrb[indx + m3][c]));

                            dgrb[indx][c] = (wtnw
                                * (1.325 * dgrb[indx - m1][c]
                                    - 0.175 * dgrb[indx - m3][c]
                                    - 0.075 * dgrb[indx - m1 - 2][c]
                                    - 0.075 * dgrb[indx - m1 - v2][c])
                                + wtne
                                    * (1.325 * dgrb[indx + p1][c]
                                        - 0.175 * dgrb[indx + p3][c]
                                        - 0.075 * dgrb[indx + p1 + 2][c]
                                        - 0.075 * dgrb[indx + p1 + v2][c])
                                + wtsw
                                    * (1.325 * dgrb[indx - p1][c]
                                        - 0.175 * dgrb[indx - p3][c]
                                        - 0.075 * dgrb[indx - p1 - 2][c]
                                        - 0.075 * dgrb[indx - p1 - v2][c])
                                + wtse
                                    * (1.325 * dgrb[indx + m1][c]
                                        - 0.175 * dgrb[indx + m3][c]
                                        - 0.075 * dgrb[indx + m1 + 2][c]
                                        - 0.075 * dgrb[indx + m1 + v2][c]))
                                / (wtnw + wtne + wtsw + wtse);

                            cc += 2;
                            indx += 2;
                        }
                    }

                    for rr in 12..rr1 - 12 {
                        let mut cc = 12 + (FC!(rr, 1, filter_pattern) & 1);
                        let mut indx = rr * TS + cc;
                        //c = FC!(rr, cc+1) / 2;
                        while cc < cc1 - 12 {
                            for c in 0..2 {
                                dgrb[indx][c] = ((hvwt[indx - v1]) * dgrb[indx - v1][c]
                                    + (1.0 - hvwt[indx + 1]) * dgrb[indx + 1][c]
                                    + (1.0 - hvwt[indx - 1]) * dgrb[indx - 1][c]
                                    + (hvwt[indx + v1]) * dgrb[indx + v1][c])
                                    / ((hvwt[indx - v1])
                                        + (1.0 - hvwt[indx + 1])
                                        + (1.0 - hvwt[indx - 1])
                                        + (hvwt[indx + v1]));
                            }
                            cc += 2;
                            indx += 2;
                        }
                    }

                    for rr in 12..rr1 - 12 {
                        let mut cc = 12;
                        let mut indx = rr * TS + cc;
                        while cc < cc1 - 12 {
                            rgb[indx][0] = rgb[indx][1] - dgrb[indx][0];
                            rgb[indx][2] = rgb[indx][1] - dgrb[indx][1];
                            cc += 1;
                            indx += 1;
                        }
                    }

                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
                    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

                    // copy smoothed results back to image matrix
                    for rr in 16..rr1 - 16 {
                        let row = rr + top;
                        let mut cc = 16;
                        while cc < cc1 - 16 {
                            let col = cc + left;
                            let indx = row * width + col;
                            for c in 0..3 {
                                image[indx][c] =
                                    CLIP!(pixel_value_max * rgb[rr * TS + cc][c] + 0.5_f32);
                            }
                            cc += 1
                        }
                    }
                    //end of main loop
                });
        });

    Ok(vek_array_to_rgbimage(
        &image,
        buffer.width,
        buffer.height,
        buffer.mode,
    ))
}
