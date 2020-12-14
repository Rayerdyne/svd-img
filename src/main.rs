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
    SVDError, NoSVDResult, NTooSmall
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
        .about("Compress images using SVD")
        .arg(Arg::with_name("input")
            .help("Sets the input image file name")
            .required(true)
            .index(1))
        .arg(Arg::with_name("output")
            .help("Set the output file name. Default output name is 'output.svd'.")
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
            .help("Sets the compression ratio, in percentage. (clashes with -n option).")
            .short("p")
            .takes_value(true)
            .conflicts_with("num-vectors"))
        .arg(Arg::with_name("type-f32")
            .help("Sets the type used to represent float values to f32 (simple precision)")
            .short("f32")
            .long("simple-precision"))
        .arg(Arg::with_name("type-f64")
            .help("Sets the type used to represent float values to f64 (double precision) (default)")
            .short("f64")
            .long("double-precision")
            .conflicts_with("type-f32"))

        .get_matches()
}

fn main() -> Result<(), Error> {
    let matches = app_args();

    let input = matches.value_of("input").unwrap();
    let output = matches.value_of("output").unwrap_or("output.svd");

    let options: EncodeOptions;

    let action_type = if matches.is_present("mode-decode") { DECODE }
                      else { ENCODE };
    
    options.use_f64 = matches.is_present("type-f64");

    if action_type == ENCODE {
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
                        CompressionPolicy::with_ratio(r)
                    },
                    Err(e) => {
                        println!("Invalid compression ratio: {:?}", e);
                        return Ok(())
                    }
                }
                None => {
                    println!("Using default compression ratio (25%).");
                    CompressionPolicy::with_ratio(25)
                }
            }
        };

        match encode(input, output, options) {
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
