use std::io::{Read, Seek};
use std::io::SeekFrom::{Start,Current,};

use byteorder::LittleEndian;
use byteorder::ReadBytesExt;

use super::fmt::{WaveFmt};
use super::errors::Error;
use super::CommonFormat;

/// Read audio frames
/// 
/// The inner reader is interpreted as a raw audio data
/// bitstream having a format specified by `format`.
/// 
#[derive(Debug)]
pub struct AudioFrameReader<R: Read + Seek> {
    inner : R,
    format: WaveFmt,
    start: u64,
    length: u64
}

impl<R: Read + Seek> AudioFrameReader<R> {

    /// Create a new `AudioFrameReader`
    /// 
    /// ### Panics
    /// 
    /// This method does a few sanity checks on the provided format
    /// parameter to confirm the `block_alignment` law is fulfilled
    /// and the format tag is readable by this implementation (only
    /// format 0x01 is supported at this time.) 
    pub fn new(mut inner: R, format: WaveFmt, start: u64, length: u64) -> Result<Self, Error> {
        assert!(format.block_alignment * 8 == format.bits_per_sample * format.channel_count, 
            "Unable to read audio frames from packed formats: block alignment is {}, should be {}",
            format.block_alignment, (format.bits_per_sample / 8 ) * format.channel_count);
        
        assert!(format.common_format() == CommonFormat::IntegerPCM , 
                "Unsupported format tag {:?}", format.tag);
        
        inner.seek(Start(start))?;
        Ok( AudioFrameReader { inner , format , start, length} )
    }

    /// Unwrap the inner reader.
    pub fn into_inner(self) -> R {
        self.inner
    }

    /// Locate the read position to a different frame
    /// 
    /// Seeks within the audio stream.
    /// 
    /// Returns the new location of the read position.
    /// 
    /// locate() behaves similarly to Read methods in that
    /// seeking after the end of the audio data is not an error.
    pub fn locate(&mut self, to :u64) -> Result<u64,Error> {
        let position = to * self.format.block_alignment as u64;
        let seek_result = self.inner.seek(Start(self.start + position))?;
        Ok( (seek_result - self.start) / self.format.block_alignment as u64 )
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

        let tell = self.inner.seek(Current(0))?;

        if (tell - self.start) < self.length {
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
        } else {
            Ok( 0 )
        }
    }
}