use std::io::{Read, Seek};
use std::io::SeekFrom::{Start,};

use byteorder::LittleEndian;
use byteorder::ReadBytesExt;

use super::chunks::WaveFmt;
use super::errors::Error;

#[derive(Debug)]
pub struct AudioFrameReader<R: Read + Seek> {
    inner : R,
    format: WaveFmt
}

impl<R: Read + Seek> AudioFrameReader<R> {
    /// Create a new AudioFrameReader, taking possession of a reader.
    pub fn new(inner: R, format: WaveFmt) -> Self {
        assert!(format.block_alignment * 8 == format.bits_per_sample * format.channel_count, 
            "Unable to read audio frames from packed formats: block alignment is {}, should be {}",
            format.block_alignment, (format.bits_per_sample / 8 ) * format.channel_count);
        
        assert!(format.tag == 1, "Unsupported format tag {}", format.tag);
        AudioFrameReader { inner , format }
    }

    pub fn locate(&mut self, to :u64) -> Result<u64,Error> {
        let position = to * self.format.block_alignment as u64;
        let seek_result = self.inner.seek(Start(position))?;
        Ok( seek_result / self.format.block_alignment as u64 )
    }

    pub fn create_frame_buffer(&self) -> Vec<i32> {
        vec![0i32; self.format.channel_count as usize]
    }

    pub fn read_integer_frame(&mut self, buffer:&mut [i32]) -> Result<(),Error> {
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

        Ok( () )
    }
}

