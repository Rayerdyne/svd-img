use super::{
    Error,
    write::FileWriter
};

use std::fs::File;

use image::{
    io::Reader as ImageReader,
    DynamicImage,
    RgbaImage
};

use nalgebra::{ DMatrix, DVector };

type SVDVectors<T> = Vec<(T, DVector<T>, DVector<T>)>;

pub struct EncodeOptions {
    pub policy: CompressionPolicy,
    pub use_f64: bool,
    pub eps: f32,
    pub n_iter: usize
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

    let matrix = image_matrix(xx);

    let f = match File::create(output) {
        Ok(file) => file,
        Err(e) => return Err(Error::FileWriteError(e))
    };

    let mut fw = FileWriter::new(f);

    if options.use_f64 {
        let vectors: SVDVectors<f64> = matrix_reduce_f64(&matrix, options)?;
        write_vectors_header(&mut fw, &vectors, true)?;
        write_vectors_f64(&mut fw, &vectors)?;
    }
    else {
        let vectors: SVDVectors<f32> = matrix_reduce_f32(&matrix, options)?;
        
        write_vectors_header(&mut fw, &vectors, false)?;
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

fn matrix_reduce_f64(matrix: &DMatrix<u8>, options: EncodeOptions)
    -> Result<SVDVectors<f64>, Error>
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

fn matrix_reduce_f32(matrix: &DMatrix<u8>, options: EncodeOptions)
    -> Result<SVDVectors<f32>, Error>
    {
    
    let (h, w) = matrix.shape();
    let n = options.n_with(h, w)?;

    let m2 = DMatrix::from_fn(h, w, |i, j| matrix[(i, j)].into());

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

fn write_vectors_header<T>(fw: &mut FileWriter, vectors: &SVDVectors<T>,
    use_f64: bool) -> Result<(), Error> 
    where T: std::fmt::Debug + nalgebra::Scalar
    {
    let n = vectors.len();
    let height = vectors[0].1.nrows();
    let width = vectors[0].2.nrows();
    if use_f64 { fw.write_u8(1)?; }
    else       { fw.write_u8(0)?; }
    fw.write_u32(n as u32)?;
    fw.write_u32(height as u32)?;
    fw.write_u32(width as u32)?;

    Ok(())
}

fn write_vectors_f64(fw: &mut FileWriter, vectors: &SVDVectors<f64>)
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

fn write_vectors_f32(fw: &mut FileWriter, vectors: &SVDVectors<f32>)
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

impl EncodeOptions {
    pub fn default() -> Self {
        EncodeOptions {
            eps: 1.0e-5,
            policy: CompressionPolicy::with_ratio_percentage(25),
            use_f64: true,
            n_iter: 0
        }
    }

    fn n_with(&self, h: usize, w: usize) -> Result<usize, Error> {
        match self.policy {
            CompressionPolicy::Number(n) => {
                if n <= 0 {  return Err(Error::NTooSmall);  }
                Ok(n)
            },
            CompressionPolicy::Ratio(r) => {
                let data_bits = if self.use_f64 {  8_f64  }
                                else            {  4_f64  };
                let r_f64 = r as f64 / 100_f64;

                let img_size = h as f64 * w as f64;
                let vector_size = data_bits * (1_f64 + h as f64 + w as f64);
                // n * vector_size = r_f64 * img_size
                let n = (r_f64 * img_size) / vector_size;
                if n.round() <= 0.0 {  return Err(Error::RatioTooRestrictive);  }
                Ok(n.round() as usize)
            }
        }
    }
}