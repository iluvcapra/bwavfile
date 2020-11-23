
extern crate serde_json;
use core::fmt::Debug;
use serde_json::{Value, from_str};
use std::fs::File;
use std::io::Read;

use bwavfile::WaveReader;

// Media Tests
//
// These tests compare metadata and format data read by ffprobe with the same values
// as read by `WaveReader`.

// This seems rickety but we're going with it
fn assert_match_stream<T>(stream_key: &str, 
                    other: impl Fn(&mut WaveReader<File>) -> T)
                    where T: PartialEq + Debug,
                          T: Into<Value>
                    {

    let mut json_file = File::open("tests/media_ffprobe_result.json").unwrap();
    let mut s = String::new();
    json_file.read_to_string(&mut s).unwrap();
    if let Value::Array(v) = from_str(&mut s).unwrap() { /* */
        v.iter()
            .filter(|value| {
                !value["format"]["filename"].is_null()
            })
            .for_each(|value| {
                let filen : &str = value["format"]["filename"].as_str().unwrap();
                let json_value : &Value = &value["streams"][0][stream_key];
                let mut wavfile = WaveReader::open(filen).unwrap();
                let wavfile_value: T = other(&mut wavfile);
                println!("asserting {} for {}",stream_key, filen);
                assert_eq!(Into::<Value>::into(wavfile_value), *json_value);

            })
    }
} 

#[test]
fn test_frame_count() {    
    assert_match_stream("duration_ts", |w| w.frame_length().unwrap() );
}

#[test]
fn test_sample_rate() {
    assert_match_stream("sample_rate", |w| format!("{}", w.format().unwrap().sample_rate) );
}

#[test]
fn test_channel_count() {
    assert_match_stream("channels", |w| w.format().unwrap().channel_count );
}
