extern crate bwavfile;

use bwavfile::ChannelMask;
use bwavfile::Error;
use bwavfile::WaveReader;
use bwavfile::I24;

#[test]
fn test_open() {
    let path = "tests/media/ff_silence.wav";

    match WaveReader::open(path) {
        Ok(_) => (),
        Err(x) => {
            assert!(false, "Opened error.wav with unexpected error {:?}", x)
        }
    }
}

#[test]
fn test_format_silence() -> Result<(), Error> {
    let path = "tests/media/ff_silence.wav";

    let mut w = WaveReader::open(path)?;

    let format = w.format()?;

    assert_eq!(format.sample_rate, 44100);
    assert_eq!(format.channel_count, 1);
    assert_eq!(format.tag as u16, 1);
    Ok(())
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
fn test_frame_count() -> Result<(), Error> {
    let path = "tests/media/ff_silence.wav";

    let mut w = WaveReader::open(path)?;
    let l = w.frame_length()?;
    assert_eq!(l, 44100);

    Ok(())
}

#[test]
fn test_minimal_wave() {
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
    let mut buffer = w.format().unwrap().create_frame_buffer::<i16>(1);

    let mut reader = w.audio_frame_reader().unwrap();

    assert_eq!(reader.read_frames(&mut buffer).unwrap(), 1);
    assert_eq!(buffer[0], -2823_i16);
    assert_eq!(reader.read_frames(&mut buffer).unwrap(), 1);
    assert_eq!(buffer[0], 2012_i16);
    assert_eq!(reader.read_frames(&mut buffer).unwrap(), 1);
    assert_eq!(buffer[0], 4524_i16);
}

#[test]
fn test_locate_multichannel_read() {
    let path = "tests/media/ff_pink.wav";

    let mut w = WaveReader::open(path).expect("Failure opening test file");
    let mut buffer = w.format().unwrap().create_frame_buffer::<I24>(1);

    let mut reader = w.audio_frame_reader().unwrap();

    assert_eq!(reader.read_frames(&mut buffer).unwrap(), 1);
    assert_eq!(buffer[0], I24::from(332702));
    assert_eq!(buffer[1], I24::from(3258791));
    assert_eq!(reader.read_frames(&mut buffer).unwrap(), 1);
    assert_eq!(buffer[0], I24::from(-258742)); // 0x800000 = 8388608 // 8129866 - 8388608
    assert_eq!(buffer[1], I24::from(0x0D7EF9));

    assert_eq!(reader.locate(100).unwrap(), 100);
    assert_eq!(reader.read_frames(&mut buffer).unwrap(), 1);
    assert_eq!(buffer[0], I24::from(0x109422));
    assert_eq!(buffer[1], I24::from(-698901)); // 7689707 - 8388608
}

#[test]
fn test_channels_stereo() {
    let path = "tests/media/ff_pink.wav";

    let mut w = WaveReader::open(path).expect("Failure opening test file");
    let channels = w.channels().unwrap();

    assert_eq!(channels.len(), 2);
    assert_eq!(channels[0].index, 0);
    assert_eq!(channels[1].index, 1);
    assert_eq!(channels[0].speaker, ChannelMask::FrontLeft);
    assert_eq!(channels[1].speaker, ChannelMask::FrontRight);
}

#[test]
fn test_channels_mono_no_extended() {
    let path = "tests/media/audacity_16bit.wav";

    let mut w = WaveReader::open(path).expect("Failure opening test file");
    let channels = w.channels().unwrap();

    assert_eq!(channels.len(), 1);
    assert_eq!(channels[0].index, 0);
    assert_eq!(channels[0].speaker, ChannelMask::FrontCenter);
}

#[test]
fn test_channels_stereo_no_fmt_extended() {
    let path = "tests/media/pt_24bit_stereo.wav";

    let mut w = WaveReader::open(path).expect("Failure opening test file");
    let channels = w.channels().unwrap();

    assert_eq!(channels.len(), 2);
    assert_eq!(channels[0].index, 0);
    assert_eq!(channels[1].index, 1);
    assert_eq!(channels[0].speaker, ChannelMask::FrontLeft);
    assert_eq!(channels[1].speaker, ChannelMask::FrontRight);
}

///See issue 6 and 7
#[test]
fn test_frame_reader_consumes_reader() {
    // Issue #6
    use bwavfile::{AudioFrameReader, WaveFmt};
    use std::fs::File;
    fn from_wav_filename(
        wav_filename: &str,
    ) -> Result<(WaveFmt, AudioFrameReader<std::io::BufReader<File>>), ()> {
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

///See to PR#10
#[test]
fn test_cue_read_sounddevices() {
    let mut f = WaveReader::open("tests/media/sounddevices_6_cue_points.wav").unwrap();
    let cue_points = f.cue_points().unwrap();
    assert_eq!(cue_points.len(), 6);

    assert_eq!(cue_points[0].frame, 0);
    assert_eq!(cue_points[0].length, None);
    assert_eq!(cue_points[0].label, None);
    assert_eq!(cue_points[0].note, None);
    assert_eq!(cue_points[0].offset, 90112);

    assert_eq!(cue_points[1].frame, 0);
    assert_eq!(cue_points[1].length, None);
    assert_eq!(cue_points[1].label, None);
    assert_eq!(cue_points[1].note, None);
    assert_eq!(cue_points[1].offset, 176128);

    assert_eq!(cue_points[2].frame, 0);
    assert_eq!(cue_points[2].length, None);
    assert_eq!(cue_points[2].label, None);
    assert_eq!(cue_points[2].note, None);
    assert_eq!(cue_points[2].offset, 237568);

    assert_eq!(cue_points[3].frame, 0);
    assert_eq!(cue_points[3].length, None);
    assert_eq!(cue_points[3].label, None);
    assert_eq!(cue_points[3].note, None);
    assert_eq!(cue_points[3].offset, 294912);

    assert_eq!(cue_points[4].frame, 0);
    assert_eq!(cue_points[4].length, None);
    assert_eq!(cue_points[4].label, None);
    assert_eq!(cue_points[4].note, None);
    assert_eq!(cue_points[4].offset, 380928);

    assert_eq!(cue_points[5].frame, 0);
    assert_eq!(cue_points[5].length, None);
    assert_eq!(cue_points[5].label, None);
    assert_eq!(cue_points[5].note, None);
    assert_eq!(cue_points[5].offset, 385024);
}
