//! wave-inter.rs
//! (c) 2021 Jamie Hardt. All rights reserved.
//! 
//! This program demonstrats combining several wave files into a single
//! polyphonic wave file.

extern crate clap;

use std::io;
use clap::{Arg, App};

fn main() -> io::Result<()> {

    let matches = App::new("wave-deinter")
        .version("0.1")
        .author("Jamie Hardt")
        .about("Extract each channel of a polyphonic wave file as a new monoaural wave file.")
        .arg(Arg::with_name("OUTPUT")
            .long("output")
            .short("o")
            .help("Output file basename. If absent, will be the basename of INPUT.")
        )
        .arg(Arg::with_name("numeric_names")
            .long("numeric")    
            .short("n")
            .help("Use numeric channel names \"01\" \"02\" etc.")
            .takes_value(false)
        )
        .arg(Arg::with_name("channel_delimiter")
            .long("delim")
            .short("d")
            .help("Channel label delimiter.")
            .default_value(".")
        )
        .arg(Arg::with_name("INPUT")
            .help("Input wave file")
            .required(true)
        )
        .get_matches();

    println!("Command line opts: {:?}", matches);

    Ok(())
}