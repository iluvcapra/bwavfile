//! bilts.rs
//! (c) 2021 Jamie Hardt. All rights reserved.
//! 
//! This program demonstrates the creation of a wave file with a BLITS
//! ("Black and Lanes' Ident Tones for Surround") channel identification and 
//! alignment signal.
//! 
//! TODO: Pre-calculate the sine waves to speed up generation
//! TODO: Implement command-line interface

use std::f64;
use std::io;
use bwavfile::{WaveWriter, WaveFmt};

fn sine_wave(t: u64, amplitude : i32, wavelength : u32) -> i32 {
    //I did it this way because I'm weird
    Some(t).map(|i| (i as f64) * 2f64 * f64::consts::PI / wavelength as f64 )
    .map(|f| f.sin() )
    .map(|s| (s * amplitude as f64) as i32)
    .unwrap()
}

/// Return the corresponding f32 gain for a dbfs.
/// 
/// Retval will always be positive
fn dbfs_to_f32(dbfs : f32) -> f32 {
    10f32.powf(dbfs / 20f32)
}

fn dbfs_to_signed_int(dbfs: f32, bit_depth: u32) -> i32 {
    let full_code = (1i32 << (bit_depth + 1)) - 1;
    ((full_code as f32) * dbfs_to_f32(dbfs)) as i32
}


#[derive(Clone, Copy, PartialEq)]
enum ToneBurst {
    /// Tone of .0 frequency for .1 duration at .2 dBfs
    Tone(f32, u64, f32),
    /// Silence of .0 Duration
    Silence(u64),
}

impl ToneBurst {
    fn duration(&self, sample_rate : u32) -> u64 {
        match self {
            Self::Tone(_, dur, _) => *dur * sample_rate as u64 / 1000,
            Self::Silence(dur) => *dur * sample_rate as u64 / 1000
        }
    }
}

trait ToneBurstSignal {

    fn duration(&self, sample_rate: u32) -> u64;

    fn signal(&self, t: u64, sample_rate: u32, bit_depth: u32) -> i32;
}

impl ToneBurstSignal for Vec<ToneBurst> {
    
    fn duration(&self, sample_rate: u32) -> u64 {
        self.iter().fold(0u64, |accum, &item| {
            accum + &item.duration(sample_rate)
        })
    }

    fn signal(&self, t: u64, sample_rate: u32, bit_depth: u32) -> i32 { 
        self.iter()
            .scan(0u64, |accum, &item| {
                let dur = item.duration(sample_rate);
                let this_time_range = *accum..(*accum + dur);
                *accum = *accum + dur;
                Some( (this_time_range, item) )
            })
            .find(|(range, _)| range.contains(&t))
            .map(|(_, item)| {
                match item {
                    ToneBurst::Tone(freq, _, dbfs) => {
                        let gain = dbfs_to_signed_int(dbfs, bit_depth);
                        sine_wave(t, gain, (sample_rate as f32 / freq) as u32)
                    },
                    ToneBurst::Silence(_) => {
                        0
                    }
                }
            }).unwrap_or(0i32)
    }
}


fn main() -> io::Result<()> {

    let sample_rate = 48000;
    let bits_per_sample = 24;

    // BLITS Tone signal format
    // From EBU Tech 3304 §4 - https://tech.ebu.ch/docs/tech/tech3304.pdf
    let left_channel_sequence : Vec<ToneBurst> = vec![
        // channel ident    
        ToneBurst::Tone(880.0, 600, -18.0),
        ToneBurst::Silence(200),
        ToneBurst::Silence(4000),

        // LR ident
        ToneBurst::Tone(1000.0, 1000, -18.0),
        ToneBurst::Silence(300),
        ToneBurst::Tone(1000.0, 300, -18.0),
        ToneBurst::Silence(300),
        ToneBurst::Tone(1000.0, 300, -18.0),
        ToneBurst::Silence(300),
        ToneBurst::Tone(1000.0, 300, -18.0),
        ToneBurst::Silence(300),
        ToneBurst::Tone(1000.0, 2000, -18.0),
        ToneBurst::Silence(300),

        // Phase check,
        ToneBurst::Tone(2000.0, 3000, -24.0),
        ToneBurst::Silence(200)
    ];

    let right_channel_sequence : Vec<ToneBurst> = vec![
        // channel ident
        ToneBurst::Silence(800),    
        ToneBurst::Tone(880.0, 600, -18.0),
        ToneBurst::Silence(200),
        ToneBurst::Silence(3200),

        // LR ident
        ToneBurst::Tone(1000.0, 5100, -18.0),
        ToneBurst::Silence(300),

        // Phase check,
        ToneBurst::Tone(2000.0, 3000, -24.0),
        ToneBurst::Silence(200)
    ];

    let center_channel_sequence : Vec<ToneBurst> = vec![
        // channel ident
        ToneBurst::Silence(1600),    
        ToneBurst::Tone(1320.0, 600, -18.0),
        ToneBurst::Silence(200),
        ToneBurst::Silence(2400),

        // LR ident
        ToneBurst::Silence(5400),

        // Phase check,
        ToneBurst::Tone(2000.0, 3000, -24.0),
        ToneBurst::Silence(200)
    ];

    let lfe_channel_sequence : Vec<ToneBurst> = vec![
        // channel ident
        ToneBurst::Silence(2400),    
        ToneBurst::Tone(82.5, 600, -18.0),
        ToneBurst::Silence(200),
        ToneBurst::Silence(1600),

        // LR ident
        ToneBurst::Silence(5400),

        // Phase check,
        ToneBurst::Tone(2000.0, 3000, -24.0),
        ToneBurst::Silence(200)
    ];

    let ls_channel_sequence : Vec<ToneBurst> = vec![
        // channel ident
        ToneBurst::Silence(3200),    
        ToneBurst::Tone(660.0, 600, -18.0),
        ToneBurst::Silence(200),
        ToneBurst::Silence(800),

        // LR ident
        ToneBurst::Silence(5400),

        // Phase check,
        ToneBurst::Tone(2000.0, 3000, -24.0),
        ToneBurst::Silence(200)
    ];

    let rs_channel_sequence : Vec<ToneBurst> = vec![
        // channel ident
        ToneBurst::Silence(4000),    
        ToneBurst::Tone(660.0, 600, -18.0),
        ToneBurst::Silence(200),

        // LR ident
        ToneBurst::Silence(5400),

        // Phase check,
        ToneBurst::Tone(2000.0, 3000, -24.0),
        ToneBurst::Silence(200)
    ];

    let length = [&left_channel_sequence, &right_channel_sequence, 
        &center_channel_sequence, &lfe_channel_sequence, 
        &ls_channel_sequence, &rs_channel_sequence].iter()
            .map(|i| i.duration(sample_rate)).max().unwrap_or(0);

    let frames = (0..=length).map(|frame| {
        (left_channel_sequence.signal(frame, sample_rate, bits_per_sample),
        right_channel_sequence.signal(frame, sample_rate, bits_per_sample),
        center_channel_sequence.signal(frame, sample_rate, bits_per_sample),
        lfe_channel_sequence.signal(frame, sample_rate, bits_per_sample),
        ls_channel_sequence.signal(frame, sample_rate, bits_per_sample),
        rs_channel_sequence.signal(frame, sample_rate, bits_per_sample))
    });

    let format = WaveFmt::new_pcm_multichannel(sample_rate, bits_per_sample as u16, 0b111111);

    let file = WaveWriter::create("blits.wav", format).expect("Failed to create file");
    let mut fw = file.audio_frame_writer().expect("Failed to open audio writer");
    for frame in frames {
        let buf = vec![frame.0, frame.1, frame.2, frame.3, frame.4, frame.5];
        fw.write_integer_frames(&buf).expect("Failed to write audio frame");
    }
    fw.end().expect("Failed to close frame writer");

    Ok(())
}
