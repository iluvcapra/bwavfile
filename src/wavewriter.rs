use std::fs::File;
use std::io::{BufWriter, Cursor, Seek, SeekFrom, Write};
use std::path::Path;

use super::fmt::WaveFmt;
use super::fourcc::{
    FourCC, WriteFourCC, AXML_SIG, BEXT_SIG, DATA_SIG, DS64_SIG, ELM1_SIG, FMT__SIG, IXML_SIG,
    JUNK_SIG, RF64_SIG, RIFF_SIG, WAVE_SIG,
};
use super::Error;
//use super::common_format::CommonFormat;
use super::bext::Bext;
use super::chunks::WriteBWaveChunks;

use byteorder::LittleEndian;
use byteorder::WriteBytesExt;

/// Write audio frames to a `WaveWriter`.
///
///
pub struct AudioFrameWriter<W>
where
    W: Write + Seek,
{
    inner: WaveChunkWriter<W>,
}

impl<W> AudioFrameWriter<W>
where
    W: Write + Seek,
{
    fn new(inner: WaveChunkWriter<W>) -> Self {
        AudioFrameWriter { inner }
    }

    fn write_integer_frames_to_buffer(&self, from_frames: &[i32], to_buffer: &mut [u8]) -> () {
        assert!(
            from_frames.len() % self.inner.inner.format.channel_count as usize == 0,
            "frames buffer does not contain a number of samples % channel_count == 0"
        );
        self.inner.inner.format.pack_frames(&from_frames, to_buffer);
        ()
    }

    /// Write interleaved samples in `buffer`
    ///
    /// # Panics
    ///
    /// This function will panic if `buffer.len()` modulo the Wave file's channel count
    /// is not zero.
    pub fn write_integer_frames(&mut self, buffer: &[i32]) -> Result<u64, Error> {
        let mut write_buffer = self
            .inner
            .inner
            .format
            .create_raw_buffer(buffer.len() / self.inner.inner.format.channel_count as usize);

        self.write_integer_frames_to_buffer(&buffer, &mut write_buffer);

        self.inner.write(&write_buffer)?;
        self.inner.flush()?;
        Ok(write_buffer.len() as u64 / self.inner.inner.format.channel_count as u64)
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
pub struct WaveChunkWriter<W>
where
    W: Write + Seek,
{
    ident: FourCC,
    inner: WaveWriter<W>,
    content_start_pos: u64,
    length: u64,
}

impl<W> WaveChunkWriter<W>
where
    W: Write + Seek,
{
    fn begin(mut inner: WaveWriter<W>, ident: FourCC) -> Result<Self, Error> {
        let length: u64 = 0;
        inner.inner.write_fourcc(ident)?;
        inner.inner.write_u32::<LittleEndian>(length as u32)?;
        inner.increment_form_length(8)?;
        let content_start_pos = inner.inner.seek(SeekFrom::End(0))?;
        Ok(WaveChunkWriter {
            ident,
            inner,
            content_start_pos,
            length,
        })
    }

    fn end(mut self) -> Result<WaveWriter<W>, Error> {
        if self.length % 2 == 1 {
            self.inner.inner.seek(SeekFrom::End(0))?;
            self.inner.inner.write(&[0u8])?;
            self.inner.increment_form_length(1)?;
        }
        Ok(self.inner)
    }

    fn increment_chunk_length(&mut self, amount: u64) -> Result<(), std::io::Error> {
        self.length = self.length + amount;
        if !self.inner.is_rf64 {
            self.inner
                .inner
                .seek(SeekFrom::Start(self.content_start_pos - 4))?;
            self.inner
                .inner
                .write_u32::<LittleEndian>(self.length as u32)?;
        } else {
            if self.ident == DATA_SIG {
                let data_chunk_64bit_field_offset = 8 + 4 + 8 + 8;
                self.inner
                    .inner
                    .seek(SeekFrom::Start(self.content_start_pos - 4))?;
                self.inner.inner.write_u32::<LittleEndian>(0xFFFF_FFFF)?;
                // this only need to happen once, not every time we increment

                self.inner
                    .inner
                    .seek(SeekFrom::Start(data_chunk_64bit_field_offset))?;
                self.inner.inner.write_u64::<LittleEndian>(self.length)?;
            } else {
                todo!("FIXME RF64 wave writing is not yet supported for chunks other than `data`")
            }
        }

        Ok(())
    }
}

impl<W> Write for WaveChunkWriter<W>
where
    W: Write + Seek,
{
    fn write(&mut self, buffer: &[u8]) -> Result<usize, std::io::Error> {
        self.inner.inner.seek(SeekFrom::End(0))?;
        let written = self.inner.inner.write(buffer)?;
        self.inner.increment_form_length(written as u64)?;
        self.increment_chunk_length(written as u64)?;

        Ok(written)
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        self.inner.inner.flush()
    }
}

/// Wave, Broadcast-WAV and RF64/BW64 writer.
///
/// A `WaveWriter` creates a new wave file at the given path (with `create()`)
/// or into the given `Write`- and `Seek`-able inner writer.
///
/// Audio is added to the wave file by starting the audio data chunk with
/// `WaveWriter::audio_frame_writer()`. All of the functions that add chunks
/// move the WaveWriter and return it to the host when complete.
///
/// # Structure of New Wave Files
///
/// `WaveWriter` will create a Wave file with two chunks automatically: a 96
/// byte `JUNK` chunk and a standard `fmt ` chunk, which has the extended
/// length if the format your provided requires it. The first `JUNK` chunk is
/// a reservation for a `ds64` record which will be written over it if
/// the file needs to be upgraded to RF64 format.
///
/// Chunks are added to the file in the order the client adds them.
/// `audio_file_writer()` will add a `data` chunk for the audio data, and will
/// also add an `elm1` filler chunk prior to the data chunk to ensure that the
/// first byte of the data chunk's content is aligned with 0x4000.
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
/// frame_writer.write_integer_frames(&[0i32]).unwrap();
/// frame_writer.write_integer_frames(&[0i32]).unwrap();
/// frame_writer.write_integer_frames(&[0i32]).unwrap();
/// frame_writer.end().unwrap();
/// ```
///
/// ## Resources
///
/// ### Implementation of Wave Files
/// - [Peter Kabal, McGill University](http://www-mmsp.ece.mcgill.ca/Documents/AudioFormats/WAVE/WAVE.html)
/// - [Multimedia Programming Interface and Data Specifications 1.0](http://www-mmsp.ece.mcgill.ca/Documents/AudioFormats/WAVE/Docs/riffmci.pdf)
///   (August 1991), IBM Corporation and Microsoft Corporation
///  
/// ### Implementation of Broadcast Wave Files
/// - [EBU Tech 3285][ebu3285] (May 2011), "Specification of the Broadcast Wave Format (BWF)"
///   - [Supplement 1](https://tech.ebu.ch/docs/tech/tech3285s1.pdf) (July 1997): MPEG Audio
///   - [EBU Rec 68](https://tech.ebu.ch/docs/r/r068.pdf): Signal modulation and format constraints
///
/// ### Implementation of 64-bit Wave Files
/// - [ITU-R 2088][itu2088] (October 2019), "Long-form file format for the international exchange of audio programme materials with metadata"
///   - Presently in force, adopted by the EBU in [EBU Tech 3306v2][ebu3306v2] (June 2018).
/// - [EBU Tech 3306v1][ebu3306v1] (July 2009), "MBWF / RF64: An extended File Format for Audio"
///   - No longer in force, however long-established.
///
///
/// [ebu3285]: https://tech.ebu.ch/docs/tech/tech3285.pdf
/// [ebu3306v1]: https://tech.ebu.ch/docs/tech/tech3306v1_1.pdf
/// [ebu3306v2]: https://tech.ebu.ch/docs/tech/tech3306.pdf
/// [itu2088]: https://www.itu.int/dms_pubrec/itu-r/rec/bs/R-REC-BS.2088-1-201910-I!!PDF-E.pdf
/// [rfc3261]: https://tools.ietf.org/html/rfc2361
pub struct WaveWriter<W>
where
    W: Write + Seek,
{
    inner: W,
    form_length: u64,

    /// True if file is RF64
    pub is_rf64: bool,

    /// Format of the wave file.
    pub format: WaveFmt,
}

const DS64_RESERVATION_LENGTH: u32 = 96;

impl WaveWriter<BufWriter<File>> {
    /// Create a new Wave file at `path`.
    pub fn create<P: AsRef<Path>>(path: P, format: WaveFmt) -> Result<Self, Error> {
        let f = File::create(path)?;
        let b = BufWriter::new(f);
        Self::new(b, format)
    }
}

impl WaveWriter<File> {
    /// Creare a new Wave file with unbuffered IO at `path`
    pub fn create_unbuffered<P: AsRef<Path>>(path: P, format: WaveFmt) -> Result<Self, Error> {
        let f = File::create(path)?;
        Self::new(f, format)
    }
}

impl<W> WaveWriter<W>
where
    W: Write + Seek,
{
    /// Wrap a writer in a Wave writer.
    ///
    /// The inner writer will immediately have a RIFF WAVE file header
    /// written to it along with the format descriptor (and possibly a `fact`
    /// chunk if appropriate).
    pub fn new(mut inner: W, format: WaveFmt) -> Result<Self, Error> {
        inner.write_fourcc(RIFF_SIG)?;
        inner.write_u32::<LittleEndian>(0)?;
        inner.write_fourcc(WAVE_SIG)?;

        let mut retval = WaveWriter {
            inner,
            form_length: 0,
            is_rf64: false,
            format,
        };

        retval.increment_form_length(4)?;

        // write ds64_reservation
        retval.write_junk(DS64_RESERVATION_LENGTH)?;

        let mut chunk = retval.chunk(FMT__SIG)?;
        chunk.write_wave_fmt(&format)?;
        let retval = chunk.end()?;

        Ok(retval)
    }

    fn write_chunk(&mut self, ident: FourCC, data: &[u8]) -> Result<(), Error> {
        self.inner.seek(SeekFrom::End(0))?;
        self.inner.write_fourcc(ident)?;
        assert!(data.len() < u32::MAX as usize);
        self.inner.write_u32::<LittleEndian>(data.len() as u32)?;
        self.inner.write(data)?;
        if data.len() % 2 == 0 {
            self.increment_form_length(8 + data.len() as u64)?;
        } else {
            self.inner.write(&[0u8])?;
            self.increment_form_length(8 + data.len() as u64 + 1)?;
        }
        Ok(())
    }

    /// Write Broadcast-Wave metadata to the file.
    ///
    /// This function will write the metadata chunk immediately to the end of
    /// the file; if you have already written and closed the audio data the
    /// bext chunk will be positioned after it.
    pub fn write_broadcast_metadata(&mut self, bext: &Bext) -> Result<(), Error> {
        //FIXME Implement re-writing
        let mut c = Cursor::new(vec![0u8; 0]);
        c.write_bext(&bext)?;
        let buf = c.into_inner();
        self.write_chunk(BEXT_SIG, &buf)?;
        Ok(())
    }

    /// Write iXML metadata
    pub fn write_ixml(&mut self, ixml: &[u8]) -> Result<(), Error> {
        //FIXME Implement re-writing
        self.write_chunk(IXML_SIG, &ixml)
    }

    /// Write axml/ADM metadata
    pub fn write_axml(&mut self, axml: &[u8]) -> Result<(), Error> {
        //FIXME Implement re-writing
        self.write_chunk(AXML_SIG, &axml)
    }

    /// Write a `JUNK` filler chunk
    pub fn write_junk(&mut self, length: u32) -> Result<(), Error> {
        let filler = vec![0u8; length as usize];
        self.write_chunk(JUNK_SIG, &filler)
    }

    /// Create an audio frame writer, which takes possession of the callee
    /// `WaveWriter`.
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
        Ok(AudioFrameWriter::new(inner))
    }

    /// Open a wave chunk writer here
    fn chunk(mut self, ident: FourCC) -> Result<WaveChunkWriter<W>, Error> {
        self.inner.seek(SeekFrom::End(0))?;
        WaveChunkWriter::begin(self, ident)
    }

    /// Upgrade this file to RF64
    fn promote_to_rf64(&mut self) -> Result<(), std::io::Error> {
        if !self.is_rf64 {
            self.inner.seek(SeekFrom::Start(0))?;
            self.inner.write_fourcc(RF64_SIG)?;
            self.inner.write_u32::<LittleEndian>(0xFFFF_FFFF)?;
            self.inner.seek(SeekFrom::Start(12))?;

            self.inner.write_fourcc(DS64_SIG)?;
            self.inner.seek(SeekFrom::Current(4))?;
            self.inner.write_u64::<LittleEndian>(self.form_length)?;
            self.is_rf64 = true;
        }
        Ok(())
    }

    /// Add `amount` to the RIFF/RF64 form length
    fn increment_form_length(&mut self, amount: u64) -> Result<(), std::io::Error> {
        self.form_length = self.form_length + amount;
        if self.is_rf64 {
            self.inner.seek(SeekFrom::Start(8 + 4 + 8))?;
            self.inner.write_u64::<LittleEndian>(self.form_length)?;
        } else if self.form_length < u32::MAX as u64 {
            self.inner.seek(SeekFrom::Start(4))?;
            self.inner
                .write_u32::<LittleEndian>(self.form_length as u32)?;
        } else {
            self.promote_to_rf64()?;
        }
        Ok(())
    }
}

#[test]
fn test_new() {
    use super::fourcc::ReadFourCC;
    use byteorder::ReadBytesExt;
    use std::io::Cursor;

    let mut cursor = Cursor::new(vec![0u8; 0]);
    let format = WaveFmt::new_pcm_mono(4800, 24);
    WaveWriter::new(&mut cursor, format).unwrap();

    cursor.seek(SeekFrom::Start(0)).unwrap();

    assert_eq!(cursor.read_fourcc().unwrap(), RIFF_SIG);
    let form_size = cursor.read_u32::<LittleEndian>().unwrap();
    assert_eq!(cursor.read_fourcc().unwrap(), WAVE_SIG);

    assert_eq!(cursor.read_fourcc().unwrap(), JUNK_SIG);
    let junk_size = cursor.read_u32::<LittleEndian>().unwrap();
    assert_eq!(junk_size, 96);
    cursor.seek(SeekFrom::Current(junk_size as i64)).unwrap();

    assert_eq!(cursor.read_fourcc().unwrap(), FMT__SIG);
    let fmt_size = cursor.read_u32::<LittleEndian>().unwrap();
    assert_eq!(form_size, 4 + 8 + junk_size + 8 + fmt_size);
}

#[test]
fn test_write_audio() {
    use super::fourcc::ReadFourCC;
    use byteorder::ReadBytesExt;
    use std::io::Cursor;

    let mut cursor = Cursor::new(vec![0u8; 0]);
    let format = WaveFmt::new_pcm_mono(48000, 24);
    let w = WaveWriter::new(&mut cursor, format).unwrap();

    let mut frame_writer = w.audio_frame_writer().unwrap();

    frame_writer.write_integer_frames(&[0i32]).unwrap();
    frame_writer.write_integer_frames(&[0i32]).unwrap();
    frame_writer.write_integer_frames(&[0i32]).unwrap();

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

    assert_eq!(
        form_size,
        4 + 8 + junk_size + 8 + fmt_size + 8 + elm1_size + 8 + data_size + data_size % 2
    )
}

#[test]
fn test_write_bext() {
    use std::io::Cursor;

    let mut cursor = Cursor::new(vec![0u8; 0]);
    let format = WaveFmt::new_pcm_mono(48000, 24);
    let mut w = WaveWriter::new(&mut cursor, format).unwrap();

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

    w.write_broadcast_metadata(&bext).unwrap();

    let mut frame_writer = w.audio_frame_writer().unwrap();

    frame_writer.write_integer_frames(&[0i32]).unwrap();
    frame_writer.write_integer_frames(&[0i32]).unwrap();
    frame_writer.write_integer_frames(&[0i32]).unwrap();

    frame_writer.end().unwrap();
}

// NOTE! This test of RF64 writing takes several minutes to complete.
#[test]
fn test_create_rf64() {
    use super::fourcc::ReadFourCC;
    use byteorder::ReadBytesExt;

    let mut cursor = Cursor::new(vec![0u8; 0]);
    let format = WaveFmt::new_pcm_stereo(48000, 24);
    let w = WaveWriter::new(&mut cursor, format).unwrap();

    let buflen = 16000 as u64;

    let buf = vec![0i32; buflen as usize];

    let four_and_a_half_hours_of_frames = 48000 * 16_200;

    let mut af = w.audio_frame_writer().unwrap();

    for _ in 0..(four_and_a_half_hours_of_frames * format.channel_count as u64 / buflen) {
        af.write_integer_frames(&buf).unwrap();
    }
    af.end().unwrap();

    assert!(
        cursor.seek(SeekFrom::End(0)).unwrap() > 0xFFFF_FFFFu64,
        "internal test error, Created file is not long enough to be RF64"
    );
    let expected_data_length = four_and_a_half_hours_of_frames * format.block_alignment as u64;

    cursor.seek(SeekFrom::Start(0)).unwrap();
    assert_eq!(cursor.read_fourcc().unwrap(), RF64_SIG);
    assert_eq!(cursor.read_u32::<LittleEndian>().unwrap(), 0xFFFF_FFFF);
    assert_eq!(cursor.read_fourcc().unwrap(), WAVE_SIG);

    assert_eq!(cursor.read_fourcc().unwrap(), DS64_SIG);
    let ds64_size = cursor.read_u32::<LittleEndian>().unwrap();
    let form_size = cursor.read_u64::<LittleEndian>().unwrap();
    let data_size = cursor.read_u64::<LittleEndian>().unwrap();
    assert_eq!(data_size, expected_data_length);
    cursor
        .seek(SeekFrom::Current(ds64_size as i64 - 16))
        .unwrap();

    assert_eq!(cursor.read_fourcc().unwrap(), FMT__SIG);
    let fmt_size = cursor.read_u32::<LittleEndian>().unwrap();
    cursor
        .seek(SeekFrom::Current((fmt_size + fmt_size % 2) as i64))
        .unwrap();

    assert_eq!(cursor.read_fourcc().unwrap(), ELM1_SIG);
    let elm1_size = cursor.read_u32::<LittleEndian>().unwrap();
    let data_start = cursor
        .seek(SeekFrom::Current((elm1_size + elm1_size % 2) as i64))
        .unwrap();

    assert!(
        (data_start + 8) % 0x4000 == 0,
        "data content start is not aligned, starts at {}",
        data_start + 8
    );
    assert_eq!(cursor.read_fourcc().unwrap(), DATA_SIG);
    assert_eq!(cursor.read_u32::<LittleEndian>().unwrap(), 0xFFFF_FFFF);
    cursor.seek(SeekFrom::Current(data_size as i64)).unwrap();

    assert_eq!(
        4 + 8 + ds64_size as u64 + 8 + data_size + 8 + fmt_size as u64 + 8 + elm1_size as u64,
        form_size
    )
}
