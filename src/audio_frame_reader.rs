use std::io::{Read, Seek};
use std::io::SeekFrom::{Start,};

use byteorder::LittleEndian;
use byteorder::ReadBytesExt;

use super::fmt::{WaveFmt};
use super::errors::Error;
use super::CommonFormat;
use super::raw_chunk_reader::RawChunkReader;

/// Read audio frames
/// 
/// The inner reader is interpreted as a raw audio data
/// bitstream having a format specified by `format`.
/// 
#[derive(Debug)]
pub struct AudioFrameReader<'a, R: Read + Seek> {
    inner : RawChunkReader<'a,R>,
    format: WaveFmt
}

impl<'a, R: Read + Seek> AudioFrameReader<'a, R> {

    /// Create a new `AudioFrameReader`
    /// 
    /// ### Panics
    /// 
    /// This method does a few sanity checks on the provided format
    /// parameter to confirm the `block_alignment` law is fulfilled
    /// and the format tag is readable by this implementation (only
    /// format 0x01 is supported at this time.) 
    pub fn new(inner: RawChunkReader<'a, R>, format: WaveFmt) -> Self {
        assert!(format.block_alignment * 8 == format.bits_per_sample * format.channel_count, 
            "Unable to read audio frames from packed formats: block alignment is {}, should be {}",
            format.block_alignment, (format.bits_per_sample / 8 ) * format.channel_count);
        

        assert!(format.common_format() == CommonFormat::IntegerPCM , 
                "Unsupported format tag {:?}", format.tag);
                
        AudioFrameReader { inner , format }
    }

    /// Locate the read position to a different frame
    /// 
    /// Seeks within the audio stream.
    /// 
    /// Returns the new location of the read position.
    pub fn locate(&mut self, to :u64) -> Result<u64,Error> {
        let position = to * self.format.block_alignment as u64;
        let seek_result = self.inner.seek(Start(position))?;
        Ok( seek_result / self.format.block_alignment as u64 )
    }

    /// Create a frame buffer sized to hold frames of the reader
    /// 
    /// This is a conveneince method that creates a `Vec<i32>` with
    /// as many elements as there are channels in the underlying stream. 
    pub fn create_frame_buffer(&self) -> Vec<i32> {
        vec![0i32; self.format.channel_count as usize]
    }

    /// Read a frame
    /// 
    /// A single frame is read from the audio stream and the read location
    /// is advanced one frame.
    /// 
    /// Regardless of the number of bits in the audio sample, this method
    /// always writes `i32` samples back to the buffer. These samples are 
    /// written back "right-aligned" so samples that are shorter than i32
    /// will leave the MSB bits empty. 
    /// 
    /// For example: A full-code sample in 16 bit (0xFFFF) will be written 
    /// back to the buffer as 0x0000FFFF.
    ///  
    /// 
    /// ### Panics
    /// 
    /// The `buffer` must have a number of elements equal to the number of 
    /// channels and this method will panic if this is not the case.
    pub fn read_integer_frame(&mut self, buffer:&mut [i32]) -> Result<u64,Error> {
        assert!(buffer.len() as u16 == self.format.channel_count, 
            "read_integer_frame was called with a mis-sized buffer, expected {}, was {}", 
            self.format.channel_count, buffer.len());

        let framed_bits_per_sample = self.format.block_alignment * 8 / self.format.channel_count;

        for n in 0..(self.format.channel_count as usize) {
            buffer[n] = match (self.format.bits_per_sample, framed_bits_per_sample) {
                (0..=8,8) => self.inner.read_u8()? as i32 - 0x80_i32, // EBU 3285 §A2.2
                (9..=16,16) => self.inner.read_i16::<LittleEndian>()? as i32,
                (10..=24,24) => self.inner.read_i24::<LittleEndian>()?,
                (25..=32,32) => self.inner.read_i32::<LittleEndian>()?,
                (b,_)=> panic!("Unrecognized integer format, bits per sample {}, channels {}, block_alignment {}", 
                    b, self.format.channel_count, self.format.block_alignment)
            }
        }

        Ok( 1 )
    }
}