![Crates.io](https://img.shields.io/crates/l/bwavfile)
![Crates.io](https://img.shields.io/crates/v/bwavfile)
![GitHub last commit](https://img.shields.io/github/last-commit/iluvcapra/bwavfile)

# bwavfile
Rust Wave File Reader/Writer with Broadcast-WAV, MBWF and RF64 Support

This is currently a work-in-progress!

## Use

```rust

let path = "tests/media/ff_silence.wav";

let mut w = WaveReader::open(path)?;
let length = w.frame_length()?;
let format = w.format()?;

let bext = w.broadcast_extension()?;
println!("Description field: {}", &bext.description);
println!("Originator field: {}", &bext.originator);

let frame_reader = w.audio_frame_reader()?;

let mut buffer: Vec<i32> = w.create_frame_buffer();
while( frame_reader.read_integer_frame(&mut buffer) > 0) {
    println!("Read frames {:?}", &buffer);
}

```

## Note on Testing

All of the media for the integration tests is committed to the respository
in either zipped form or is created by ffmpeg. Before you can run tests, you will
need to have ffmpeg installed on your host, and you will need to `cd` into the 
`tests` directory and run the `create_test_media.sh` script.
