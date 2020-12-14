use super::{
    Error,
    read::FileReader
};

use std::fs::File;

use nalgebra::{ DMatrix, DVector };

use image::{ RgbaImage, ImageBuffer, Rgba };

type SVDVectors<T> = Vec<(T, DVector<T>, DVector<T>)>;

pub fn decode(input: &str, output: &str) -> Result<(), Error> {
    let f = File::open(input)?;
    let mut fr = FileReader::new(f);
    let use_f64 = if fr.read_u8()? >= 1 { true } else { false };
    
    let imgbuf = if use_f64 {
        let vectors = read_file_f64(&mut fr)?;
        let matrix = recompute_matrix_f64(&vectors)?;
        imgbuf_from_matrix(&matrix)?
    }
    else {
        let vectors = read_file_f32(&mut fr)?;
        let matrix = recompute_matrix_f32(&vectors)?;
        imgbuf_from_matrix(&matrix)?
    };

    imgbuf.save(output).unwrap();

    Ok(())
}

fn read_file_f64(fr: &mut FileReader) -> Result<SVDVectors<f64>, Error> {

    let (n, height, width) = read_file_header(fr)?;

    let mut res = Vec::with_capacity(n);

    for _i in 0..n {
        let sv_i = fr.read_f64()?;
        let mut u_i   = DVector::<f64>::zeros(height);
        let mut v_t_i = DVector::<f64>::zeros(width);
        for j in 0..height 
            { u_i[j] = fr.read_f64()?; }
        for j in 0..width 
            { v_t_i[j] = fr.read_f64()?; }

        res.push((sv_i, u_i, v_t_i));
    }

    Ok(res)
}

fn read_file_f32(fr: &mut FileReader) -> Result<SVDVectors<f32>, Error> {

    let (n, height, width) = read_file_header(fr)?;

    let mut res = Vec::with_capacity(n);

    for _i in 0..n {
        let sv_i = fr.read_f32()?;
        let mut u_i   = DVector::<f32>::zeros(height);
        let mut v_t_i = DVector::<f32>::zeros(width);
        for j in 0..height 
            { u_i[j] = fr.read_f32()?; }
        for j in 0..width 
            { v_t_i[j] = fr.read_f32()?; }

        res.push((sv_i, u_i, v_t_i));
    }

    Ok(res)
}

fn read_file_header(fr: &mut FileReader) -> Result<(usize, usize, usize), Error> {
    let n = fr.read_u32()? as usize;
    let height = fr.read_u32()? as usize;
    let width  = 
    fr.read_u32()? as usize;
    Ok((n, height, width))
}

pub fn recompute_matrix_f64(vectors: &SVDVectors<f64>) 
    -> Result<DMatrix<u8>, Error> {
    
    let n = vectors.len();
    if n <= 0  {  return Err(Error::NTooSmall);  }
    let height = vectors[0].1.nrows();
    let width  = vectors[0].2.nrows();

    let mut m = DMatrix::<f64>::zeros(height, width);

    for i in 0..n {
        let triplet = &vectors[i];
        for j in 0..height {
            for k in 0..width {
                m[(j, k)] += triplet.0 *
                             triplet.1[j] *
                             triplet.2[k];
            }
        }
    }

    let m2 = DMatrix::from_fn(height, width, |i, j| {
        m[(i, j)].round() as u8
    });

    Ok(m2)
}

pub fn recompute_matrix_f32(vectors: &SVDVectors<f32>) 
    -> Result<DMatrix<u8>, Error> {
    
    let n = vectors.len();
    if n <= 0  {  return Err(Error::NTooSmall);  }
    let height = vectors[0].1.nrows();
    let width  = vectors[0].2.nrows();

    let mut m = DMatrix::<f32>::zeros(height, width);

    for i in 0..n {
        let triplet = &vectors[i];
        for j in 0..height {
            for k in 0..width {
                m[(j, k)] += triplet.0 *
                             triplet.1[j] *
                             triplet.2[k];
            }
        }
    }

    let m2 = DMatrix::from_fn(height, width, |i, j| {
        m[(i, j)].round() as u8
    });

    Ok(m2)
}



fn imgbuf_from_matrix(matrix: &DMatrix<u8>) -> Result<RgbaImage, Error> {
    let (m_height, m_width) = matrix.shape();
    let (height, width) = (m_height as u32/ 2, m_width as u32/ 2);

    let mut imgbuf = ImageBuffer::new(height, width);

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let (i, j) = (x as usize, y as usize);
        let r = matrix[(2*i  , 2*j  )];
        let g = matrix[(2*i+1, 2*j  )];
        let b = matrix[(2*i  , 2*j+1)];
        let a = matrix[(2*i+1, 2*j+1)];
        *pixel = Rgba([r, g, b, a]);
    }

    Ok(imgbuf)
}