use std::{
    io::{Read, Error as IOError},
    fs::File,
    path::Path
};

pub struct FileReader {
    file: File,
}

#[derive(Debug)]
pub enum FileReaderError {
    UnexpectedEOF
}

#[allow(dead_code)]
impl FileReader {
    pub fn new(f: File) -> Self {
        FileReader {
            file: f
        }
    }

    pub fn from_path(path: &Path) -> Result<Self, IOError> {
        let file = File::open(path)?;
        Ok(Self {
            file: file
        })
    }

    pub fn from_name(name: &str) -> Result<Self, IOError> {
        Self::from_path(Path::new(name))
    }

    pub fn read_u8(&mut self) -> Result<u8, FileReaderError> {
        let mut buffer = [0_u8; 1]; 
        self.file.read(&mut buffer)?;
        Ok(buffer[0])
    }

    pub fn read_u16(&mut self) -> Result<u16, FileReaderError> {
        let mut buffer = [0_u8; 2]; 
        self.file.read(&mut buffer)?;
        Ok(u16::from_be_bytes(buffer))
    }

    pub fn read_u32(&mut self) -> Result<u32, FileReaderError> {
        let mut buffer = [0_u8; 4]; 
        self.file.read(&mut buffer)?;
        Ok(u32::from_be_bytes(buffer))
    }

    pub fn read_i8(&mut self) -> Result<i8, FileReaderError> {
        let mut buffer = [0_u8; 1]; 
        self.file.read(&mut buffer)?;
        Ok(i8::from_be_bytes(buffer))
    }

    pub fn read_i16(&mut self) -> Result<i16, FileReaderError> {
        let mut buffer = [0_u8; 2]; 
        self.file.read(&mut buffer)?;
        Ok(i16::from_be_bytes(buffer))
    }

    pub fn read_i32(&mut self) -> Result<i32, FileReaderError> {
        let mut buffer = [0_u8; 4]; 
        self.file.read(&mut buffer)?;
        Ok(i32::from_be_bytes(buffer))
    }

    pub fn read_f32(&mut self) -> Result<f32, FileReaderError> {
        let mut buffer = [0_u8; 4]; 
        self.file.read(&mut buffer)?;
        Ok(f32::from_be_bytes(buffer))
    }

    pub fn read_f64(&mut self) -> Result<f64, FileReaderError> {
        let mut buffer = [0_u8; 8]; 
        self.file.read(&mut buffer)?;
        Ok(f64::from_be_bytes(buffer))
    }
}

impl std::convert::From<IOError> for FileReaderError {
    fn from(_: IOError) -> Self {
        FileReaderError::UnexpectedEOF
    }
}
