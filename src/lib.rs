/*!
# bwavfile

Rust Wave File Reader/Writer with Broadcast-WAV, MBWF and RF64 Support

## Interfaces

### `WaveReader`

`WaveReader` can open and parse a Wave, Broadcast-Wave, or RF64/BW64 64-bit
wave file. Metadata can be accessed and parsed in arbitrary order and audio
samples can be accessed using the `AudioFrameReader` type, created by an
accessor method of `WaveReader`.

### `WaveWriter`

`WaveWriter` can create a new Wave, Broadcast-Wave, or RF64/BW64 64-bit wave
file. Metadata chunks and audio samples are added sequentially, write-only, to
a Wave file which is automatically promoted from standard Wave to RF64 wave
when the total WAVE form size exceeds 0xFFFFFFFF bytes.


## Objectives and Roadmap

This package aims to support read and writing any kind of WAV file you are likely
to encounter in a professional audio, motion picture production, broadcast, or music
production.

Apps we test against:
- Avid Pro Tools
- iZotope RX Audio Editor
- FFMpeg
- Audacity
- Sound Devices field recorders: 702T, MixPre-10 II

[github]: https://github.com/iluvcapra/bwavfile
*/

extern crate byteorder;
extern crate encoding;
extern crate uuid;

mod common_format;
mod errors;
mod fourcc;

mod list_form;
mod parser;

mod bext;
mod chunks;
mod cue;
mod fmt;

mod wavereader;
mod wavewriter;

pub use bext::Bext;
pub use common_format::CommonFormat;
pub use cue::Cue;
pub use errors::Error;
pub use fmt::{ADMAudioID, ChannelDescriptor, ChannelMask, WaveFmt, WaveFmtExtended};
pub use wavereader::{AudioFrameReader, WaveReader};
pub use wavewriter::{AudioFrameWriter, WaveWriter};
