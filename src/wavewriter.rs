use std::io::{Write, Seek, SeekFrom};
use std::fs::File;
use std::io::Cursor;

use super::errors::Error;
use super::chunks::{WriteBWaveChunks};
use super::fmt::{WaveFmt};
use super::fourcc::{FourCC, RIFF_SIG, WAVE_SIG, FMT__SIG, JUNK_SIG, BEXT_SIG, DATA_SIG, FLLR_SIG, WriteFourCC};
use super::audio_frame_writer::AudioFrameWriter;

use byteorder::LittleEndian;
use byteorder::WriteBytesExt;

/// This isn't working yet, do not use.
pub struct WaveWriter<W> where W: Write + Seek {
    pub inner : W,
    pub form_size : u64,
    pub format : WaveFmt
}

impl WaveWriter<File> {
    fn create(path: &str, format: WaveFmt) -> Result<Self, Error>  {
        let f = File::create(path)?;
        Self::new(f, format)
    }
}

impl<W: Write + Seek> WaveWriter<W> {
    /// Wrap a `Write` struct with a wavewriter.
    fn new(inner : W, format: WaveFmt) -> Result<Self,Error> {
        let mut retval = Self { inner, form_size : 0 , format: format};
        retval.inner.seek(SeekFrom::Start(0))?;
        retval.inner.write_fourcc(RIFF_SIG)?;
        retval.inner.write_u32::<LittleEndian>(0)?;
        retval.inner.write_fourcc(WAVE_SIG)?;
        retval.update_form_size(4)?;
        retval.write_ds64_reservation()?;
        retval.write_format(format)?;
        Ok( retval )
    }
    
    /// Unwrap the inner writer.
    fn into_inner(self) -> W {
        return self.inner;
    }

    /// Return an AudioFrameWriter, consuming the reader.
    fn audio_frame_writer(mut self) -> Result<AudioFrameWriter<W>, Error> {

        let framing = 0x4000;
        self.append_data_framing_chunk(framing)?;

        self.inner.write_fourcc(DATA_SIG)?;
        self.inner.write_u32::<LittleEndian>(0u32)?;
        self.update_form_size(8)?;

        Ok( AudioFrameWriter::make(self) )
    }

    fn append_data_framing_chunk(&mut self, framing: u64) -> Result<(), Error> {
        let current_length = self.inner.seek(SeekFrom::End(0))?;
        let size_to_add = (current_length % framing) - 8;
        let chunk_size_to_add = size_to_add - 8;

        let buf = vec![ 0u8; chunk_size_to_add as usize];
        self.append_chunk(FLLR_SIG, &buf)?;

        Ok( () )
    }

    /// Append data as a new chunk to the wave file.
    fn append_chunk(&mut self, ident: FourCC, data: &[u8]) -> Result<(),Error> {
        assert!(data.len() < (u32::MAX as usize), 
            "append_chunk() can only be used for chunks sized less than u32::MAX");

        let chunk_length = data.len() as u32;
        let total_chunk_size : u64 = (8 + chunk_length + (chunk_length % 2)) as u64;
        self.inner.seek(SeekFrom::End(0))?;
        self.inner.write_fourcc(ident)?;
        self.inner.write_u32::<LittleEndian>(chunk_length)?;
        self.inner.write(data)?;
        if chunk_length % 2 > 0 { self.inner.write_u8(0)?; }
        self.update_form_size(total_chunk_size)?;
        Ok( () )
    }
}


impl<W: Write + Seek> WaveWriter<W> { /* Private implementation */

    fn write_ds64_reservation(&mut self) -> Result<(), Error> {
        let ds64_reservation_data = vec![0u8; 92];
        self.append_chunk(JUNK_SIG, &ds64_reservation_data)
    }

    fn write_format(&mut self, format: WaveFmt) -> Result<(), Error> {
        let mut buf : Vec<u8> = vec![];
        let mut cursor = Cursor::new(&mut buf);
        cursor.write_wave_fmt(&format)?;
        self.append_chunk(FMT__SIG, &buf)
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
        todo!()
    }
}
