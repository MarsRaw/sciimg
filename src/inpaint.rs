// https://www.researchgate.net/publication/238183352_An_Image_Inpainting_Technique_Based_on_the_Fast_Marching_Method

// INFO: Rayon is a data-parallelism library, that is VERY trivial to use.
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;

use crate::{enums, error, imagebuffer::ImageBuffer, rgbimage::RgbImage, stats};

//INFO: as you're doing a fair bit of comparitve number work you may want to look into the
//comparison and ordering traits, many of them are available with #[derive(x,y,z)]
#[derive(PartialEq, PartialOrd, Default, Debug, Clone)]
struct Point {
    x: usize,
    y: usize,
    score: u32,
}

struct RgbVec {
    rgb: Vec<[f32; 3]>,
    width: usize,
    height: usize,
}

const DEFAULT_WINDOW_SIZE: i32 = 3;

fn get_num_good_neighbors(mask: &ImageBuffer, x: i32, y: i32) -> u32 {
    // Juggling the possibility of negitive numbers and whether or now we allow that.
    // a match or matches! would be more appropriate here... but you say you dislike them so...
    // TODO: refactor into a match.
    let t = if y > 0 {
        mask.get(x as usize, (y - 1) as usize).unwrap() == 0.0
    } else {
        false
    };
    let tl = if x > 0 && y > 0 {
        mask.get((x - 1) as usize, (y - 1) as usize).unwrap() == 0.0
    } else {
        false
    };
    let l = if x > 0 {
        mask.get((x - 1) as usize, y as usize).unwrap() == 0.0
    } else {
        false
    };
    let bl = if x > 0 && y < mask.height as i32 - 1 {
        mask.get((x - 1) as usize, (y + 1) as usize).unwrap() == 0.0
    } else {
        false
    };
    let b = if y < mask.height as i32 - 1 {
        mask.get(x as usize, (y + 1) as usize).unwrap() == 0.0
    } else {
        false
    };
    let br = if x < mask.width as i32 - 1 && y < mask.height as i32 - 1 {
        mask.get((x + 1) as usize, (y + 1) as usize).unwrap() == 0.0
    } else {
        false
    };
    let r = if x < mask.width as i32 - 1 {
        mask.get((x + 1) as usize, y as usize).unwrap() == 0.0
    } else {
        false
    };
    let tr = if x < mask.width as i32 - 1 && y > 0 {
        mask.get((x + 1) as usize, (y - 1) as usize).unwrap() == 0.0
    } else {
        false
    };

    let mut s = 0;

    s += if t { 1 } else { 0 };
    s += if tl { 1 } else { 0 };
    s += if l { 1 } else { 0 };
    s += if bl { 1 } else { 0 };
    s += if b { 1 } else { 0 };
    s += if br { 1 } else { 0 };
    s += if r { 1 } else { 0 };
    s += if tr { 1 } else { 0 };

    s
}

// SOOOOOOooooooooo sloooooooooooooooow :-(
// INFO: perhaps consider something like find_starting_point_par() below...
fn find_starting_point(mask: &ImageBuffer) -> Option<Point> {
    for y in 0..mask.height {
        for x in 0..mask.width {
            if let Ok(v) = mask.get(x, y) {
                if v > 0.0 {
                    return Some(Point { x, y, score: 0 });
                }
            }
        }
    }
    None
}

// INFO:
// I do not know about the algorithm you're using but if you're doing nested forloops it's gonna be
// a bad time...
// This would probably be ok assuming you are ok with what may ?potentially? be a range of starting
// points...
fn find_starting_point_par(mask: &&ImageBuffer) -> Option<Point> {
    // Note the call to into_par_iter(), the parallel version of into_iter() from Rayon.
    _ = (0..mask.height).into_par_iter().map(|y| {
        (0..mask.width).into_iter().map(move |x| {
            if let Ok(v) = mask.get(x, y) {
                if v > 0.0 {
                    return Some(Point { x, y, score: 0 });
                } else {
                    None
                }
            } else {
                None
            }
        })
    });
    // INFO:
    // If you expect truly, truly large images, consider a Mutex<Point> and a threadpool via the
    // Crossbeam or Rayon crates.
    None
}

fn isolate_window(
    buffer: &RgbVec,
    mask: &ImageBuffer,
    channel: usize,
    window_size: i32, // QUESTION: window sizes can be negative?
    x: usize,
    y: usize,
) -> Vec<f32> {
    let _v: Vec<f32> = Vec::with_capacity(36);
    let start = (window_size / 2 * -1) as usize;
    let end = (window_size / 2 + 1) as usize;

    // INFO:
    // Consider iterators with .collect::<Vec<f32>>()
    // for _y in start..end as i32 {
    //     for _x in start..end as i32 {
    //         let get_x = x as i32 + _x;
    //         let get_y = y as i32 + _y;
    //         if get_x >= 0
    //             && get_x < buffer.width as i32
    //             && get_y >= 0
    //             && get_y < buffer.height as i32
    //             && mask.get(get_x as usize, get_y as usize).unwrap() == 0.0
    //         {
    //             v.push(buffer.rgb[(get_y * buffer.width as i32 + get_x) as usize][channel]);
    //         }
    //     }
    // }
    // This .iter() or .into_iter() lend themselves very nicely to the Rayon library for parallelism, you can often swap .iter() for .par_iter() and so on. [it's not always trivial, but it often is --esp on img stuff].
    (start..end)
        .into_iter()
        .map(|i| {
            let v_inner = (start..end)
                .into_iter()
                .filter_map(move |j| {
                    let get_x = x + &i;
                    let get_y = y + &j;

                    if get_x >= 0 // This comparison is redundant, the type system can guarantee
                    // that it'll be greater than 0.
                        && get_x < buffer.width
                        && get_y & y >= 0 // This one too..
                        && get_y & y < buffer.height
                        && mask.get(get_x as usize, get_y as usize).unwrap() == 0.0
                    // .unwrap(),
                    // unless you the developer, can guarantee that it will not panic are for the
                    // most part considered poor form.
                    {
                        Some(buffer.rgb[(get_y * buffer.width + get_x) as usize][channel])
                    } else {
                        None
                    }
                })
                .collect::<Vec<f32>>();

            v_inner
        })
        .flatten()
        .collect::<Vec<f32>>()
}

fn predict_value(buffer: &RgbVec, mask: &ImageBuffer, channel: usize, x: usize, y: usize) -> f32 {
    let window = isolate_window(&buffer, &mask, channel, DEFAULT_WINDOW_SIZE, x, y);
    if let Some(mean) = stats::mean(&window[0..]) {
        mean
    } else {
        panic!("Unless it's impossible for this to fail something should be done here..")
    }
}

// INFO:
// Consider taking a generic numeric type with a signature like:
// fn get_point_and_score_at_xy<N>(mask, &ImageBuffer, x: N, y:N)->...
fn get_point_and_score_at_xy(mask: &ImageBuffer, x: i32, y: i32) -> Option<Point> {
    if x < 0 || x >= mask.width as i32 || y < 0 || y >= mask.height as i32 {
        return None;
    }

    // INFO:
    // if-let would be nicer...
    //let v = mask.get(x as usize, y as usize).unwrap();
    if let Ok(v) = mask.get(x as usize, y as usize) {
        if v == 0.0 {
            return None;
        }
    }

    let score = get_num_good_neighbors(&mask, x, y);

    Some(Point {
        x: x as usize,
        y: y as usize,
        score,
    })
}

//INFO: simplified this a little, unsure why you went for an Option...
//if there was a good reason (perhaps that you may not find a next point?)
// helper functions like this are often (in numeriacly heavy libraries decorated with an inline
// macro) NOTE: inline in rust is NOT like inline in c++, do the reading!
#[inline(always)]
fn find_larger(pt: Point, right: Point) -> Point {
    if pt.score > right.score {
        pt.clone()
    } else {
        right.clone()
    }
}

fn find_next_point(mask: &ImageBuffer, x: i32, y: i32) -> Option<Point> {
    // INFO:
    // If it's always this sort of construction perhaps consider something like below's
    // Point::create_identity() method.

    // let mut pts: Vec<Option<Point>> = Vec::with_capacity(8);
    // QUESTION: some kind of identity matrix?
    // YOURS:
    // pts.push(get_point_and_score_at_xy(&mask, x, y - 1));
    // pts.push(get_point_and_score_at_xy(&mask, x - 1, y - 1));
    // pts.push(get_point_and_score_at_xy(&mask, x - 1, y));
    // pts.push(get_point_and_score_at_xy(&mask, x - 1, y + 1));
    // pts.push(get_point_and_score_at_xy(&mask, x, y + 1));
    // pts.push(get_point_and_score_at_xy(&mask, x + 1, y + 1));
    // pts.push(get_point_and_score_at_xy(&mask, x + 1, y));
    // pts.push(get_point_and_score_at_xy(&mask, x + 1, y - 1));
    //
    //let mut largest_score: Option<Point> = None;
    // for opt_pt in pts.iter() {
    //     match opt_pt {
    //         Some(pt) => {
    //             largest_score = Some(find_larger(largest_score, pt));
    //         }
    //         None => (),
    //     }
    // }

    // MINE, if using your constructor with the ... push() calls
    // You'll find rust's iterators, in general compile to tighter assembly than anything written
    // in the python-like for loop syntax.
    // If you want to use your Vec<Option<point>> impl
    // pts.iter().fold(Some(Point::default()), |p1, p2| {
    //     let p1_u = p1.unwrap();
    //     let p2_u = p2.clone().unwrap();
    //     Some(if p1_u.score > p2_u.score { p1_u } else { p2_u })
    // })

    // MINE:
    // or else...using the preflattened in the create_identity constructors
    let pts = Point::create_identity(&mask, x, y);
    Some(pts.iter().fold(Point::default(), |p1, p2| {
        if p1.score > p2.score {
            p1
        } else {
            p2.clone() // You'll almost always see the .clone() call in folds.
        }
    }))
}

impl Point {
    fn create_identity(mask: &ImageBuffer, x: i32, y: i32) -> Vec<Point> {
        let mut pts: Vec<Option<Point>> = Vec::with_capacity(8);

        pts.push(get_point_and_score_at_xy(&mask, x, y - 1));
        pts.push(get_point_and_score_at_xy(&mask, x - 1, y - 1));
        pts.push(get_point_and_score_at_xy(&mask, x - 1, y));
        pts.push(get_point_and_score_at_xy(&mask, x - 1, y + 1));
        pts.push(get_point_and_score_at_xy(&mask, x, y + 1));
        pts.push(get_point_and_score_at_xy(&mask, x + 1, y + 1));
        pts.push(get_point_and_score_at_xy(&mask, x + 1, y));
        pts.push(get_point_and_score_at_xy(&mask, x + 1, y - 1));

        pts.into_iter().flat_map(|p| p).collect()
    }
}
//INFO: I'll probably leave the rest here for you to decide whether or not you prefer these 'rusty'
//changes.

fn infill(buffer: &mut RgbVec, mask: &mut ImageBuffer, starting: &Point) {
    let mut current = starting.to_owned();
    loop {
        let pt_new_value_0 = predict_value(&buffer, &mask, 0, current.x, current.y);
        let pt_new_value_1 = predict_value(&buffer, &mask, 1, current.x, current.y);
        let pt_new_value_2 = predict_value(&buffer, &mask, 2, current.x, current.y);

        buffer.rgb[current.y * buffer.width + current.x][0] = pt_new_value_0;
        buffer.rgb[current.y * buffer.width + current.x][1] = pt_new_value_1;
        buffer.rgb[current.y * buffer.width + current.x][2] = pt_new_value_2;

        mask.put(current.x, current.y, 0.0);

        // INFO: you can deploy your new found if-let tools here.
        match find_next_point(&mask, current.x as i32, current.y as i32) {
            Some(pt) => current = pt.to_owned(),
            None => break,
        }
    }
}

fn rgb_image_to_vec(rgb: &RgbImage) -> error::Result<RgbVec> {
    let mut v: Vec<[f32; 3]> = Vec::with_capacity(rgb.width * rgb.height);
    v.resize(rgb.width * rgb.height, [0.0, 0.0, 0.0]);

    //INFO: you can deploy your new .iter().XYZ_map() skills here.
    //.filter_map() is great when you want to deal with Option<T>, i.e keeping the None or Err
    //variants.
    //.flat_map() is great when you're wanting to cast away all the None, Err variants.
    for y in 0..rgb.height {
        for x in 0..rgb.width {
            let idx = y * rgb.width + x;
            let r = match rgb.get_band(0).get(x, y) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };
            let g = match rgb.get_band(1).get(x, y) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };
            let b = match rgb.get_band(2).get(x, y) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };
            v[idx][0] = r;
            v[idx][1] = g;
            v[idx][2] = b;
        }
    }

    Ok(RgbVec {
        rgb: v,
        width: rgb.width,
        height: rgb.height,
    })
}

fn vec_to_rgb_image(buffer: &RgbVec) -> error::Result<RgbImage> {
    //INFO: unwrap is poor form, esp on constructors.
    //There is a cool library/crate called infallable that will guaratnee that your code cannot
    //panic by checking stuff via the compiler, i've seen people using that with .unwrap() in
    //increasing number of late.
    let mut red = ImageBuffer::new(buffer.width, buffer.height).unwrap();
    let mut green = ImageBuffer::new(buffer.width, buffer.height).unwrap();
    let mut blue = ImageBuffer::new(buffer.width, buffer.height).unwrap();

    //INFO: try .iter()
    for y in 0..buffer.height {
        for x in 0..buffer.width {
            let r = buffer.rgb[y * (buffer.width) + x][0];
            let g = buffer.rgb[y * (buffer.width) + x][1];
            let b = buffer.rgb[y * (buffer.width) + x][2];
            red.put(x, y, r);
            green.put(x, y, g);
            blue.put(x, y, b);
        }
    }

    RgbImage::new_from_buffers_rgb(&red, &green, &blue, enums::ImageMode::U8BIT)
}

// Embarrassingly slow and inefficient. Runs slow in debug. A lot faster with a release build.
// INFO: if you're interested in making this faster, maybe there exists a more sophisticated
// implementation? if this is *the* or close to *the* best possible (single-threaded) implementation,
// I'm happy to revisit it if you want to take another crack using some of the suggestions
// contained herein.
pub fn apply_inpaint_to_buffer_with_mask(
    rgb: &RgbImage,
    mask_src: &ImageBuffer,
) -> error::Result<RgbImage> {
    let mut working_buffer = match rgb_image_to_vec(&rgb) {
        Ok(b) => b,
        Err(e) => return Err(e),
    };

    let mut mask = mask_src.clone();

    // Crop the mask image if it's larger than the input image.
    // Sizes need to match
    if mask.width > working_buffer.width {
        let x = (mask.width - working_buffer.width) / 2;
        let y = (mask.height - working_buffer.height) / 2;
        mask = match mask.get_subframe(x, y, working_buffer.width, working_buffer.height) {
            Ok(m) => m,
            Err(_) => return Err("Error subframing mask"),
        }
    }

    // For this to work, we need the mask to be mutable and we're
    // going to fill it in with 0x0 values as we go. If we don't, then
    // we'll keep finding starting points and this will be an infinite
    // loop. Which is bad. Perhaps consider an alternate method here.
    loop {
        // TODO: Don't leave embedded match statements. I hate that as much as embedded case statements...
        // INFO: for a two arm match most would advise an if-let destructuring. (esp if you hate match
        // statements so...)
        if let Some(pt) = find_starting_point(&mask) {
            infill(&mut working_buffer, &mut mask, &pt);
        } else {
            break;
        }
    }

    // Already returning the type you've indicated, so this doesn't need matching out at all...
    vec_to_rgb_image(&working_buffer)
}

pub fn apply_inpaint_to_buffer(rgb: &RgbImage, mask: &ImageBuffer) -> error::Result<RgbImage> {
    apply_inpaint_to_buffer_with_mask(&rgb, &mask)
}

pub fn make_mask_from_red(rgbimage: &RgbImage) -> error::Result<ImageBuffer> {
    let mut new_mask = match ImageBuffer::new(rgbimage.width, rgbimage.height) {
        Ok(b) => b,
        Err(e) => return Err(e),
    };

    // INFO:
    // Consider rust's iterators for this sort of thing.. you may find them very pleasant to
    // work with. SEE ABOVE INFO:s
    for y in 0..rgbimage.height {
        for x in 0..rgbimage.width {
            let r = rgbimage.get_band(0).get(x, y).unwrap();
            let g = rgbimage.get_band(1).get(x, y).unwrap();
            let b = rgbimage.get_band(2).get(x, y).unwrap();

            // if r != g || r != b || g != b {
            //     new_mask.put(x, y, 255.0).unwrap();
            // }
            if r == 255.0 && g == 0.0 && b == 0.0 {
                new_mask.put(x, y, 255.0);
            }
        }
    }

    Ok(new_mask)
}
