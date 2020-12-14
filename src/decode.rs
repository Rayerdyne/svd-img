use super::{
    Error,
    read::FileReader
};

use std::fs::File;

use nalgebra::{ DMatrix, DVector };

use image::{ RgbaImage, ImageBuffer, Rgba };

type SVDVectors = Vec<(f64, DVector<f64>, DVector<f64>)>;

pub fn decode(input: &str, output: &str) -> Result<(), Error> {
    let vectors = read_file(input)?;

    let matrix = recompute_matrix(vectors)?;

    let imgbuf = imgbuf_from_matrix(matrix)?;

    imgbuf.save(output).unwrap();

    Ok(())
}

fn read_file(name: &str) -> Result<SVDVectors, Error> {
    let f = File::open(name)?;
    let mut fr = FileReader::new(f);

    let n = fr.read_u32()? as usize;
    let height = fr.read_u32()? as usize;
    let width  = fr.read_u32()? as usize;

    let mut res = Vec::with_capacity(n);

    for i in 0..n {
        let sv_i = fr.read_f64()?;
        let mut u_i   = DVector::<f64>::zeros(height);
        let mut v_t_i = DVector::<f64>::zeros(width);
        for j in 0..height 
            { u_i[j] = fr.read_f64()?; }
        for j in 0..width 
            { v_t_i[j] = fr.read_f64()?; }

        res[i] = (sv_i, u_i, v_t_i);
    }

    Ok(res)
}

fn recompute_matrix(vectors: SVDVectors) -> Result<DMatrix<u8>, Error> {
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

fn imgbuf_from_matrix(matrix: DMatrix<u8>) -> Result<RgbaImage, Error> {
    let (m_height, m_width) = matrix.shape();
    let (height, width) = (m_height as u32/ 2, m_width as u32/ 2);

    let mut imgbuf = ImageBuffer::new(height, width);

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let (i, j) = (x as usize, y as usize);
        let r = matrix[(i  , j  )];
        let g = matrix[(i+1, j  )];
        let b = matrix[(i  , j+1)];
        let a = matrix[(i+1, j+1)];
        *pixel = Rgba([r, g, b, a]);
    }

    Ok(imgbuf)
}