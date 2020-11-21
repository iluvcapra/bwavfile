extern crate encoding;
extern crate byteorder;

/**!
 * bwavfile
 * 
 * Rust Wave File Reader/Writer with Broadcast-WAV, MBWF and RF64 Support
 * 
 * This crate is currently a work-in-progress (I'm using it to teach myself
 * rust) so the interface may change dramatically and not all features work.
 * 
 !*/


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