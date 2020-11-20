
use std::fs::File;

use super::parser::Parser;
use super::fourcc::{FourCC, FMT__SIG, BEXT_SIG, DATA_SIG};
use super::errors::Error as ParserError;
use super::raw_chunk_reader::RawChunkReader;
use super::chunks::{WaveFmt, Bext};
use super::audio_frame_reader::AudioFrameReader;
use super::chunks::ReadBWaveChunks;
//use super::validation;
use std::io::SeekFrom::{Start};
use std::io::{Read, Seek};


/**
 * Wave, Broadcast-WAV and RF64/BW64 parser/reader.
 * 

*/
#[derive(Debug)]
pub struct WaveReader<R: Read + Seek> {
    pub inner: R,
}

impl WaveReader<File> {
    /**
     * Open a file for reading.
     * 
     * A convenience that opens `path` and calls `Self::new()`
     */
    pub fn open(path: &str) -> Result<Self, ParserError> {
        let inner = File::open(path)?;
        return Ok( Self::new(inner)? )
    }
}

impl<R: Read + Seek> WaveReader<R> {
    /**
     * Wrap a `Read` struct in a new `WaveReader`.
     * 
     * This is the primary entry point into the `WaveReader` interface. The
     * stream passed as `inner` must be at the beginning of the header of the
     * WAVE data. For a .wav file, this means it must be at the start of the 
     * file.
     * 
     * This function does a minimal validation on the provided stream and
     * will return an `Err(errors::Error)` immediately if there is a structural 
     * inconsistency that makes the stream unreadable or if it's missing 
     * essential components that make interpreting the audio data impoossible.
    */
    pub fn new(inner: R) -> Result<Self,ParserError> {
        let mut retval = Self { inner };
        retval.validate_readable()?;
        Ok(retval)
    }

    /**
     * Unwrap and reliqnish ownership of the inner reader.
     */
    pub fn into_inner(self) -> R {
        return self.inner;
    }

    /**
     * Create an `AudioFrameReader` for reading each audio frame.
    */
    pub fn audio_frame_reader(&mut self) -> Result<AudioFrameReader<RawChunkReader<R>>, ParserError> {
        let format = self.format()?;
        let audio_chunk_reader = self.chunk_reader(DATA_SIG, 0)?;
        Ok(AudioFrameReader::new(audio_chunk_reader, format))
    }

    /**
     * The count of audio frames in the file.
     */
    pub fn frame_length(&mut self) -> Result<u64, ParserError> {
        let (_, data_length ) = self.get_chunk_extent_at_index(DATA_SIG, 0)?;
        let format = self.format()?;
        Ok( data_length / (format.block_alignment as u64) )
    } 

    /**
     * Sample and frame format of this wave file.
     */
    pub fn format(&mut self) -> Result<WaveFmt, ParserError> {
        self.chunk_reader(FMT__SIG, 0)?.read_wave_fmt()
    }

    /**
     * The Broadcast-WAV metadata record for this file.
     */
    pub fn broadcast_extension(&mut self) -> Result<Bext, ParserError> {
        self.chunk_reader(BEXT_SIG, 0)?.read_bext()
    }
}

impl<R:Read+Seek> WaveReader<R> { /* Private Implementation */

    fn chunk_reader(&mut self, signature: FourCC, at_index: u32) -> Result<RawChunkReader<R>, ParserError> {
        let (start, length) = self.get_chunk_extent_at_index(signature, at_index)?;
        Ok( RawChunkReader::new(&mut self.inner, start, length) )
    } 

    pub fn get_chunk_extent_at_index(&mut self, fourcc: FourCC, index: u32) -> Result<(u64,u64), ParserError> {
        let p = Parser::make(&mut self.inner)?.into_chunk_list()?;

        if let Some(chunk) = p.iter().filter(|item| item.signature == fourcc).nth(index as usize) {
            Ok ((chunk.start, chunk.length))
        } else {
            Err( ParserError::ChunkMissing { signature : fourcc })
        }
    }
}
