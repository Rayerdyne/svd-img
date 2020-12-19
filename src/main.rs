mod read;
mod write;
mod encode;
mod decode;

extern crate clap;
use clap::{Arg, App};
use encode::{encode, EncodeOptions, CompressionPolicy};
use decode::decode;

use std::io::Error as IOError;

const ENCODE: u8 = 1;
const DECODE: u8 = 2;

#[derive(Debug)]
pub enum Error {
    FileReaderError(read::FileReaderError),
    FileWriteError(IOError),
    ImageReadError,
    ImageFormatError,
    SVDError, NoSVDResult, 
    NTooSmall, RatioTooRestrictive
}

impl std::convert::From<IOError> for Error {
    fn from(e: IOError) -> Self {
        Error::FileWriteError(e)
    }
}

impl std::convert::From<read::FileReaderError> for Error {
    fn from(e: read::FileReaderError) -> Self {
        Error::FileReaderError(e)
    }
}

fn app_args() -> clap::ArgMatches<'static> {
    App::new("svd-img")
        .version("0.1.0")
        .author("François Straet")
        .about("Compress images and WAV files using SVD")
        .before_help("The input type (image or WAV file) is deduced from its \
                      extention: \".WAV\" or \".wav\" files are considered as \
                      sounds, otherwise as an image file.")
        .arg(Arg::with_name("input")
            .help("Sets the input file name")
            .required(true)
            .index(1))
        .arg(Arg::with_name("output")
            .help("Set the output file name.")
            .required(true)
            .index(2))
        .arg(Arg::with_name("mode-encode")
            .help("Sets the mode to encode (clashes with -d) (default)")
            .short("e")
            .long("encode"))
        .arg(Arg::with_name("mode-decode")
            .help("Sets the mode to decode (clashes with -e)")
            .short("d")
            .long("decode")
            .conflicts_with("mode-encode"))
        .arg(Arg::with_name("num-vectors")
            .help("Sets the number of vectors to store in the compressed file.")
            .short("n")
            .takes_value(true))
        .arg(Arg::with_name("compression-ratio")
            .help("Sets the compression ratio, in percentage, compared to the \
                   uncompressed RGBA image. (clashes with -n option).")
            .short("p")
            .takes_value(true)
            .conflicts_with("num-vectors"))
        .arg(Arg::with_name("type-f32")
            .help("Sets the type used to represent float values to f32 \
                  (simple precision)")
            .short("4")
            .long("simple-precision"))
        .arg(Arg::with_name("type-f64")
            .help("Sets the type used to represent float values to f64 \
                   (double precision) (default)")
            .short("8")
            .long("double-precision")
            .conflicts_with("type-f32"))
        .arg(Arg::with_name("epsilon")
            .help("Sets the tolerance used to determine if a value converged \
                   to zero (simple precision) used to compute the SVD")
            .short("E")
            .long("epsilon")
            .takes_value(true))
        .arg(Arg::with_name("n-iter")
            .help("Sets the maximum number of iteration when computing the \
SVD, 0 for iterating until convergence")
            .short("i")
            .long("n-iter")
            .takes_value(true))
        .arg(Arg::with_name("wav-input")
            .help("Consider the input as WAV file, independently of the file \
                   extention.")
            .short("W")
            .long("wav-input"))
        .get_matches()
}

fn main() -> Result<(), Error> {
    let matches = app_args();

    let input = matches.value_of("input").unwrap();
    let output = matches.value_of("output").unwrap();

    let action_type = if matches.is_present("mode-decode") { DECODE }
                      else { ENCODE };

    let mut options = EncodeOptions::default();
    
    options.use_f64 = !matches.is_present("type-f32");
    options.eps = match matches.value_of("epsilon").unwrap_or("1.0e-5")
                               .parse::<f32>() {
        Ok(x) => x,
        Err(e) => {
            println!("Invalid epsilon provided: {:?}", e);
            return Ok(());
        }
    };
    options.n_iter = match matches.value_of("n-iter").unwrap_or("0")
                                  .parse::<usize>() {
        Ok(n) => n,
        Err(e) => {
            println!("Invalid number of iterations: {:?}", e);
            return Ok(());
        }
    };
    options.is_wav = matches.is_present("wav-input");
    options.policy = match matches.value_of("num-vectors") {
        Some(n_str) => match n_str.parse::<usize>() {
            Ok(n) => CompressionPolicy::with_number(n),
            Err(e) => {
                println!("Invalid number of vectors: {:?}", e);
                return Ok(())
            }
        },
        None => match matches.value_of("compression-ratio") {
            Some(r_str) => match r_str.parse::<u8>() {
                Ok(r) => {
                    if r > 100 {
                        println!("Compression ratio cannot exceed 100% !");
                        return Ok(());
                    }
                    CompressionPolicy::with_ratio_percentage(r)
                },
                Err(e) => {
                    println!("Invalid compression ratio: {:?}", e);
                    return Ok(())
                }
            }
            None => {
                if action_type == ENCODE {
                    println!("Using default compression ratio (25%).");
                }
                CompressionPolicy::with_ratio_percentage(25)
            }
        }
    };

    if action_type == ENCODE {
        match encode(input, output, &mut options) {
            Err(e) => println!("Could not encode {}: {:?}.", input, e),
            Ok(_) => {}
        }
    }
    
    else if action_type == DECODE {
        match decode(input, output) {
            Err(e) => println!("Could not decode {}: {:?}.", input, e),
            Ok(_) => {}
        }
    }

    Ok(())
}
