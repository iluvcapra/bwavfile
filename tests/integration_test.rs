extern crate bwavfile;

use bwavfile::WaveReader;
use bwavfile::Error;
use bwavfile::{ ChannelMask}; 

#[test]
fn test_open() {
    let path = "tests/media/ff_silence.wav";

    match WaveReader::open(path) {
        Ok(_) => {
            ()
        }, 
        Err(x) => {
            assert!(false, "Opened error.wav with unexpected error {:?}", x)
        }
    }
}

#[test]
fn test_format_silence() -> Result<(),Error> {
    let path = "tests/media/ff_silence.wav";

    let mut w = WaveReader::open(path)?;

    let format = w.format()?;

    assert_eq!(format.sample_rate, 44100);
    assert_eq!(format.channel_count, 1);
    assert_eq!(format.tag as u16, 1);
    Ok( () )
}

#[test]
fn test_format_error() {
    let path = "tests/media/error.wav";

    if let Ok(_) = WaveReader::open(path) {
        assert!(false);
    } else {
        assert!(true);
    }
}

#[test]
fn test_frame_count() -> Result<(),Error> {
    let path = "tests/media/ff_silence.wav";

    let mut w = WaveReader::open(path)?;
    let l = w.frame_length()?;
    assert_eq!(l, 44100);

    Ok( () )
}

#[test]
fn test_minimal_wave()  {
    let path = "tests/media/ff_silence.wav";

    let mut w = WaveReader::open(path).expect("Failure opening file");

    if let Err(Error::NotMinimalWaveFile) = w.validate_minimal() {
        assert!(true);
    } else {
        assert!(false);
    }

    let min_path = "tests/media/ff_minimal.wav";

    let mut w = WaveReader::open(min_path).expect("Failure opening file");

    if let Err(Error::NotMinimalWaveFile) = w.validate_minimal() {
        assert!(false);
    } else {
        assert!(true);
    }
}

#[test]
fn test_read() {
    let path = "tests/media/audacity_16bit.wav";

    let mut w = WaveReader::open(path).expect("Failure opening test file");
    let mut buffer = w.format().unwrap().create_frame_buffer();

    let mut reader = w.audio_frame_reader().unwrap();

    

    assert_eq!(reader.read_integer_frame(&mut buffer).unwrap(), 1);
    assert_eq!(buffer[0], -2823_i32);
    assert_eq!(reader.read_integer_frame(&mut buffer).unwrap(), 1);
    assert_eq!(buffer[0], 2012_i32);
    assert_eq!(reader.read_integer_frame(&mut buffer).unwrap(), 1);
    assert_eq!(buffer[0], 4524_i32); 
}

#[test]
fn test_locate_multichannel_read() {
    let path = "tests/media/ff_pink.wav";

    let mut w = WaveReader::open(path).expect("Failure opening test file");
    let mut buffer = w.format().unwrap().create_frame_buffer();

    let mut reader = w.audio_frame_reader().unwrap();

    assert_eq!(reader.read_integer_frame(&mut buffer).unwrap(), 1);
    assert_eq!(buffer[0], 332702_i32);
    assert_eq!(buffer[1], 3258791_i32);
    assert_eq!(reader.read_integer_frame(&mut buffer).unwrap(), 1);
    assert_eq!(buffer[0], -258742_i32); // 0x800000 = 8388608 // 8129866 - 8388608
    assert_eq!(buffer[1], 0x0D7EF9_i32);

    assert_eq!(reader.locate(100).unwrap(), 100);
    assert_eq!(reader.read_integer_frame(&mut buffer).unwrap(), 1);
    assert_eq!(buffer[0], 0x109422_i32);
    assert_eq!(buffer[1], -698901_i32); // 7689707 - 8388608
}

#[test]
fn test_channels_stereo() {
    let path = "tests/media/ff_pink.wav";

    let mut w = WaveReader::open(path).expect("Failure opening test file");
    let channels = w.channels().unwrap();

    assert_eq!(channels.len(), 2);
    assert_eq!(channels[0].index,0);
    assert_eq!(channels[1].index,1);
    assert_eq!(channels[0].speaker,ChannelMask::FrontLeft);
    assert_eq!(channels[1].speaker,ChannelMask::FrontRight);
}

#[test]
fn test_channels_mono_no_extended() {
    let path = "tests/media/audacity_16bit.wav";

    let mut w = WaveReader::open(path).expect("Failure opening test file");
    let channels = w.channels().unwrap();

    assert_eq!(channels.len(), 1);
    assert_eq!(channels[0].index,0);
    assert_eq!(channels[0].speaker,ChannelMask::FrontCenter);
}

#[test]
fn test_channels_stereo_no_fmt_extended() {
    let path = "tests/media/pt_24bit_stereo.wav";

    let mut w = WaveReader::open(path).expect("Failure opening test file");
    let channels = w.channels().unwrap();

    assert_eq!(channels.len(), 2);
    assert_eq!(channels[0].index,0);
    assert_eq!(channels[1].index,1);
    assert_eq!(channels[0].speaker,ChannelMask::FrontLeft);
    assert_eq!(channels[1].speaker,ChannelMask::FrontRight);
}

//See issue 6 and 7
#[test]
fn test_frame_reader_consumes_reader() {
    // Issue #6
    use bwavfile::WaveFmt;
    use bwavfile::AudioFrameReader;
    use std::fs::File;
    fn from_wav_filename(wav_filename: &str) -> Result<(WaveFmt, AudioFrameReader<File>), ()> {
        if let Ok(mut r) = WaveReader::open(&wav_filename) {
            let format = r.format().unwrap();
            let frame_reader = r.audio_frame_reader().unwrap();
            Ok((format, frame_reader))
        } else {
            Err(())
        }
    }

    let _result = from_wav_filename("tests/media/pt_24bit_stereo.wav").unwrap();
}