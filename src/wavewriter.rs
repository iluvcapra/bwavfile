use std::collections::HashMap;
use std::io::{Write, Seek, SeekFrom};
use std::fs::File;
use std::io::Cursor;

use super::errors::Error;
use super::chunks::WriteBWaveChunks;
use super::fmt::WaveFmt;

use super::common_format::CommonFormat;
use super::fourcc::{FourCC, RIFF_SIG, RF64_SIG, WAVE_SIG, FMT__SIG, JUNK_SIG, 
    DS64_SIG, FACT_SIG, DATA_SIG, BEXT_SIG, FLLR_SIG, WriteFourCC};

    use super::bext::Bext;

//use super::audio_frame_writer::AudioFrameWriter;

use byteorder::LittleEndian;
use byteorder::WriteBytesExt;

/// This isn't working yet, do not use.
pub struct WaveWriter<W> where W: Write + Seek {
    pub inner : W,
    pub form_size : u64,
    pub format : WaveFmt,
    pub ds64_sizes : Option<HashMap<FourCC,u64>>,
    pub bext_start : Option<u64>,
    pub fact_start : Option<u64>,
}

impl WaveWriter<File> {
    pub fn create(path: &str, format: WaveFmt, broadcast_extension: Option<Bext>) -> Result<Self, Error>  {
        let f = File::create(path)?;
        Self::new(f, format, broadcast_extension)
    }
}

impl<W: Write + Seek> WaveWriter<W> {
    /// Wrap a `Write` struct with a wavewriter.
    pub fn new(inner : W, format: WaveFmt, broadcast_extension : Option<Bext>) -> Result<Self,Error> {
        let mut retval = Self { inner, form_size : 0, format: format, 
            ds64_sizes : None,
            bext_start : None,
            fact_start : None,
        };

        retval.inner.seek(SeekFrom::Start(0))?;
        retval.inner.write_fourcc(RIFF_SIG)?;
        retval.inner.write_u32::<LittleEndian>(0)?;
        retval.inner.write_fourcc(WAVE_SIG)?;
        retval.update_form_size(4)?;

        retval.append_head_chunks(format, broadcast_extension)?;

        Ok( retval )
    }
    
    /// Unwrap the inner writer.
    pub fn into_inner(self) -> W {
        return self.inner;
    }

    pub fn audio_frame_writer() {

    }

}


impl<W: Write + Seek> WaveWriter<W> { /* Private implementation */

    fn append_head_chunks(&mut self, format : WaveFmt, broadcast_extension : Option<Bext>) -> Result<(),Error> {
        self.write_ds64_reservation()?;
        self.write_format(format)?;
        
        if format.common_format() != CommonFormat::IntegerPCM {
            self.append_fact_chunk()?;
        }
        
        if let Some(bext) = broadcast_extension {
            self.append_bext_chunk(bext)?;
        }

        self.append_data_framing_chunk(0x4000)?;
        
        Ok(())
    }

    fn append_fact_chunk(&mut self) -> Result<(),Error> {
        let buf = vec![0u8; 4];
        self.fact_start = Some( self.inner.seek(SeekFrom::Current(0))? + 8);
        self.append_chunk(FACT_SIG, &buf)?;
        Ok(())
    }

    fn append_bext_chunk(&mut self, bext: Bext) -> Result<(),Error> {
        let buf = vec![0u8;0];
        let mut cursor = Cursor::new(buf);
        cursor.write_bext(&bext)?;
        let buf = cursor.into_inner();
        self.bext_start = Some( self.inner.seek(SeekFrom::Current(0))? + 8);
        self.append_chunk(BEXT_SIG, &buf)?;
        Ok(())
    }

    fn append_data_framing_chunk(&mut self, framing: u64) -> Result<(), Error> {
        let current_length = self.inner.seek(SeekFrom::End(0))?;
        let size_to_add = framing - ((current_length % framing) - 8);
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

    let w = WaveWriter::new(cursor, format, None).unwrap();

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

    assert_eq!( cursor.read_fourcc().unwrap(), FLLR_SIG);
    let junk2_size = cursor.read_u32::<LittleEndian>().unwrap();
    cursor.seek(SeekFrom::Current(junk2_size as i64)).unwrap();

    assert_eq!( form_size , junk2_size + junk_size + fmt_size + 4 + 8 * 3);
}
