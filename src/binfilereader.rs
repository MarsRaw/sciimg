use anyhow::{anyhow, Result};
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
    pub fn from_i32(v: i32) -> Result<Endian> {
        match v {
            1 => Ok(Endian::BigEndian),
            0 => Ok(Endian::LittleEndian),
            100 => Ok(Endian::NativeEndian),
            _ => Err(anyhow!("Invalid endian enum value: {}", v)),
        }
    }
}

macro_rules! bytes_to_primitive {
    ($bytes_res:expr, $type:ident, $endian:expr) => {
        if let Ok(bytes) = $bytes_res {
            match $endian {
                Endian::BigEndian => Ok($type::from_be_bytes(bytes)),
                Endian::LittleEndian => Ok($type::from_le_bytes(bytes)),
                Endian::NativeEndian => Ok($type::from_ne_bytes(bytes)),
            }
        } else {
            Err(anyhow!("Failed to convert to primitive type"))
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

    pub fn read_string(&self, start: usize, len: usize) -> Result<String> {
        let v: Vec<u8> = self.map[start..(start + len)].to_vec();
        match String::from_utf8_lossy(&v) {
            std::borrow::Cow::Borrowed(v) => Ok(v.to_string()),
            std::borrow::Cow::Owned(v) => Ok(v),
        }
    }

    fn read_bytes_raw_16byte(&self, start: usize) -> Result<[u8; 16]> {
        match self.map[start..(start + 16)].try_into() {
            Ok(v) => Ok(v),
            Err(why) => Err(anyhow!("Unable to read bytes: {}", why)),
        }
    }

    fn read_bytes_raw_8byte(&self, start: usize) -> Result<[u8; 8]> {
        match self.map[start..(start + 8)].try_into() {
            Ok(v) => Ok(v),
            Err(why) => Err(anyhow!("Unable to read bytes: {}", why)),
        }
    }

    fn read_bytes_raw_4byte(&self, start: usize) -> Result<[u8; 4]> {
        match self.map[start..(start + 4)].try_into() {
            Ok(v) => Ok(v),
            Err(why) => Err(anyhow!("Unable to read bytes: {}", why)),
        }
    }

    fn read_bytes_raw_2byte(&self, start: usize) -> Result<[u8; 2]> {
        match self.map[start..(start + 2)].try_into() {
            Ok(v) => Ok(v),
            Err(why) => Err(anyhow!("Unable to read bytes: {}", why)),
        }
    }

    fn read_bytes_raw_1byte(&self, start: usize) -> Result<[u8; 1]> {
        match self.map[start..(start + 1)].try_into() {
            Ok(v) => Ok(v),
            Err(why) => Err(anyhow!("Unable to read bytes: {}", why)),
        }
    }

    pub fn read_i128_with_endiness(&self, start: usize, endiness: Endian) -> Result<i128> {
        bytes_to_primitive!(self.read_bytes_raw_16byte(start), i128, endiness)
    }

    pub fn read_i128(&self, start: usize) -> Result<i128> {
        self.read_i128_with_endiness(start, self.endiness)
    }

    pub fn read_u128_with_endiness(&self, start: usize, endiness: Endian) -> Result<u128> {
        bytes_to_primitive!(self.read_bytes_raw_16byte(start), u128, endiness)
    }

    pub fn read_u128(&self, start: usize) -> Result<u128> {
        self.read_u128_with_endiness(start, self.endiness)
    }

    pub fn read_i64_with_endiness(&self, start: usize, endiness: Endian) -> Result<i64> {
        bytes_to_primitive!(self.read_bytes_raw_8byte(start), i64, endiness)
    }

    pub fn read_i64(&self, start: usize) -> Result<i64> {
        self.read_i64_with_endiness(start, self.endiness)
    }

    pub fn read_u64_with_endiness(&self, start: usize, endiness: Endian) -> Result<u64> {
        bytes_to_primitive!(self.read_bytes_raw_8byte(start), u64, endiness)
    }

    pub fn read_u64(&self, start: usize) -> Result<u64> {
        self.read_u64_with_endiness(start, self.endiness)
    }

    pub fn read_f64_with_endiness(&self, start: usize, endiness: Endian) -> Result<f64> {
        bytes_to_primitive!(self.read_bytes_raw_8byte(start), f64, endiness)
    }

    pub fn read_f64(&self, start: usize) -> Result<f64> {
        self.read_f64_with_endiness(start, self.endiness)
    }

    pub fn read_i32_with_endiness(&self, start: usize, endiness: Endian) -> Result<i32> {
        bytes_to_primitive!(self.read_bytes_raw_4byte(start), i32, endiness)
    }

    pub fn read_i32(&self, start: usize) -> Result<i32> {
        self.read_i32_with_endiness(start, self.endiness)
    }

    pub fn read_u32_with_endiness(&self, start: usize, endiness: Endian) -> Result<u32> {
        bytes_to_primitive!(self.read_bytes_raw_4byte(start), u32, endiness)
    }

    pub fn read_u32(&self, start: usize) -> Result<u32> {
        self.read_u32_with_endiness(start, self.endiness)
    }

    pub fn read_i16_with_endiness(&self, start: usize, endiness: Endian) -> Result<i16> {
        bytes_to_primitive!(self.read_bytes_raw_2byte(start), i16, endiness)
    }

    pub fn read_i16(&self, start: usize) -> Result<i16> {
        self.read_i16_with_endiness(start, self.endiness)
    }

    pub fn read_u16_with_endiness(&self, start: usize, endiness: Endian) -> Result<u16> {
        bytes_to_primitive!(self.read_bytes_raw_2byte(start), u16, endiness)
    }

    pub fn read_u16(&self, start: usize) -> Result<u16> {
        self.read_u16_with_endiness(start, self.endiness)
    }

    pub fn read_f32_with_endiness(&self, start: usize, endiness: Endian) -> Result<f32> {
        bytes_to_primitive!(self.read_bytes_raw_4byte(start), f32, endiness)
    }

    pub fn read_f32(&self, start: usize) -> Result<f32> {
        self.read_f32_with_endiness(start, self.endiness)
    }

    pub fn read_i8_with_endiness(&self, start: usize, endiness: Endian) -> Result<i8> {
        bytes_to_primitive!(self.read_bytes_raw_1byte(start), i8, endiness)
    }

    pub fn read_i8(&self, start: usize) -> Result<i8> {
        self.read_i8_with_endiness(start, self.endiness)
    }

    pub fn read_u8_with_endiness(&self, start: usize, endiness: Endian) -> Result<u8> {
        bytes_to_primitive!(self.read_bytes_raw_1byte(start), u8, endiness)
    }

    pub fn read_u8(&self, start: usize) -> Result<u8> {
        self.read_u8_with_endiness(start, self.endiness)
    }

    pub fn read_bytes(&self, start: usize, length: usize) -> Result<Vec<u8>> {
        Ok(self.map[start..(start + length)].to_vec())
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
