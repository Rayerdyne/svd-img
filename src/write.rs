use std::{
    fs::File,
    io::{
        prelude::Write,
        Error as IOError,
    }
};

pub struct FileWriter {
    file: File
}

#[allow(dead_code)]
impl FileWriter {
    fn new(f: File) -> Self {
        Self {
            file: f
        }
    }    

    fn write_u8(&mut self, x: u8) -> Result<(), IOError> {
        self.file.write_all(&x.to_be_bytes())?;
        Ok(())
    }

    fn write_u16(&mut self, x: u16) -> Result<(), IOError> {
        self.file.write_all(&x.to_be_bytes())?;
        Ok(())
    }

    fn write_u32(&mut self, x: u32) -> Result<(), IOError> {
        self.file.write_all(&x.to_be_bytes())?;
        Ok(())
    }

    fn write_i8(&mut self, x: i8) -> Result<(), IOError> {
        self.file.write_all(&x.to_be_bytes())?;
        Ok(())
    }

    fn write_i16(&mut self, x: i16) -> Result<(), IOError> {
        self.file.write_all(&x.to_be_bytes())?;
        Ok(())
    }

    fn write_i32(&mut self, x: i32) -> Result<(), IOError> {
        self.file.write_all(&x.to_be_bytes())?;
        Ok(())
    }

    fn write_f32(&mut self, x: f32) -> Result<(), IOError> {
        self.file.write_all(&x.to_be_bytes())?;
        Ok(())
    }

    fn write_f64(&mut self, x: f64) -> Result<(), IOError> {
        self.file.write_all(&x.to_be_bytes())?;
        Ok(())
    }
}