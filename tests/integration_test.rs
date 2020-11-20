extern crate wavfile;

use wavfile::WaveReader;
use wavfile::Error;

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
    assert_eq!(format.tag, 1);
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