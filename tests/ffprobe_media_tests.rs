extern crate serde_json;

use serde_json::{Result, Value, from_str};

use std::fs::File;
use std::io::Read;

#[test]
fn test_a() {
    let mut json_file = File::open("tests/ffprobe_media_results.json").unwrap();
    let mut s = String::new();
    json_file.read_to_string(&mut s).unwrap();
    let v: Value = from_str(&mut s).unwrap();

    //println!("file list: {:?}", ffprobe_data);

}