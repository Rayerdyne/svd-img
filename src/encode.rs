use super::Error;


use image::{
    io::Reader as ImageReader,
    DynamicImage,
    RgbaImage
};

use nalgebra::DMatrix;

pub enum EncodeOptions {
    Number(usize),
    Ratio(u8)
}

/// Encodes image file in `input` to vector file `output`, with given 
/// `options`.
pub fn encode(input: &str, output: &str, options: EncodeOptions) 
    -> Result<(), Error> {

    let img = read_image_file(input)?;

    let matrix = image_matrix(img.into_rgba8());

    let vectors = matrix_reduce(matrix, options);

    write_vectors(output, vectors);

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
            a[(i,   j  )] = pixel[0];
            a[(i+1, j  )] = pixel[1];
            a[(i,   j+1)] = pixel[2];
            a[(i+1, j+1)] = pixel[3];
        }
    }

    a
}

fn matrix_reduce(matrix: DMatrix<u8>, options: EncodeOptions)
    -> [Vec<f64>; 2] {

    let uu: Vec<f64>;
    let vv_star: Vec<f64>;

    [uu, vv_star]
}

fn write_vectors(output: &str, vectors: [Vec<f64>; 2]) {

}

impl EncodeOptions {
    pub fn with_number(n: usize) -> Self {
        EncodeOptions::Number(n)
    }
    pub fn with_ratio(r: u8) -> Self {
        EncodeOptions::Ratio(r)
    }
}