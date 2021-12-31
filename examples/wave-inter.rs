//! wave-inter.rs
//! (c) 2021 Jamie Hardt. All rights reserved.
//!
//! This program demonstrates combining several wave files into a single
//! polyphonic wave file.

use std::io;

extern crate bwavfile;

#[macro_use]
extern crate clap;
use clap::{App, Arg};

fn main() -> io::Result<()> {
    let matches = App::new("wave-inter")
        .version(crate_version!())
        .author(crate_authors!())
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
