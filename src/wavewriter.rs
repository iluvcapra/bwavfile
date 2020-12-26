use std::collections::HashMap;
use std::io::{Write, Seek, SeekFrom};
use std::fs::File;
use std::io::Cursor;

use super::errors::Error;
use super::chunks::WriteBWaveChunks;
use super::fmt::WaveFmt;

use super::common_format::CommonFormat;
use super::fourcc::{FourCC, RIFF_SIG, RF64_SIG, WAVE_SIG, FMT__SIG, JUNK_SIG, 
    DS64_SIG, FACT_SIG, FLLR_SIG, ELM1_SIG, WriteFourCC};

use super::bext::Bext;

//use super::audio_frame_writer::AudioFrameWriter;

use byteorder::LittleEndian;
use byteorder::WriteBytesExt;

enum WritingState {
    NoChunk,
    ChunkOpen { ident: FourCC, length: u64, length_field_pos: u64 }
}

/// This isn't working yet, do not use.
pub struct WaveWriter<W> where W: Write + Seek {
    pub format : WaveFmt,
    inner : W,
    form_size : u64,
    ds64_sizes : Option<HashMap<FourCC,u64>>,
    writing_state : WritingState,
}

impl WaveWriter<File> {
    pub fn create(path: &str, format: WaveFmt) -> Result<Self, Error>  {
        let f = File::create(path)?;
        Self::new(f, format)
    }
}

impl<W: Write + Seek> WaveWriter<W> {
    /// Wrap a `Write` struct with a wavewriter.
    pub fn new(inner : W, format: WaveFmt) -> Result<Self,Error> {
        let mut retval = Self { inner, form_size : 0, format: format, 
            ds64_sizes : None,
            writing_state : WritingState::NoChunk
        };

        retval.inner.seek(SeekFrom::Start(0))?;
        retval.inner.write_fourcc(RIFF_SIG)?;
        retval.inner.write_u32::<LittleEndian>(0)?;
        retval.inner.write_fourcc(WAVE_SIG)?;
        retval.update_form_size(4)?;
        retval.write_ds64_reservation()?;
        retval.write_format_chunk()?;
        if format.common_format() != CommonFormat::IntegerPCM {
            retval.write_fact_chunk()?;
        }
        retval.write_framing_filler()?;

        Ok( retval )
    }
    
    /// Unwrap the inner writer.
    pub fn into_inner(self) -> W {
        return self.inner;
    }

    pub fn begin_chunk(&mut self, ident : FourCC) -> Result<(),Error> {
        self.inner.seek(SeekFrom::End(0))?;
        self.inner.write_fourcc(ident)?;
        let length_field_pos = self.inner.seek(SeekFrom::Current(0))?;
        self.inner.write_u32::<LittleEndian>(0)?;
        self.writing_state = WritingState::ChunkOpen {ident, length: 0, length_field_pos };
        self.update_form_size(8)?;
        Ok( () )
    }

    pub fn append_data_to_chunk(&mut self, buffer : &[u8]) -> Result<u64,Error> {
        match self.writing_state {
            WritingState::ChunkOpen {ident: _, length, length_field_pos} => {
                self.inner.seek(SeekFrom::End(0))?;
                self.inner.write(buffer)?;
                self.inner.seek(SeekFrom::Start(length_field_pos))?;
                let new_length = length + buffer.len() as u64;
                if new_length >= (u32::MAX as u64) {
                    todo!();
                } else {
                    self.inner.write_u32::<LittleEndian>(new_length as u32)?;
                    self.update_form_size(buffer.len() as u64)?;
                }
                
                Ok(buffer.len() as u64)
            },
            _ => Err(Error::DataChunkNotPreparedForAppend)
        }
    }

    pub fn end_chunk(&mut self) -> Result<(), Error> {
        match self.writing_state {
            WritingState::ChunkOpen { ident:_, length, length_field_pos : _ } => {
                if length % 2 == 1 {
                    self.inner.seek(SeekFrom::End(0))?;
                    self.inner.write_u8(0)?;
                    self.update_form_size(1)?;
                }
                Ok(())
            },
            WritingState::NoChunk => Ok(())
        }
    }

}


impl<W: Write + Seek> WaveWriter<W> { /* Private implementation */

    fn write_format_chunk(&mut self) -> Result<(), Error> {
        let mut buf : Vec<u8> = vec![];
        let mut cursor = Cursor::new(&mut buf);
        cursor.write_wave_fmt(&self.format)?;
        self.append_chunk(FMT__SIG, &buf)
    }

    fn write_fact_chunk(&mut self) -> Result<(), Error> {
        self.append_chunk(FACT_SIG, &[0u8; 4])?;
        Ok(())
    }

    fn write_ds64_reservation(&mut self) -> Result<(),Error> {
        self.append_chunk(JUNK_SIG, &[0u8; 96])?;
        Ok(())
    }

    fn write_framing_filler(&mut self) -> Result<(),Error> {
        let framing = 0x4000;
        let current_length = self.inner.seek(SeekFrom::End(0))?;
        let size_to_add = framing - ((current_length % framing) - 8);
        let chunk_size_to_add = size_to_add - 8;

        let buf = vec![ 0u8; chunk_size_to_add as usize];
        self.append_chunk(ELM1_SIG, &buf)?;
        Ok( () )
    }

    fn append_chunk(&mut self, ident : FourCC, buffer: &[u8]) -> Result<(),Error> {
        self.begin_chunk(ident)?;
        self.append_data_to_chunk(buffer)?;
        self.end_chunk()?;
        Ok(())
    }

    fn update_form_size(&mut self, added_size: u64) -> Result<(),Error> {
        self.inner.seek(SeekFrom::Start(4))?;
        let new_size = added_size + self.form_size;
        if new_size < (u32::MAX as u64) {
            self.inner.write_u32::<LittleEndian>(new_size as u32)?;
        } else {
            self.update_form_size_ds64(new_size)?;
        }
        self.form_size = new_size;
        Ok( () )
    }
    
    fn update_form_size_ds64(&mut self, new_size: u64) -> Result<(), Error> {
        if self.ds64_sizes.is_none() {
            self.inner.seek(SeekFrom::Start(0))?;
            self.inner.write_fourcc(RF64_SIG)?;
            self.inner.seek(SeekFrom::Start(12))?;
            self.inner.write_fourcc(DS64_SIG)?;
            self.ds64_sizes = Some( HashMap::new() );
        }

        self.inner.seek(SeekFrom::Start(20))?;
        self.inner.write_u64::<LittleEndian>(new_size)?;

        Ok(())
    }
}

#[test]
fn test_simple_create() {
    use super::fourcc::ReadFourCC;
    use byteorder::ReadBytesExt;

    let buf = vec![0u8; 0];

    let mut cursor = Cursor::new(buf);
    let format = WaveFmt::new_pcm(48000, 24, 1);

    let w = WaveWriter::new(cursor, format).unwrap();

    cursor = w.into_inner();

    cursor.seek(SeekFrom::Start(0)).unwrap();

    assert_eq!( cursor.read_fourcc().unwrap(), RIFF_SIG);
    let form_size = cursor.read_u32::<LittleEndian>().unwrap();
    assert_eq!( cursor.read_fourcc().unwrap(), WAVE_SIG);

    assert_eq!( cursor.read_fourcc().unwrap(), JUNK_SIG);
    let junk_size = cursor.read_u32::<LittleEndian>().unwrap();
    cursor.seek(SeekFrom::Current(junk_size as i64)).unwrap();

    assert_eq!( cursor.read_fourcc().unwrap(), FMT__SIG);
    let fmt_size = cursor.read_u32::<LittleEndian>().unwrap();
    cursor.seek(SeekFrom::Current(fmt_size as i64)).unwrap();

    assert_eq!( cursor.read_fourcc().unwrap(), ELM1_SIG);
    let junk2_size = cursor.read_u32::<LittleEndian>().unwrap();
    cursor.seek(SeekFrom::Current(junk2_size as i64)).unwrap();

    assert_eq!( form_size , junk2_size + junk_size + fmt_size + 4 + 8 * 3);
}
