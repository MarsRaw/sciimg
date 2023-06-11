use memmap::Mmap;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Endian {
    BigEndian = 1,
    LittleEndian = 0,
    NativeEndian = 100,
}

impl Endian {
    pub fn from_i32(v: i32) -> Endian {
        match v {
            1 => Endian::BigEndian,
            0 => Endian::LittleEndian,
            100 => Endian::NativeEndian,
            _ => panic!("Invalid endian enum value"),
        }
    }
}

macro_rules! bytes_to_primitive {
    ($bytes:expr, $type:ident, $endian:expr) => {
        match $endian {
            Endian::BigEndian => $type::from_be_bytes($bytes),
            Endian::LittleEndian => $type::from_le_bytes($bytes),
            Endian::NativeEndian => $type::from_ne_bytes($bytes),
        }
    };
}

#[derive(Debug)]
pub struct BinFileReader {
    file_ptr: File,
    map: Mmap,
    file_path: PathBuf,
    endiness: Endian,
}

/// A strongly (over)simplified means of reading a file directly into primitive types. Wraps around a memory mapped file pointer
impl BinFileReader {
    pub fn new<P>(file_path: P) -> BinFileReader
    where
        P: AsRef<Path> + Copy,
        PathBuf: From<P>,
    {
        BinFileReader::new_as_endiness(file_path, Endian::LittleEndian)
    }

    pub fn new_as_endiness<P>(file_path: P, endiness: Endian) -> BinFileReader
    where
        P: AsRef<Path> + Copy,
        PathBuf: From<P>,
    {
        let file_ptr = File::open(file_path).expect("Error opening file");
        let map: Mmap = unsafe { Mmap::map(&file_ptr).expect("Error creating memory map") };

        BinFileReader {
            file_ptr,
            map,
            file_path: file_path.as_ref().into(),
            endiness,
        }
    }

    pub fn set_endiness(&mut self, endiness: Endian) {
        self.endiness = endiness;
    }

    pub fn read_string(&self, start: usize, len: usize) -> String {
        let v: Vec<u8> = self.map[start..(start + len)].to_vec();
        String::from_utf8(v).expect("Failed reading string value")
    }

    fn read_bytes_raw_16byte(&self, start: usize) -> [u8; 16] {
        self.map[start..(start + 16)]
            .try_into()
            .expect("slice with incorrect length")
    }

    fn read_bytes_raw_8byte(&self, start: usize) -> [u8; 8] {
        self.map[start..(start + 8)]
            .try_into()
            .expect("slice with incorrect length")
    }

    fn read_bytes_raw_4byte(&self, start: usize) -> [u8; 4] {
        self.map[start..(start + 4)]
            .try_into()
            .expect("slice with incorrect length")
    }

    fn read_bytes_raw_2byte(&self, start: usize) -> [u8; 2] {
        self.map[start..(start + 2)]
            .try_into()
            .expect("slice with incorrect length")
    }

    fn read_bytes_raw_1byte(&self, start: usize) -> [u8; 1] {
        self.map[start..(start + 1)]
            .try_into()
            .expect("slice with incorrect length")
    }

    pub fn read_i128_with_endiness(&self, start: usize, endiness: Endian) -> i128 {
        bytes_to_primitive!(self.read_bytes_raw_16byte(start), i128, endiness)
    }

    pub fn read_i128(&self, start: usize) -> i128 {
        self.read_i128_with_endiness(start, self.endiness)
    }

    pub fn read_u128_with_endiness(&self, start: usize, endiness: Endian) -> u128 {
        bytes_to_primitive!(self.read_bytes_raw_16byte(start), u128, endiness)
    }

    pub fn read_u128(&self, start: usize) -> u128 {
        self.read_u128_with_endiness(start, self.endiness)
    }

    pub fn read_i64_with_endiness(&self, start: usize, endiness: Endian) -> i64 {
        bytes_to_primitive!(self.read_bytes_raw_8byte(start), i64, endiness)
    }

    pub fn read_i64(&self, start: usize) -> i64 {
        self.read_i64_with_endiness(start, self.endiness)
    }

    pub fn read_u64_with_endiness(&self, start: usize, endiness: Endian) -> u64 {
        bytes_to_primitive!(self.read_bytes_raw_8byte(start), u64, endiness)
    }

    pub fn read_u64(&self, start: usize) -> u64 {
        self.read_u64_with_endiness(start, self.endiness)
    }

    pub fn read_f64_with_endiness(&self, start: usize, endiness: Endian) -> f64 {
        bytes_to_primitive!(self.read_bytes_raw_8byte(start), f64, endiness)
    }

    pub fn read_f64(&self, start: usize) -> f64 {
        self.read_f64_with_endiness(start, self.endiness)
    }

    pub fn read_i32_with_endiness(&self, start: usize, endiness: Endian) -> i32 {
        bytes_to_primitive!(self.read_bytes_raw_4byte(start), i32, endiness)
    }

    pub fn read_i32(&self, start: usize) -> i32 {
        self.read_i32_with_endiness(start, self.endiness)
    }

    pub fn read_u32_with_endiness(&self, start: usize, endiness: Endian) -> u32 {
        bytes_to_primitive!(self.read_bytes_raw_4byte(start), u32, endiness)
    }

    pub fn read_u32(&self, start: usize) -> u32 {
        self.read_u32_with_endiness(start, self.endiness)
    }

    pub fn read_i16_with_endiness(&self, start: usize, endiness: Endian) -> i16 {
        bytes_to_primitive!(self.read_bytes_raw_2byte(start), i16, endiness)
    }

    pub fn read_i16(&self, start: usize) -> i16 {
        self.read_i16_with_endiness(start, self.endiness)
    }

    pub fn read_u16_with_endiness(&self, start: usize, endiness: Endian) -> u16 {
        bytes_to_primitive!(self.read_bytes_raw_2byte(start), u16, endiness)
    }

    pub fn read_u16(&self, start: usize) -> u16 {
        self.read_u16_with_endiness(start, self.endiness)
    }

    pub fn read_f32_with_endiness(&self, start: usize, endiness: Endian) -> f32 {
        bytes_to_primitive!(self.read_bytes_raw_4byte(start), f32, endiness)
    }

    pub fn read_f32(&self, start: usize) -> f32 {
        self.read_f32_with_endiness(start, self.endiness)
    }

    pub fn read_i8_with_endiness(&self, start: usize, endiness: Endian) -> i8 {
        bytes_to_primitive!(self.read_bytes_raw_1byte(start), i8, endiness)
    }

    pub fn read_i8(&self, start: usize) -> i8 {
        self.read_i8_with_endiness(start, self.endiness)
    }

    pub fn read_u8_with_endiness(&self, start: usize, endiness: Endian) -> u8 {
        bytes_to_primitive!(self.read_bytes_raw_1byte(start), u8, endiness)
    }

    pub fn read_u8(&self, start: usize) -> u8 {
        self.read_u8_with_endiness(start, self.endiness)
    }

    pub fn read_bytes(&self, start: usize, length: usize) -> Vec<u8> {
        self.map[start..(start + length)].to_vec()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn source_file_path(&self) -> PathBuf {
        self.file_path.clone()
    }

    pub fn file_pointer_clone(&self) -> Result<File, io::Error> {
        self.file_ptr.try_clone()
    }
}
