use std::fs::File;
use std::io::{Write,Seek,SeekFrom};

use super::Error;
use super::fourcc::{FourCC, WriteFourCC, RIFF_SIG, WAVE_SIG, FMT__SIG, DATA_SIG, ELM1_SIG, JUNK_SIG, BEXT_SIG};
use super::fmt::WaveFmt;
//use super::common_format::CommonFormat;
use super::chunks::WriteBWaveChunks;
use super::bext::Bext;

use byteorder::LittleEndian;
use byteorder::WriteBytesExt;

/// Write audio frames to a `WaveWriter`.
/// 
/// 
pub struct AudioFrameWriter<W> where W: Write + Seek {
    inner : WaveChunkWriter<W>
}

impl<W> AudioFrameWriter<W> where W: Write + Seek {

    /// Write one audio frame.
    /// 
    pub fn write_integer_frame(&mut self, buffer: &[i32]) -> Result<u64,Error> {
        let format = self.inner.inner.format;
        assert!(buffer.len() as u16 == format.channel_count, 
            "read_integer_frame was called with a mis-sized buffer, expected {}, was {}", 
            format.channel_count, buffer.len());

        let framed_bits_per_sample = format.block_alignment * 8 / format.channel_count;

        for n in 0..(format.channel_count as usize) {
            match (format.bits_per_sample, framed_bits_per_sample) {
                (0..=8,8) => self.inner.write_u8((buffer[n] + 0x80) as u8 )?, // EBU 3285 §A2.2
                (9..=16,16) => self.inner.write_i16::<LittleEndian>(buffer[n] as i16)?,
                (10..=24,24) => self.inner.write_i24::<LittleEndian>(buffer[n])?,
                (25..=32,32) => self.inner.write_i32::<LittleEndian>(buffer[n])?,
                (b,_)=> panic!("Unrecognized integer format, bits per sample {}, channels {}, block_alignment {}", 
                    b, format.channel_count, format.block_alignment)
            }
        }
        self.inner.flush()?;
        Ok(1)
    }

    /// Finish writing audio frames and unwrap the inner `WaveWriter`.
    /// 
    /// This method must be called when the client has finished writing audio
    /// data. This will finalize the audio data chunk.
    pub fn end(self) -> Result<WaveWriter<W>, Error> {
        self.inner.end()
    }
}

/// Write a wave data chunk.
/// 
/// `WaveChunkWriter` implements `Write` and as bytes are written to it,
/// 
/// ### Important! 
/// 
/// When you are done writing to a chunk you must call `end()` in order to 
/// finalize the chunk for storage.
pub struct WaveChunkWriter<W> where W: Write + Seek {
    inner : WaveWriter<W>,
    content_start_pos : u64,
    length : u64
}

impl<W> WaveChunkWriter<W> where W: Write + Seek {

    fn begin(mut inner : WaveWriter<W>, ident : FourCC) -> Result<Self,Error> {
        let length : u64 = 0;
        inner.inner.write_fourcc(ident)?;
        inner.inner.write_u32::<LittleEndian>(length as u32)?;
        inner.increment_form_length(8)?;
        let content_start_pos = inner.inner.seek(SeekFrom::End(0))?;
        Ok( WaveChunkWriter { inner , content_start_pos, length } )
    }

    fn end(mut self) -> Result<WaveWriter<W>, Error> {
        if self.length % 2 == 1 {
            self.inner.inner.seek(SeekFrom::End(0))?;
            self.inner.inner.write(&[0u8])?;
            self.inner.increment_form_length(1)?;
        }
        Ok( self.inner )
    }

    fn increment_chunk_length(&mut self, amount: u64) -> Result<(), std::io::Error> {
        self.length = self.length + amount;
        if self.length < u32::MAX as u64 {
            self.inner.inner.seek(SeekFrom::Start(self.content_start_pos - 4))?;
            self.inner.inner.write_u32::<LittleEndian>(self.length as u32)?;
        } else {
            todo!("FIXME RF64 wave writing is not yet supported")
        }

        Ok(())
    }
}

impl<W> Write for WaveChunkWriter<W> where W: Write + Seek {

    fn write(&mut self, buffer: &[u8]) -> Result<usize, std::io::Error> { 
        self.inner.inner.seek(SeekFrom::End(0))?;
        let written = self.inner.inner.write(buffer)?;
        self.increment_chunk_length(written as u64)?;
        self.inner.increment_form_length(written as u64)?;

        Ok( written )
    }

    fn flush(&mut self) -> Result<(), std::io::Error> { 
        self.inner.inner.flush()
    }
}

/// Wave, Broadcast-WAV and RF64/BW64 writer.
/// 
/// A WaveWriter creates a new wave file at the given path (with `create()`)
/// or into the given `Write`- and `Seek`-able inner writer.
/// 
/// Audio is added to the wave file by starting the audio data chunk with
/// `WaveWriter::audio_frame_writer()`.
/// 
/// ```
/// use bwavfile::{WaveWriter,WaveFmt};
/// # use std::io::Cursor;
/// 
/// // Write a three-sample wave file to a cursor
/// let mut cursor = Cursor::new(vec![0u8;0]);
/// let format = WaveFmt::new_pcm_mono(48000, 24);
/// let w = WaveWriter::new(&mut cursor, format).unwrap();
///
/// let mut frame_writer = w.audio_frame_writer().unwrap();
///
/// frame_writer.write_integer_frame(&[0i32]).unwrap();
/// frame_writer.write_integer_frame(&[0i32]).unwrap();
/// frame_writer.write_integer_frame(&[0i32]).unwrap();
/// frame_writer.end().unwrap();
/// ``` 
pub struct WaveWriter<W> where W: Write + Seek {
    inner : W,
    form_length: u64,

    /// Format of the wave file.
    pub format: WaveFmt
}

impl WaveWriter<File> {

    /// Create a new Wave file at `path`.
    pub fn create(path : &str, format : WaveFmt) -> Result<Self, Error> {
        let f = File::create(path)?;
        Ok( Self::new(f, format)? )
    }
}

impl<W> WaveWriter<W> where W: Write + Seek {

    /// Wrap a writer in a Wave writer.
    /// 
    /// The inner writer will immediately have a RIFF WAVE file header 
    /// written to it along with the format descriptor (and possibly a `fact`
    /// chunk if appropriate).
    pub fn new(mut inner : W, format: WaveFmt) -> Result<Self, Error> {
        inner.write_fourcc(RIFF_SIG)?;
        inner.write_u32::<LittleEndian>(0)?;
        inner.write_fourcc(WAVE_SIG)?;

        let mut retval = WaveWriter { inner, form_length: 0, format};
        retval.increment_form_length(4)?;

        let mut chunk = retval.chunk(JUNK_SIG)?;
        chunk.write(&[0u8; 96])?;
        let retval = chunk.end()?;

        let mut chunk = retval.chunk(FMT__SIG)?;
        chunk.write_wave_fmt(&format)?;
        let retval = chunk.end()?;

        Ok( retval )
    }


    fn chunk(mut self, ident: FourCC) -> Result<WaveChunkWriter<W>,Error> {
        self.inner.seek(SeekFrom::End(0))?;
        WaveChunkWriter::begin(self, ident)
    }

    /// Write Broadcast-Wave metadata to the file.
    /// 
    /// This function will write the metadata chunk immediately to the end of 
    /// the file; if you have already written and closed the audio data the 
    /// bext chunk will be positioned after it.
    fn write_broadcast_metadata(self, bext: &Bext) -> Result<Self,Error> {
        let mut b = self.chunk(BEXT_SIG)?;
        b.write_bext(bext)?;
        Ok(b.end()?)
    }

    /// Create an audio frame writer, which takes possession of the callee 
    /// `WaveWriter`.
    /// 
    /// 
    pub fn audio_frame_writer(mut self) -> Result<AudioFrameWriter<W>, Error> {
        // append elm1 chunk

        let framing = 0x4000;

        let lip = self.inner.seek(SeekFrom::End(0))?;
        let to_add = framing - (lip % framing) - 16;
        let mut chunk = self.chunk(ELM1_SIG)?;
        let buf = vec![0u8; to_add as usize];
        chunk.write(&buf)?;
        let closed = chunk.end()?;

        let inner = closed.chunk(DATA_SIG)?;
        Ok( AudioFrameWriter { inner } )
    }

    fn increment_form_length(&mut self, amount: u64) -> Result<(), std::io::Error> {
        self.form_length = self.form_length + amount;
        if self.form_length < u32::MAX as u64 {
            self.inner.seek(SeekFrom::Start(4))?;
            self.inner.write_u32::<LittleEndian>(self.form_length as u32)?;
            Ok(())
        } else {
            todo!("FIXME RF64 wave writing is not yet supported")
        }

    }

}

#[test]
fn test_new() {
    use std::io::Cursor;
    use super::fourcc::ReadFourCC;
    use byteorder::ReadBytesExt;

    let mut cursor = Cursor::new(vec![0u8;0]);
    let format = WaveFmt::new_pcm_mono(4800, 24);
    WaveWriter::new(&mut cursor, format).unwrap();

    cursor.seek(SeekFrom::Start(0)).unwrap();

    assert_eq!(cursor.read_fourcc().unwrap(), RIFF_SIG);
    let form_size = cursor.read_u32::<LittleEndian>().unwrap();
    assert_eq!(cursor.read_fourcc().unwrap(), WAVE_SIG);

    assert_eq!(cursor.read_fourcc().unwrap(), JUNK_SIG);
    let junk_size = cursor.read_u32::<LittleEndian>().unwrap();
    assert_eq!(junk_size,96);
    cursor.seek(SeekFrom::Current(junk_size as i64)).unwrap();

    assert_eq!(cursor.read_fourcc().unwrap(), FMT__SIG);
    let fmt_size = cursor.read_u32::<LittleEndian>().unwrap();
    assert_eq!(form_size, 4 + 8 + junk_size + 8 + fmt_size);
}

#[test]
fn test_write_audio() {
    use std::io::Cursor;
    use super::fourcc::ReadFourCC;
    use byteorder::ReadBytesExt;

    let mut cursor = Cursor::new(vec![0u8;0]);
    let format = WaveFmt::new_pcm_mono(48000, 24);
    let w = WaveWriter::new(&mut cursor, format).unwrap();
    
    let mut frame_writer = w.audio_frame_writer().unwrap();

    frame_writer.write_integer_frame(&[0i32]).unwrap();
    frame_writer.write_integer_frame(&[0i32]).unwrap();
    frame_writer.write_integer_frame(&[0i32]).unwrap();

    frame_writer.end().unwrap();

    cursor.seek(SeekFrom::Start(0)).unwrap();

    cursor.seek(SeekFrom::Start(0)).unwrap();
    assert_eq!(cursor.read_fourcc().unwrap(), RIFF_SIG);
    let form_size = cursor.read_u32::<LittleEndian>().unwrap();

    assert_eq!(cursor.read_fourcc().unwrap(), WAVE_SIG); //4

    assert_eq!(cursor.read_fourcc().unwrap(), JUNK_SIG); //4
    let junk_size = cursor.read_u32::<LittleEndian>().unwrap(); //4
    cursor.seek(SeekFrom::Current(junk_size as i64)).unwrap();

    assert_eq!(cursor.read_fourcc().unwrap(), FMT__SIG); //4
    let fmt_size = cursor.read_u32::<LittleEndian>().unwrap(); //4
    cursor.seek(SeekFrom::Current(fmt_size as i64)).unwrap();

    assert_eq!(cursor.read_fourcc().unwrap(), ELM1_SIG); //4
    let elm1_size = cursor.read_u32::<LittleEndian>().unwrap(); //4
    cursor.seek(SeekFrom::Current(elm1_size as i64)).unwrap();

    assert_eq!(cursor.read_fourcc().unwrap(), DATA_SIG); //4
    let data_size = cursor.read_u32::<LittleEndian>().unwrap(); //4
    assert_eq!(data_size, 9);

    let tell = cursor.seek(SeekFrom::Current(0)).unwrap();
    assert!(tell % 0x4000 == 0);

    assert_eq!(form_size, 4 + 8 + junk_size + 8 + fmt_size + 8 + elm1_size + 8 + data_size + data_size % 2)
}

#[test]
fn test_write_bext() {
    use std::io::Cursor;

    let mut cursor = Cursor::new(vec![0u8;0]);
    let format = WaveFmt::new_pcm_mono(48000, 24);
    let w = WaveWriter::new(&mut cursor, format).unwrap();

    let bext = Bext {
        description: String::from("Test description"),
        originator: String::from(""),
        originator_reference: String::from(""),
        origination_date: String::from("2020-01-01"),
        origination_time: String::from("12:34:56"),
        time_reference: 0,
        version: 0,
        umid: None,
        loudness_value: None,
        loudness_range: None,
        max_true_peak_level: None,
        max_momentary_loudness: None,
        max_short_term_loudness: None,
        coding_history: String::from(""),
    };

    let w = w.write_broadcast_metadata(&bext).unwrap();

    let mut frame_writer = w.audio_frame_writer().unwrap();

    frame_writer.write_integer_frame(&[0i32]).unwrap();
    frame_writer.write_integer_frame(&[0i32]).unwrap();
    frame_writer.write_integer_frame(&[0i32]).unwrap();

    frame_writer.end().unwrap();
}