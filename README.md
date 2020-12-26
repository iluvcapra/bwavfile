[![Crates.io](https://img.shields.io/crates/l/bwavfile)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/bwavfile)](https://crates.io/crates/bwavfile/)
![GitHub last commit](https://img.shields.io/github/last-commit/iluvcapra/bwavfile)
[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/iluvcapra/bwavfile/Rust)](https://github.com/iluvcapra/bwavfile/actions?query=workflow%3ARust)

# bwavfile
Rust Wave File Reader/Writer with Broadcast-WAV, MBWF and RF64 Support

### Features

This is currently a work-in-progress! However many features presently work:

| Feature                               | Read  | Write |
|---------------------------------------|:-----:|:-----:|
| Standard .wav files                   | ☑️   | ☑  ️ |
| Transparent promotion to RF64/BW64    | ☑️   |     |
| Unified interface for regular and extended Wave format | ☑️  |  |
| Channel/speaker map metadata          | ☑️   |     |
| Ambisonic B-format metadata           | ☑️   |     |
| EBU Broadcast-WAVE metadata           | ☑️   |     |
| Basic iXML/ADM metadata               | ☑️   |     |
| Enhanced iXML metadata support        |     |     |
| Broadcast-WAVE Level overview `levl` metadata     |    |    |
| Cue list metadata                     | ☑️   |     |
| Sampler and instrument metadata       |     |     |
| Enhanced Wave file form validation    | ☑ ️  |     |


## Use Examples

### Reading Audio Frames From a File

```rust

 use bwavfile::WaveReader;
 let mut r = WaveReader::open("tests/media/ff_silence.wav").unwrap();
 
 let format = r.format().unwrap();
 assert_eq!(format.sample_rate, 44100);
 assert_eq!(format.channel_count, 1);
 
 let mut buffer = format.create_frame_buffer();
 let mut frame_reader = r.audio_frame_reader().unwrap();
 
 let read = frame_reader.read_integer_frame(&mut buffer).unwrap();
 
 assert_eq!(buffer, [0i32]);
 assert_eq!(read, 1);
```

### Accessing Channel Descriptions

```rust
 use bwavfile::{WaveReader, ChannelMask};
 
 let mut f = WaveReader::open("tests/media/pt_24bit_51.wav").unwrap();
    
 let chans = f.channels().unwrap();
 assert_eq!(chans[0].index, 0);
 assert_eq!(chans[0].speaker, ChannelMask::FrontLeft);
 assert_eq!(chans[3].index, 3);
 assert_eq!(chans[3].speaker, ChannelMask::LowFrequency);
 assert_eq!(chans[4].speaker, ChannelMask::BackLeft);
```

## Note on Testing

All of the media for the integration tests is committed to the respository
in zipped form. Before you can run tests, you need to `cd` into the `tests` 
directory and run the `create_test_media.sh` script. Note that one of the 
test files (the RF64 test case) is over four gigs in size.
