use super::{
    Error,
    write::FileWriter,
    decode::recompute_matrix
};

use std::fs::File;


use image::{
    io::Reader as ImageReader,
    DynamicImage,
    RgbaImage
};

use nalgebra::{ DMatrix, DVector };

type SVDVectors = Vec<(f64, DVector<f64>, DVector<f64>)>;

pub struct EncodeOptions {
    pub policy: CompressionPolicy,
    pub use_f64: bool
}

pub enum CompressionPolicy {
    Number(usize),
    Ratio(u8)
}

/// Encodes image file in `input` to vector file `output`, with given 
/// `options`.
pub fn encode(input: &str, output: &str, options: EncodeOptions) 
    -> Result<(), Error> {

    let img = read_image_file(input)?;

    let xx = img.into_rgba8();
    println!("image dimensions: {:?}", xx.dimensions());

    let matrix = image_matrix(xx);
    println!("original matrix: {}", matrix);

    let vectors = matrix_reduce(&matrix, options)?;

    let recomputed = recompute_matrix(&vectors)?;
    println!("recomputed matrix: {}", recomputed);

    write_vectors(output, &vectors)?;
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

/// Returns a DMatrix<u8> containing the data of the Rgba image.
fn image_matrix(img: RgbaImage) -> DMatrix<u8> {
    let dim = img.dimensions();
    let mut a = DMatrix::<u8>::zeros(2 * dim.0 as usize, 2 * dim.1 as usize);

    for i in 0..(dim.0 as usize) {
        for j in 0..(dim.1 as usize) {
            let pixel = img[(i as u32, j as u32)];
            a[(2*i,   2*j  )] = pixel[0];
            a[(2*i+1, 2*j  )] = pixel[1];
            a[(2*i,   2*j+1)] = pixel[2];
            a[(2*i+1, 2*j+1)] = pixel[3];
        }
    }

    a
}

fn matrix_reduce(matrix: &DMatrix<u8>, options: EncodeOptions)
    -> Result<SVDVectors, Error> {
    
    let (h, w) = matrix.shape();
    let n = match options.policy {
        CompressionPolicy::Number(n) => n,
        CompressionPolicy::Ratio(r) => {
            // TODO: check cuz sizeof(f64) = 8*sizeof(u8)
            let r2 = r as f64 / 100_f64;
            let n2 = r2 * (h as f64 * w as f64) / 
                     (1_f64 + h as f64 + w as f64);
            n2 as usize
        }
    };

    if n <= 0  {  return Err(Error::NTooSmall);  }
    println!("n: {}", n);

    let m2 = DMatrix::from_fn(h, w, |i, j| matrix[(i, j)] as f64);

    let svd = match m2.try_svd(true, true, 1.0e-5, 0) {
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

    let sv: DVector<f64> = svd.singular_values;

    let mut res = Vec::with_capacity(n);
    for i in 0..n {
        let sv_i = sv[i];
        let u_i = DVector::<f64>::from_column_slice(u.column(i).as_slice());
        let v_t_i= DVector::from_fn(w, |x, _| {  v_t[(i, x)]  });
        res.push((sv_i, u_i, v_t_i));
    }

    Ok(res)
}

fn write_vectors(output: &str, vectors: &SVDVectors) -> Result<(), Error> {
    let f = match File::create(output) {
        Ok(file) => file,
        Err(e) => return Err(Error::FileWriteError(e))
    };

    let mut fw = FileWriter::new(f);

    let n = vectors.len();
    let height = vectors[0].1.nrows();
    let width = vectors[0].2.nrows();
    fw.write_u32(n as u32)?;
    fw.write_u32(height as u32)?;
    fw.write_u32(width as u32)?;

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

impl CompressionPolicy {
    pub fn with_number(n: usize) -> Self {
        CompressionPolicy::Number(n)
    }
    pub fn with_ratio(r: u8) -> Self {
        CompressionPolicy::Ratio(r)
    }
}