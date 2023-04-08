use crate::path;

use std::str::FromStr;
use string_builder::Builder;

pub fn string_is_valid_num<T: FromStr>(s: &str) -> bool {
    let num = s.parse::<T>();
    num.is_ok()
}

pub fn string_is_valid_f64(s: &str) -> bool {
    string_is_valid_num::<f64>(s)
}

pub fn string_is_valid_f32(s: &str) -> bool {
    string_is_valid_num::<f32>(s)
}

pub fn string_is_valid_i32(s: &str) -> bool {
    string_is_valid_num::<i32>(s)
}

pub fn string_is_valid_u16(s: &str) -> bool {
    string_is_valid_num::<u16>(s)
}

pub fn filename_char_at_pos(filename: &str, pos: usize) -> char {
    let bn = path::basename(filename);
    bn.chars().nth(pos).unwrap()
}

#[macro_export]
macro_rules! max {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => {{
        let y = max!($($z),*);
        if $x > y {
            $x
        } else {
            y
        }
    }}
}

#[macro_export]
macro_rules! min {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => {{
        let y = min!($($z),*);
        if $x < y {
            $x
        } else {
            y
        }
    }}
}

pub fn stringvec(a: &str, b: &str) -> Vec<String> {
    vec![a.to_owned(), b.to_owned()]
}

pub fn stringvec_b(a: &str, b: String) -> Vec<String> {
    vec![a.to_owned(), b]
}

pub fn image_exists_on_filesystem(image_url: &str) -> bool {
    let bn = path::basename(image_url);
    path::file_exists(bn.as_str())
}

pub fn append_file_name(input_file: &str, append: &str) -> String {
    let append_with_ext = format!("-{}.png", append);
    replace_image_extension(input_file, append_with_ext.as_str())
}

pub fn replace_image_extension(input_file: &str, append: &str) -> String {
    input_file
        .replace(".png", append)
        .replace(".PNG", append)
        .replace(".jpg", append)
        .replace(".JPG", append)
        .replace(".tif", append)
        .replace(".TIF", append)
}

pub fn vec_to_str(v: &[f64]) -> String {
    let mut b = Builder::default();

    v.into_iter().for_each(|item| {
        b.append(format!("{},", item));
    });

    let mut s = b.string().unwrap();
    if !s.is_empty() {
        s.remove(s.len() - 1);
    }

    format!("({})", s)
}
