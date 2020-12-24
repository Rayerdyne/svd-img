use super::{
    Error,
    write::FileWriter
};

use std::{
    fs::{self, File},
    path::Path,
};

use image::{
    io::Reader as ImageReader,
    DynamicImage,
    Rgb, Rgba,
    RgbImage, RgbaImage
};

use wav::{
    Header as WavHeader,
    BitDepth as WavData,
};

use nalgebra::{ DMatrix, DVector, Scalar };

pub type SVDVectors<T> = Vec<(T, DVector<T>, DVector<T>)>;
pub trait Printable {
    fn to_string(&self) -> String;
}

pub struct Options {
    pub policy: CompressionPolicy,
    pub use_f64: bool,
    pub eps: f32,
    pub n_iter: usize,
    pub original_file_size: u64,

    pub use_aggregate: bool,
    pub with_alpha: bool,

    pub is_wav: bool, 
    pub is_reduce: bool,
    pub bits_per_sample: Option<u16>
}

pub enum CompressionPolicy {
    Number(usize),
    Ratio(u8)
}

/// Encodes image file in `input` to vector file `output`, with given 
/// `options`.
pub fn encode(input: &str, output: &str, options: &mut Options) 
    -> Result<(), Error> {

    options.is_wav |= input.ends_with(".WAV") ||
                      input.ends_with(".wav");

    let metadata = fs::metadata(Path::new(input))?;
    options.original_file_size = metadata.len();

    let (matrix, header) = if !options.is_wav {
        let img = read_image_file(input)?;
        if options.with_alpha {
            (image_matrix_rgba(img.into_rgba8(), options.use_aggregate), None)
        } else {
            (image_matrix_rgb(img.into_rgb8(), options.use_aggregate), None)
        }
        // let xx = img.into_rgba8();

        // (image_matrix(xx), None)
    } else {
        let mut in_file = File::open(Path::new(input))?;
        let (header_small, sound_data) = wav::read(&mut in_file)?;
        options.bits_per_sample = Some(header_small.bits_per_sample);
        let n = match &sound_data {
            WavData::Eight(x) => x.len(),
            WavData::Sixteen(x) => x.len(),
            WavData::TwentyFour(x) => x.len(),
            _ => 0,
        };

        (sound_matrix(&sound_data).unwrap(), Some((header_small, n as u32)))
    };

    let mut fw = FileWriter::from_name(output)?;

    if options.use_f64 {
        let vectors: SVDVectors<f64> = matrix_reduce_f64(&matrix, options)?;
        // println!("{}", vectors.to_string());
        write_vectors_header(&mut fw, &vectors, &options, header)?;
        write_vectors_f64(&mut fw, &vectors)?;
    }
    else {
        let vectors: SVDVectors<f32> = matrix_reduce_f32(&matrix, options)?;
        // println!("{}", vectors.to_string());
        write_vectors_header(&mut fw, &vectors, &options, header)?;
        write_vectors_f32(&mut fw, &vectors)?;
    }

    Ok(())
}

pub fn read_image_file(name: &str) -> Result<DynamicImage, Error> {
    match ImageReader::open(name) {
        Ok(img_data) => match img_data.decode() {
            Ok(img) => Ok(img),
            Err(_)  => return Err(Error::ImageFormatError)
        },
        Err(_) => return Err(Error::ImageReadError)
    }
}

/// Returns a DMatrix<i32> containing the data of the Rgba image.
fn image_matrix_rgba(img: RgbaImage, aggregate: bool) -> DMatrix<i32> {
    let dim = img.dimensions();

    if aggregate {
        DMatrix::from_fn(dim.0 as usize, dim.1 as usize, |i, j| {
            i32_from_rgba_pixel(img[(i as u32, j as u32)])
            // let pixel = img[(i as u32, j as u32)];
            // (pixel[0] as i32) << 24_i32 + 
            // (pixel[1] as i32) << 16_i32 +
            // (pixel[2] as i32) << 8_i32  +
            // pixel[3] as i32
        })
    } else {
        let mut a = DMatrix::<i32>::zeros(dim.0 as usize * 2, dim.1 as usize * 2);
        for i in 0..(dim.0 as usize) {
            for j in 0..(dim.1 as usize) {
                let pixel = img[(i as u32, j as u32)];
                a[(2*i,   2*j  )] = pixel[0] as i32;
                a[(2*i+1, 2*j  )] = pixel[1] as i32;
                a[(2*i,   2*j+1)] = pixel[2] as i32;
                a[(2*i+1, 2*j+1)] = pixel[3] as i32;
            }
        }
        a
    }
}

/// Returns a DMatrix<i32> containing the data of the Rgb image.
fn image_matrix_rgb(img: RgbImage, aggregate: bool) -> DMatrix<i32> {
    let dim = img.dimensions();

    if aggregate {
        DMatrix::from_fn(dim.0 as usize, dim.1 as usize, |i, j| {
            i32_from_rgb_pixel(img[(i as u32, j as u32)])
            // let pixel = img[(i as u32, j as u32)];
            // (pixel[0] as i32) << 16_i32 + 
            // (pixel[1] as i32) << 8_i32 +
            // pixel[2] as i32
        })
    } else {
        let mut a = DMatrix::<i32>::zeros(dim.0 as usize * 2, dim.1 as usize * 2);
        for i in 0..(dim.0 as usize) {
            for j in 0..(dim.1 as usize) {
                let pixel = img[(i as u32, j as u32)];
                a[(2*i,   2*j  )] = pixel[0] as i32;
                a[(2*i+1, 2*j  )] = pixel[1] as i32;
                a[(2*i,   2*j+1)] = pixel[2] as i32;
                a[(2*i+1, 2*j+1)] = 0xff_i32;
            }
        }
        a
    }
}

fn sound_matrix(data: &WavData) -> Option<DMatrix<i32>>
    {

    match data {
        WavData::Eight(d) => Some(matrix_from_sound_data(&d)),
        WavData::Sixteen(d) => Some(matrix_from_sound_data(&d)),
        WavData::TwentyFour(d) => Some(matrix_from_sound_data(&d)),
        _ => None
    }
}

fn matrix_from_sound_data<T>(data: &[T]) -> DMatrix<i32>
    where T: Scalar + Into<i32> + Copy
    {

    let n = data.len();
    let rows = (n as f64).sqrt().round() as usize;
    let cols = (n as f64 / rows as f64).ceil() as usize;

    DMatrix::from_fn(rows, cols, |i, j| {
        if i * rows + j < n {
            data[i * rows + j].into()
        } else { 0 }
    })
}

fn matrix_reduce_f64<T>(matrix: &DMatrix<T>, options: &Options)
    -> Result<SVDVectors<f64>, Error>
    where T: Scalar + Into<f64> + Copy
    {
    
    let (h, w) = matrix.shape();
    let n = options.n_with(h, w)?;

    let m2 = DMatrix::from_fn(h, w, |i, j| matrix[(i, j)].into());

    let svd = match m2.try_svd(true, true, options.eps.into(), 0) {
        Some(x) => x,
        None => return Err(Error::SVDError)
        
    };

    let u = match svd.u {
        Some(u) => u,
        None => return Err(Error::NoSVDResult)
    };

    let v_t = match svd.v_t {
        Some(v_t) => v_t,
        None => return Err(Error::NoSVDResult)
    };

    let sv = svd.singular_values;

    let mut res = Vec::with_capacity(n);
    for i in 0..n {
        let sv_i = sv[i];
        let u_i = DVector::from_column_slice(u.column(i).as_slice());
        let v_t_i= DVector::from_fn(w, |x, _| {  v_t[(i, x)]  });
        res.push((sv_i, u_i, v_t_i));
    }

    Ok(res)
}

fn matrix_reduce_f32(matrix: &DMatrix<i32>, options: &Options)
    -> Result<SVDVectors<f32>, Error>
    {
    
    let (h, w) = matrix.shape();
    let n = options.n_with(h, w)?;

    let m2 = DMatrix::from_fn(h, w, |i, j| 
        f32_from_i32_bad(matrix[(i, j)])
    );

    let svd = match m2.try_svd(true, true, options.eps, 0) {
        Some(x) => x,
        None => return Err(Error::SVDError)
        
    };

    let u = match svd.u {
        Some(u) => u,
        None => return Err(Error::NoSVDResult)
    };

    let v_t = match svd.v_t {
        Some(v_t) => v_t,
        None => return Err(Error::NoSVDResult)
    };

    let sv = svd.singular_values;

    let mut res = Vec::with_capacity(n);
    for i in 0..n {
        let sv_i = sv[i];
        let u_i = DVector::<f32>::from_column_slice(u.column(i).as_slice());
        let v_t_i= DVector::<f32>::from_fn(w, |x, _| {  v_t[(i, x)]  });
        res.push((sv_i, u_i, v_t_i));
    }

    Ok(res)
}

pub(crate) fn write_vectors_header<T>(fw: &mut FileWriter, vectors: &SVDVectors<T>,
    options: &Options, header: Option<(WavHeader, u32)>) -> Result<(), Error> 
    where T: std::fmt::Debug + nalgebra::Scalar
    {
    let n = vectors.len();
    let height = vectors[0].1.nrows();
    let width = vectors[0].2.nrows();

    let file_type = if options.is_wav        { 0x8 } else { 0x0 } |
                    if options.use_f64       { 0x4 } else { 0x0 } |
                    if options.with_alpha    { 0x2 } else { 0x0 } |
                    if options.use_aggregate { 0x1 } else { 0x0 };
    fw.write_u8(file_type)?;

    if options.is_wav {
        let h = header.unwrap();
        let x: [u8; 16] = h.0.into();
        fw.write_all(&x)?;
        fw.write_u32(h.1)?;
    }

    fw.write_u32(n as u32)?;
    fw.write_u32(height as u32)?;
    fw.write_u32(width as u32)?;

    Ok(())
}

pub(crate) fn write_vectors_f64(fw: &mut FileWriter, vectors: &SVDVectors<f64>)
    -> Result<(), Error> {

    let n = vectors.len();
    let height = vectors[0].1.nrows();
    let width = vectors[0].2.nrows();

    for i in 0..n {
        let triplet = &vectors[i];
        fw.write_f64(triplet.0)?;
        for j in 0..height 
            { fw.write_f64(triplet.1[j])?; }
        for j in 0..width 
            { fw.write_f64(triplet.2[j])?; }
    }

    Ok(())
}

pub(crate) fn write_vectors_f32(fw: &mut FileWriter, vectors: &SVDVectors<f32>)
    -> Result<(), Error> {

    let n = vectors.len();
    let height = vectors[0].1.nrows();
    let width = vectors[0].2.nrows();

    for i in 0..n {
        let triplet = &vectors[i];
        fw.write_f32(triplet.0)?;
        for j in 0..height 
            { fw.write_f32(triplet.1[j])?; }
        for j in 0..width 
            { fw.write_f32(triplet.2[j])?; }
    }

    Ok(())
}

impl CompressionPolicy {
    pub fn with_number(n: usize) -> Self {
        CompressionPolicy::Number(n)
    }
    pub fn with_ratio_percentage(r: u8) -> Self {
        CompressionPolicy::Ratio(r)
    }
}

impl Options {
    pub fn default() -> Self {
        Options {
            eps: 1.0e-5,
            policy: CompressionPolicy::with_ratio_percentage(25),
            use_f64: true,
            n_iter: 0,
            original_file_size: 0,

            use_aggregate: true,
            with_alpha: false,

            is_wav: false,
            bits_per_sample: None,
            is_reduce: false
        }
    }

    pub (crate) fn n_with(&self, h: usize, w: usize) -> Result<usize, Error> {
        println!("Original file size: {}", self.original_file_size);

        match self.policy {
            CompressionPolicy::Number(n) => {
                if n <= 0 {  return Err(Error::NTooSmall);  }
                Ok(n)
            },
            CompressionPolicy::Ratio(r) => {
                let data_bits = if self.use_f64 {  8_f64  }
                                else            {  4_f64  };
                let r_f64 = r as f64 / 100_f64;

                let vector_size = data_bits * (1_f64 + h as f64 + w as f64);
                // n * vector_size = r_f64 * img_size
                let n = (r_f64 * self.original_file_size as f64) / vector_size;
                if n.round() <= 0.0 {  return Err(Error::RatioTooRestrictive);  }
                println!("Output file size: {}", n * vector_size);
                Ok(n.round() as usize)
            }
        }
    }
}

impl<T> Printable for SVDVectors<T> 
    where T: Scalar + std::fmt::Display {
        fn to_string(&self) -> String {
            let mut s = String::new();
            for (sigma, u, v_t) in self {
                s.push_str(&format!("\\sigma: {}\nu: {}\nv_t: {}\n", sigma, u, v_t));
            }
            s
    }
}

/// Before crying, please consider that wav files will have 24 bits encoding so
/// that it will be OK, as there are 23 bits of fractionnal part, plus the
/// first one that will always be 1 (cf floating point standards)
fn f32_from_i32_bad(x: i32) -> f32 {
    let mut r: f32 = 0.0;
    let y = if x < 0 { -x } else { x };

    // f32 has 23 bits fraction part
    for i in 0..23 {
        let cur_i32: i32 = 2_i32.pow(i);
        let cur_f32: f32 = 2_f32.powi(i as i32);
        if y & cur_i32 != 0 {
            r += cur_f32;   
        }
    }

    if x < 0 { -r } else { r }
}

pub(crate) fn i32_from_rgba_pixel(p: Rgba<u8>) -> i32 {
    let mut r = 0_i32;
    for i in 0..8 {
        let x = 1 << i;
        let i4 = 4 * i;
        let y = 1 << i4;
        r |= if p[0] & x != 0 { y << 3 } else { 0 };
        r |= if p[1] & x != 0 { y << 2 } else { 0 };
        r |= if p[2] & x != 0 { y << 1 } else { 0 };
        r |= if p[3] & x != 0 { y      } else { 0 };
    }
    r
}

pub(crate) fn i32_from_rgb_pixel(p: Rgb<u8>) -> i32 {
    let mut r = 0_i32;
    for i in 0..8 {
        let x = 1 << i;
        let i3 = 3 * i;
        let y = 1 << i3 as i32;
        r |= if p[0] & x != 0 { y << 2 } else { 0 };
        r |= if p[1] & x != 0 { y << 1 } else { 0 };
        r |= if p[2] & x != 0 { y      } else { 0 };
    }
    r << 8
}