//! wave-deinter.rs
//! (c) 2021 Jamie Hardt. All rights reserved.
//!
//! This program demonstrates splitting a multichannel file into separate monophonic files for each
//! individual channel.

use std::io::{Read, Seek};
use std::path::Path;

extern crate bwavfile;
use bwavfile::{
    ChannelDescriptor, ChannelMask, CommonFormat, Error, Sample, WaveFmt, WaveReader, WaveWriter,
    I24,
};

#[macro_use]
extern crate clap;
use clap::{App, Arg};

fn name_suffix(
    force_numeric: bool,
    delim: &str,
    index: usize,
    channel_descriptor: &ChannelDescriptor,
) -> String {
    if force_numeric || channel_descriptor.speaker == ChannelMask::DirectOut {
        format!("{}A{:02}", delim, index)
    } else {
        let chan_name = match channel_descriptor.speaker {
            ChannelMask::FrontLeft => "L",
            ChannelMask::FrontCenter => "C",
            ChannelMask::FrontRight => "R",
            ChannelMask::BackLeft => "Ls",
            ChannelMask::BackRight => "Rs",
            ChannelMask::BackCenter => "S",
            ChannelMask::TopCenter => "Tc",
            ChannelMask::LowFrequency => "Lfe",
            ChannelMask::SideLeft => "Lss",
            ChannelMask::SideRight => "Rss",
            ChannelMask::FrontCenterLeft => "Lc",
            ChannelMask::FrontCenterRight => "Rc",
            ChannelMask::TopFrontLeft => "Ltf",
            ChannelMask::TopFrontCenter => "Ctf",
            ChannelMask::TopFrontRight => "Rtf",
            ChannelMask::TopBackLeft => "Ltb",
            ChannelMask::TopBackCenter => "Ctb",
            ChannelMask::TopBackRight => "Rtb",
            ChannelMask::DirectOut => panic!("Error, can't get here"),
        };
        format!("{}{}", delim, chan_name)
    }
}

fn deinterleave_file<S, R>(
    mut input_file: WaveReader<R>,
    input_format: WaveFmt,
    settings: Settings,
) -> Result<(), Error>
where
    S: Sample,
    R: Read + Seek,
{
    let frames_per_read = 4096;
    let channel_desc = input_file.channels()?;
    let channel_count = channel_desc.len();

    if channel_desc.len() == 1 {
        println!("Input file in monoaural, exiting.");
        return Ok(());
    }

    let infile_path = Path::new(&settings.input_path);
    let basename = infile_path
        .file_stem()
        .expect("Unable to extract file basename")
        .to_str()
        .unwrap();
    let output_dir = infile_path
        .parent()
        .expect("Unable to derive parent directory");

    let output_block_alignment = input_format.bits_per_sample / 8;
    let output_format = WaveFmt {
        channel_count: 1,
        block_alignment: output_block_alignment,
        bytes_per_second: output_block_alignment as u32 * input_format.sample_rate,
        ..input_format
    };
    let mut reader = input_file.audio_frame_reader()?;
    let mut writers = channel_desc
        .iter()
        .enumerate()
        .map(|(n, channel)| {
            let suffix = name_suffix(
                settings.use_numeric_names,
                &settings.delimiter,
                n + 1,
                channel,
            );
            let outfile_name = output_dir
                .join(format!("{}{}.wav", basename, suffix))
                .into_os_string()
                .into_string()
                .unwrap();

            println!("Will create file {}", outfile_name);

            WaveWriter::create(&outfile_name, output_format)
                .expect("Failed to create new file")
                .audio_frame_writer()
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut input_buffer = vec![S::EQUILIBRIUM; frames_per_read * channel_count];
    let mut output_buffer = vec![S::EQUILIBRIUM; frames_per_read];

    loop {
        let frames_read = reader.read_frames(&mut input_buffer)? as usize;
        if frames_read == 0 {
            break;
        }

        output_buffer.resize(frames_read, S::EQUILIBRIUM);

        for (n, writer) in writers.iter_mut().enumerate() {
            for (output, input) in output_buffer
                .iter_mut()
                .zip(input_buffer.iter().skip(n).step_by(channel_count))
            {
                *output = *input;
            }
            writer.write_frames(&output_buffer)?;
        }
    }

    for writer in writers.drain(..) {
        writer.end()?;
    }

    Ok(())
}

fn process_file<R>(mut input: WaveReader<R>, settings: Settings) -> Result<(), Error>
where
    R: Read + Seek,
{
    let format = input.format()?;

    use CommonFormat::*;
    match (format.common_format(), format.bits_per_sample) {
        (IntegerPCM, 8) => deinterleave_file::<u8, R>(input, format, settings),
        (IntegerPCM, 16) => deinterleave_file::<i16, R>(input, format, settings),
        (IntegerPCM, 24) => deinterleave_file::<I24, R>(input, format, settings),
        (IntegerPCM, 32) => deinterleave_file::<i32, R>(input, format, settings),
        (IeeeFloatPCM, 32) => deinterleave_file::<f32, R>(input, format, settings),
        other => panic!("Unsupported format: {:?}", other),
    }
}

struct Settings {
    input_path: String,
    delimiter: String,
    use_numeric_names: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("wave-deinter")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Extract each channel of a polyphonic wave file as a new monoaural wave file.")
        .arg(
            Arg::with_name("numeric_names")
                .long("numeric")
                .short("n")
                .help("Use numeric channel names \"01\" \"02\" etc.")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("channel_delimiter")
                .long("delim")
                .short("d")
                .help("Channel label delimiter.")
                .default_value("."),
        )
        .arg(
            Arg::with_name("INPUT")
                .help("Input wave file")
                .required(true)
                .multiple(true),
        )
        .get_matches();

    let settings = Settings {
        input_path: matches.value_of("INPUT").unwrap().into(),
        delimiter: matches.value_of("channel_delimiter").unwrap().into(),
        use_numeric_names: matches.is_present("numeric_names"),
    };

    process_file(WaveReader::open(&settings.input_path)?, settings)?;
    Ok(())
}
