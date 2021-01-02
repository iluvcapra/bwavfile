//! wave-inter.rs
//! (c) 2021 Jamie Hardt. All rights reserved.
//! 
//! This program demonstrates combining several wave files into a single
//! polyphonic wave file.

extern crate clap;

use std::io;
use clap::{Arg, App};

fn main() -> io::Result<()> {
    let matches = App::new("wave-inter")
        .version("0.1")
        .author("Jamie Hardt")
        .about("Combine several wave files into a single polyphonic wave file.")
        .arg(Arg::with_name("OUTPUT")
            .long("output")
            .short("o")
            .help("Output file name. If absent, will be basename, minus any channel extension, of first INPUT.")
        )
        .arg(Arg::with_name("INPUT")
            .help("Input wave file")
            .required(true)
            .multiple(true)
        )
        .get_matches();

    println!("Command line opts: {:?}", matches);

    todo!("Finish implementation");
}