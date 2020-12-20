mod read;
mod write;
mod encode;
mod decode;

extern crate clap;
use clap::{Arg, App};
use encode::{encode, Options, CompressionPolicy};
use decode::{decode, reduce};

use std::io::Error as IOError;

enum ActionTypes {
    Encode, Decode, Reduce
}

impl std::cmp::PartialEq for ActionTypes {
    fn eq(&self, other: &Self) -> bool {
        match self {
            ActionTypes::Decode => match other {
                ActionTypes::Decode => true,
                _ => false,
            },
            ActionTypes::Encode => match other {
                ActionTypes::Encode => true,
                _ => false,
            },
            ActionTypes::Reduce => match other {
                ActionTypes::Reduce => true,
                _ => false,
            },
        }
    }

    // fn ne(&self, other: &Self) -> bool {
    //     !std::cmp::eq(other)
    // }
}

#[derive(Debug)]
pub enum Error {
    FileReaderError(read::FileReaderError),
    FileWriteError(IOError),
    ImageReadError,
    ImageFormatError,
    SVDError, NoSVDResult, 
    NTooSmall, RatioTooRestrictive, NotEnoughVectorsInSource
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
        .arg(Arg::with_name("input")
            .help("Sets the input file name. If no specific option are given, \
                   the input type (image or WAV file) is deduced from its \
                   extention: \".WAV\" or \".wav\" files are considered as \
                   sounds, otherwise as an image file.")
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
        .arg(Arg::with_name("mode-reduce")
            .help("Sets the mode to reduce (compress an already compressed \
                   file even more)")
            .short("r")
            .long("reduce")
            .conflicts_with("mode-encode")
            .conflicts_with("mode-decode"))
        .arg(Arg::with_name("num-vectors")
            .help("Sets the number of vectors to store in the compressed file."
                )
            .short("n")
            .long("num-vectors")
            .takes_value(true))
        .arg(Arg::with_name("compression-%")
            .help("Sets the compression ratio, in percentage, compared to the \
                   uncompressed RGBA image. (clashes with -n option).")
            .short("p")
            .long("compression-%")
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
        .arg(Arg::with_name("with-alpha")
            .help("Adds an alpha channel to the compressed image")
            .short("a")
            .long("with-alpha"))
        .arg(Arg::with_name("no-aggregate")
            .help("Disables the aggregation of pixels values into one greater \
                   number. That is, a pixel will be spread into 4 values \
                   (even without an alpha channel).")
            .short("s")
            .long("no-aggregate"))
        .get_matches()
}

fn main() -> Result<(), Error> {
    let matches = app_args();

    let input = matches.value_of("input").unwrap();
    let output = matches.value_of("output").unwrap();

    let action_type =   if matches.is_present("mode-decode") 
                             { ActionTypes::Decode }
                        else if matches.is_present("mode-reduce")
                             { ActionTypes::Reduce }
                        else { ActionTypes::Encode };

    let mut options = Options::default();
    
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
    options.is_reduce = matches.is_present("mode-reduce");
    
    options.with_alpha = matches.is_present("with-alpha");
    options.use_aggregate = !matches.is_present("no-aggregate");

    options.policy = match matches.value_of("num-vectors") {
        Some(n_str) => match n_str.parse::<usize>() {
            Ok(n) => CompressionPolicy::with_number(n),
            Err(e) => {
                println!("Invalid number of vectors: {:?}", e);
                return Ok(())
            }
        },
        None => match matches.value_of("compression-%") {
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
                if action_type == ActionTypes::Encode {
                    println!("Using default compression ratio (25%).");
                }
                CompressionPolicy::with_ratio_percentage(25)
            }
        }
    };

    let result = if action_type == ActionTypes::Encode {
        encode(input, output, &mut options)
    }
    else if action_type == ActionTypes::Decode {
        decode(input, output)
    }
    else /* action_type == ActionTypes::Reduce */ {
        reduce(input, output, &options)        
    };

    match result {
        Err(e) => println!("Could not {}: {:?}",
            match action_type {
                ActionTypes::Encode => "encode",
                ActionTypes::Decode => "decode",
                ActionTypes::Reduce => "reduce",
            }, e),
        Ok(_) => {}
    }

    Ok(())
}
