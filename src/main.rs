mod read;
mod write;
mod encode;
mod decode;

extern crate clap;
use clap::{Arg, App};
use encode::encode;
use decode::decode;

const ERROR: u8 = 0;
const ENCODE: u8 = 1;
const DECODE: u8 = 2;

#[derive(Debug)]
pub enum Error {
    FileReaderError(read::FileReaderError)
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
        encode(input, output)?;
    }
    
    else if action_type == DECODE {
        decode(input, output)?;
    }

    Ok(())
}
