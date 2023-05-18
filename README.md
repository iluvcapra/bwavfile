[![Crates.io](https://img.shields.io/crates/l/bwavfile)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/bwavfile)](https://crates.io/crates/bwavfile/)
![GitHub last commit](https://img.shields.io/github/last-commit/iluvcapra/bwavfile)
[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/iluvcapra/bwavfile/rust.yml?branch=master)](https://github.com/iluvcapra/bwavfile/actions?query=workflow%3ARust)

# bwavfile
Wave File Reader/Writer library in Rust, with Broadcast-WAV, RF64 and production metadata support


## Features

__bwavfile__ provides a reader `WaveReader` and writer type `WaveWriter` for 
reading and creating new wave audio files.

`WaveReader` and `WaveWriter` support:
  * A unified interface for standard RIFF and RF64/BW64 64-bit Wave files.
  * When using `WaveWriter`, wave files are transparently upgraded from RIFF
    to RF64 when required.
  * Unpacked reading and writing of Integer PCM and IEEE float audio data 
    formats.
  * A unified interface for standard `WaveFormat` and extended `WaveFormatEx`
    wave data format specification.

The library has extensive metadata support, with emphasis on film and video 
production metadata:
  * Broadcast-Wave metdata extension, including long description, originator 
    information, SMPTE UMID and coding history.
  * Reading and writing of embedded iXML and axml/ADM metadata.
  * Reading and writing of timed cues and and timed cue regions.
  * Multichannel, surround, and ambisonic audio data description including 
    surround channel maps, ADM `AudioTrackFormat`, `AudioChannelFormatRef` and 
    `AudioPackRef` data structures.


## Feature Roadmap

Some features that may be included in the future include:
  * Broadcast-Wave `levl` waveform overview data reading and writing.
  * Sampler and Instrument metadata.
  * Performance improvements.


## Use Examples

  * [blits](examples/blits.rs) shows how to use `WaveWriter` to create a new
    file with BLITS alignment tones.
  * [wave-inter](examples/wave-inter.rs) uses `WaveReader` and `WaveWriter` to
    interleave several input Wave files into a single polyphonic Wave file.
  * [wave-deinter](examples/wave-deinter.rs) uses `WaveReader` and `WaveWriter`
    to de-interleave an input Wave file into several monoarual Wave files.

## Note on Testing

All of the media for the integration tests is committed to the respository
in zipped form. Before you can run tests, you need to `cd` into the `tests` 
directory and run the `create_test_media.sh` script. Note that one of the 
test files (the RF64 test case) is over four gigs in size.

[rf64test]: https://github.com/iluvcapra/bwavfile/blob/1f8542a7efb481da076120bf8107032c5b48889d/src/wavewriter.rs#L399
