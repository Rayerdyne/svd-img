mod read;
mod write;
mod encode;
mod decode;

extern crate clap;
use clap::{Arg, App};
use encode::{encode, EncodeOptions};
use decode::decode;

use std::io::Error as IOError;

const ERROR: u8 = 0;
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
            .required(true)
            .index(2))
        .arg(Arg::with_name("type")
            .help("Sets the mode: encode (enc, e) or decode (dec, d)")
            .short("m")
            .takes_value(true)
            .possible_values(&["enc", "dec", "encode", "decode", "e", "d"]))
        .arg(Arg::with_name("num-vectors")
            .help("Sets the number of vectors to store in the compressed file.")
            .short("n")
            .takes_value(true))
        .arg(Arg::with_name("compression-ratio")
            .help("Sets the compression ratio, in percentage. (clashes with -n option).")
            .short("p")
            .takes_value(true)
            .conflicts_with("num-vectors"))

        .get_matches()
}

fn main() -> Result<(), Error> {
    let matches = app_args();

    let input = matches.value_of("input").unwrap();
    let output = matches.value_of("output").unwrap_or("output.svd");

    let action_type = match matches.value_of("type").unwrap_or("encode") {
        "encode" | "enc" | "e" => ENCODE,
        "decode" | "dec" | "d" => DECODE,
        _ => ERROR
    };

    if action_type == ERROR {
        println!("Could not parse arguments for some reason (?!)");
    }

    else if action_type == ENCODE {
        let options = match matches.value_of("num-vectors") {
            Some(n_str) => match n_str.parse::<usize>() {
                Ok(n) => EncodeOptions::with_number(n),
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
                        EncodeOptions::with_ratio(r)
                    },
                    Err(e) => {
                        println!("Invalid compression ratio: {:?}", e);
                        return Ok(())
                    }
                }
                None => {
                    println!("Using default compression ratio (25%).");
                    EncodeOptions::with_ratio(25)
                }
            }
        };

        encode(input, output, options)?;
    }
    
    else if action_type == DECODE {
        decode(input, output)?;
    }

    Ok(())
}
