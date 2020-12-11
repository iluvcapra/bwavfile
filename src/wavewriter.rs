use std::io::{Write, Seek, SeekFrom};
use std::fs::File;
use std::io::Cursor;

use super::errors::Error;
use super::chunks::{WriteBWaveChunks};
use super::bext::Bext;
use super::fmt::{WaveFmt};
use super::fourcc::{FourCC, RIFF_SIG, WAVE_SIG, FMT__SIG, JUNK_SIG, BEXT_SIG, DATA_SIG, WriteFourCC};

use byteorder::LittleEndian;
use byteorder::WriteBytesExt;

/// This isn't working yet, do not use.
pub struct WaveWriter<W> where W: Write + Seek {
    inner : W
}
