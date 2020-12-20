use super::{
    Error,
    read::FileReader,
    write::FileWriter,
    encode::{
        Options,
        SVDVectors,
        write_vectors_header,
        write_vectors_f32,
        write_vectors_f64,
    }
};

use std::{
    path::Path,
    fs::File
};

use nalgebra::{ DMatrix, DVector, Scalar };

use image::{ImageBuffer, Rgb, Rgba, RgbImage, RgbaImage};

use wav::{
    BitDepth as WavData,
    Header as WavHeader
};

pub fn decode(input: &str, output: &str) -> Result<(), Error> {

    let (is_sound, use_f64, with_alpha, aggregate, header, mut fr) =
        read_file_header(input)?;

    let matrix = if use_f64 {
        let vectors = read_file_f64(&mut fr)?;
        recompute_matrix_f64(&vectors)?
    }
    else {
        let vectors = read_file_f32(&mut fr)?;
        recompute_matrix_f32(&vectors)?
    };

    if is_sound {
        let h = header.unwrap();
        let raw_data = sound_from_matrix(&matrix, h);
        let mut out_file = File::create(Path::new(output))?;
        wav::write(h.0, raw_data, &mut out_file)?;

    } else {
        if with_alpha {
            let imgbuf = imgbuf_from_matrix_rgba(&matrix, aggregate)?;
            imgbuf.save(output).unwrap();
        } else {
            let imgbuf = imgbuf_from_matrix_rgb(&matrix, aggregate)?;
            imgbuf.save(output).unwrap();
        }
    }

    Ok(())
}

pub fn reduce(input: &str, output: &str, options: &Options) 
    -> Result<(), Error> {

    let (_is_sound, use_f64, _with_alpha, _aggregate, header, mut fr) =
        read_file_header(input)?;

    let mut fw = FileWriter::from_name(output)?;

    if use_f64 {
        let mut vectors = read_file_f64(&mut fr)?;
        remove_vectors(&mut vectors, options)?;

        write_vectors_header(&mut fw, &vectors, options, header)?;
        write_vectors_f64(&mut fw, &vectors)?;
    }
    else {
        let mut vectors = read_file_f32(&mut fr)?;
        remove_vectors(&mut vectors, options)?;

        write_vectors_header(&mut fw, &vectors, options, header)?;
        write_vectors_f32(&mut fw, &vectors)?;
    }
    
    Ok(())
}

fn read_file_header(input: &str) 
    -> Result<(bool, bool, bool, bool, Option<(WavHeader, u32)>, FileReader), 
               Error>{

    let f = File::open(input)?;
    let mut fr = FileReader::new(f);
    let content_type = fr.read_u8()?;
    let is_sound =   if content_type & 0x8 != 0 { true } else { false };
    let use_f64 =    if content_type & 0x4 != 0 { true } else { false };
    let with_alpha = if content_type & 0x2 != 0 { true } else { false };
    let aggregate =  if content_type & 0x1 != 0 { true } else { false };

    let header: Option<(WavHeader, u32)> = if is_sound {
        let mut header_raw = [0_u8; 16];
        for i in 0..16 {
            header_raw[i] = fr.read_u8()?;
        }
        let n = fr.read_u32()?;
        Some((header_raw.into(), n))
    } else { None };

    Ok((is_sound, use_f64, with_alpha, aggregate, header, fr))
}

fn read_file_f64(fr: &mut FileReader) -> Result<SVDVectors<f64>, Error> {

    let (n, height, width) = read_file_dimensions(fr)?;

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

    let (n, height, width) = read_file_dimensions(fr)?;

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

fn read_file_dimensions(fr: &mut FileReader) -> Result<(usize, usize, usize), Error> {
    let n = fr.read_u32()? as usize;
    let height = fr.read_u32()? as usize;
    let width  = fr.read_u32()? as usize;
    Ok((n, height, width))
}

pub fn recompute_matrix_f64(vectors: &SVDVectors<f64>) 
    -> Result<DMatrix<i32>, Error> {
    
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
        m[(i, j)].round() as i32
    });

    Ok(m2)
}

pub fn recompute_matrix_f32(vectors: &SVDVectors<f32>) 
    -> Result<DMatrix<i32>, Error> {
    
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
        m[(i, j)].round() as i32
    });

    Ok(m2)
}

fn imgbuf_from_matrix_rgba(matrix: &DMatrix<i32>, aggregate: bool)
    -> Result<RgbaImage, Error> {

    let (m_height, m_width) = matrix.shape();
    let (height, width) = (m_height as u32/ 2, m_width as u32/ 2);

    let mut imgbuf = ImageBuffer::new(height, width);

    if aggregate {
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let (i, j) = (x as usize, y as usize);
            let r = matrix[(2*i  , 2*j  )] as u8;
            let g = matrix[(2*i+1, 2*j  )] as u8;
            let b = matrix[(2*i  , 2*j+1)] as u8;
            let a = matrix[(2*i+1, 2*j+1)] as u8;
            *pixel = Rgba([r, g, b, a]);
        }
    } else {
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let (i, j) = (x as usize, y as usize);
            let elm = matrix[(i, j)];
            let r = ((elm >> 24_i32) & 0xff) as u8;
            let g = ((elm >> 16_i32) & 0xff) as u8;
            let b = ((elm >> 8_i32)  & 0xff) as u8;
            let a = ( elm            & 0xff) as u8;
            *pixel = Rgba([r, g, b, a]);
        }
    }

    Ok(imgbuf)
}

fn imgbuf_from_matrix_rgb(matrix: &DMatrix<i32>, aggregate: bool)
    -> Result<RgbImage, Error> {

    let (m_height, m_width) = matrix.shape();
    let (height, width) = (m_height as u32/ 2, m_width as u32/ 2);

    if aggregate {
        let mut imgbuf = ImageBuffer::new(m_height as u32, m_width as u32);
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let (i, j) = (x as usize, y as usize);
            let r = matrix[(2*i  , 2*j  )] as u8;
            let g = matrix[(2*i+1, 2*j  )] as u8;
            let b = matrix[(2*i  , 2*j+1)] as u8;
            *pixel = Rgb([r, g, b]);
        }
        Ok(imgbuf)
    } else {
        let mut imgbuf = ImageBuffer::new(height, width);
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let (i, j) = (x as usize, y as usize);
            let elm = matrix[(i, j)];
            let r = ((elm >> 24_i32) & 0xff) as u8;
            let g = ((elm >> 16_i32) & 0xff) as u8;
            let b = ( elm            & 0xff) as u8;
            *pixel = Rgb([r, g, b]);
        }
        Ok(imgbuf)
    }
}

fn sound_from_matrix(matrix: &DMatrix<i32>, header: (WavHeader, u32)) 
    -> WavData {
    
    let n = header.1;
    let (rows, _cols) = matrix.shape();
    let mut x = vec![0_i32; n as usize];
    for k in 0_usize..n as usize {
        let i = k / rows;
        let j = k % rows;
        x[k] = matrix[(i, j)];
    }

    match header.0.bits_per_sample {
        8 => {
            let mut v = Vec::with_capacity(n as usize);
            for i in 0_usize..n as usize {
                v[i] = x[i] as u8;
            }
            WavData::Eight(v)
        },
        16 => {
            let mut v = vec![0; n as usize];
            for i in 0_usize..n as usize {
                v[i] = x[i] as i16;
            }
            WavData::Sixteen(v)
        },
        24 => {
            WavData::TwentyFour(x)
        },
        _ => WavData::Empty
    }
}

fn remove_vectors<T>(vectors: &mut SVDVectors<T>, options: &Options) 
    -> Result<(), Error>
    where T: Scalar {
    
    let h = vectors.len();
    let w = vectors[0].1.len() * vectors[0].2.len();
    let n2 = options.n_with(w, h)?;
    if n2 > h {
        return Err(Error::NotEnoughVectorsInSource)
    }

    vectors.truncate(n2);
    Ok(())
}