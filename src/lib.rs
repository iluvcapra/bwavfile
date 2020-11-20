extern crate encoding;
extern crate byteorder;

mod parser;
mod fourcc;
mod errors;

mod validation;

mod raw_chunk_reader;
mod audio_frame_reader;
mod chunks;

mod wavereader;
mod wavewriter;

pub use wavereader::{WaveReader};
pub use chunks::{WaveFmt,Bext};
pub use errors::Error;