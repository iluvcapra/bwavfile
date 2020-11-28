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

impl WaveWriter<File> {
    pub fn create(path : &str, format:WaveFmt, broadcast_extension: Option<Bext>) -> Result<Self,Error> {
        let inner = File::create(path)?;
        Self::make(inner, format, broadcast_extension)
    }
}

impl<W:Write + Seek> WaveWriter<W> {

    pub fn make(inner : W, format: WaveFmt, broadcast_extension: Option<Bext>) -> Result<Self, Error> {
        let mut retval = Self { inner };
        retval.prepare_created(format, broadcast_extension)?;
        Ok(retval) 
    }

    fn prepare_created(&mut self, format : WaveFmt, broadcast_extension: Option<Bext>) -> Result<(),Error> {
        self.inner.write_fourcc(RIFF_SIG)?;
        self.inner.write_u32::<LittleEndian>(4)?;
        self.inner.write_fourcc(WAVE_SIG)?;
        let mut written : u64 = 4;

        let ds64_reservation = [0u8; 92];

        written += self.primitive_append_chunk(JUNK_SIG, &ds64_reservation)?;

        let fmt_data : Vec<u8> = {
            let mut c = Cursor::new(vec![]);
            c.write_wave_fmt(&format)?;
            c.into_inner()
        };

        written += self.primitive_append_chunk(FMT__SIG, &fmt_data)?;

        if let Some(bext) = broadcast_extension {
            let mut b = Cursor::new(vec![]);
            b.write_bext(&bext)?;
            let data = b.into_inner();
            written += self.primitive_append_chunk(BEXT_SIG, &data)?;
        }
        
        // show our work
        let desired_data_alignment = 0x4000;
        let data_fourcc_start = desired_data_alignment - 8;
        let current_position_from_start = written + 8;
        let data_pad_length = data_fourcc_start - current_position_from_start;

        let data_padding = vec![0u8; data_pad_length as usize];

        written += self.primitive_append_chunk(JUNK_SIG, &data_padding)?;

        self.inner.write_fourcc(DATA_SIG)?;
        self.inner.write_u32::<LittleEndian>(0)?;

        written += 8;
        self.inner.seek(SeekFrom::Start(4))?;
        self.inner.write_u32::<LittleEndian>(written as u32)?;

        Ok(())
    }

    fn primitive_append_chunk(&mut self, signature: FourCC, data: &[u8]) -> Result<u64,Error> {
        assert!((data.len() as u32) < u32::MAX, 
            "primitive_append_chunk called with a long data buffer");

        self.inner.write_fourcc(signature)?;
        self.inner.write_u32::<LittleEndian>(data.len() as u32)?;
        self.inner.write_all(&data)?;
        let padding : u64 = data.len() as u64 % 2;
        if padding == 1 {
            self.inner.write_u8(0)?;
        }

        Ok(8 + data.len() as u64 + padding)
    }
}

