use std::fs::File;

use std::path::Path;

use std::io::Cursor;
use std::io::SeekFrom;
use std::io::SeekFrom::Start;
use std::io::{BufReader, Read, Seek};

use super::bext::Bext;
use super::chunks::ReadBWaveChunks;
use super::cue::Cue;
use super::errors::Error as ParserError;
use super::errors::Error;
use super::fmt::{ChannelDescriptor, ChannelMask, WaveFmt};
use super::fourcc::{
    FourCC, ReadFourCC, ADTL_SIG, AXML_SIG, BEXT_SIG, CUE__SIG, DATA_SIG, FLLR_SIG, FMT__SIG,
    IXML_SIG, JUNK_SIG, LIST_SIG,
};
use super::parser::Parser;
use super::{CommonFormat, Sample, I24};

use byteorder::LittleEndian;
use byteorder::ReadBytesExt;

use dasp_sample::Sample as _; // Expose to_sample()

/// Read audio frames
///
/// The inner reader is interpreted as a raw audio data
/// bitstream having a format specified by `format`.
///
#[derive(Debug)]
pub struct AudioFrameReader<R: Read + Seek> {
    inner: R,
    format: WaveFmt,
    start: u64,
    length: u64,
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
        assert!(
            format.block_alignment * 8 == format.bits_per_sample * format.channel_count,
            "Unable to read audio frames from packed formats: block alignment is {}, should be {}",
            format.block_alignment,
            (format.bits_per_sample / 8) * format.channel_count
        );

        assert!(
            format.common_format() == CommonFormat::IntegerPCM
                || format.common_format() == CommonFormat::IeeeFloatPCM,
            "Unsupported format tag {:?}",
            format.tag
        );

        inner.seek(Start(start))?;
        Ok(AudioFrameReader {
            inner,
            format,
            start,
            length,
        })
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
    pub fn locate(&mut self, to: u64) -> Result<u64, Error> {
        let position = to * self.format.block_alignment as u64;
        let seek_result = self.inner.seek(Start(self.start + position))?;
        Ok((seek_result - self.start) / self.format.block_alignment as u64)
    }

    /// Reads frames from the file into the provided buffer
    ///
    /// The function will attempt to fill the buffer, but will stop without error when the end of
    /// the file is reached.
    ///
    /// The reader will convert from the file's sample type into the buffer's sample type.
    /// Note that no dithering will be applied during sample type conversion,
    /// if dithering is required then it will need to be applied manually.
    ///
    /// The return value is the number of frames read into the buffer.
    pub fn read_frames<S>(&mut self, buffer: &mut [S]) -> Result<u64, Error>
    where
        S: Sample,
    {
        use CommonFormat::*;

        let channel_count = self.format.channel_count as usize;
        let common_format = self.format.common_format();
        let bits_per_sample = self.format.bits_per_sample;

        if buffer.len() % channel_count != 0 {
            return Err(Error::InvalidBufferSize {
                buffer_size: buffer.len(),
                channel_count: self.format.channel_count,
            });
        }

        let position = self.inner.stream_position()? - self.start;
        let frames_requested = (buffer.len() / channel_count) as u64;
        let bytes_per_frame = self.format.block_alignment as u64;
        let frames_remaining = (self.length - position) / bytes_per_frame;
        let frames_to_read = frames_requested.min(frames_remaining);
        let samples_to_read = frames_to_read as usize * channel_count;

        match (common_format, bits_per_sample) {
            (IntegerPCM, 8) => read_into_buffer(samples_to_read, buffer, || {
                Ok(self.inner.read_u8()?.to_sample())
            }),
            (IntegerPCM, 16) => read_into_buffer(samples_to_read, buffer, || {
                Ok(self.inner.read_i16::<LittleEndian>()?.to_sample())
            }),
            (IntegerPCM, 24) => read_into_buffer(samples_to_read, buffer, || {
                Ok(I24::from(self.inner.read_i24::<LittleEndian>()?).to_sample())
            }),
            (IntegerPCM, 32) => read_into_buffer(samples_to_read, buffer, || {
                Ok(self.inner.read_i32::<LittleEndian>()?.to_sample())
            }),
            (IeeeFloatPCM, 32) => read_into_buffer(samples_to_read, buffer, || {
                Ok(self.inner.read_f32::<LittleEndian>()?.to_sample())
            }),
            (_, _) => panic!(
                "Unsupported format, bits per sample {}, channels {}, sample format: {:?}",
                bits_per_sample, channel_count, common_format
            ),
        }?;

        Ok(frames_to_read)
    }
}

fn read_into_buffer<S, F>(
    sample_count: usize,
    buffer: &mut [S],
    mut read_fn: F,
) -> Result<(), Error>
where
    F: FnMut() -> Result<S, Error>,
{
    for output in buffer.iter_mut().take(sample_count) {
        *output = read_fn()?;
    }

    Ok(())
}

/// Wave, Broadcast-WAV and RF64/BW64 parser/reader.
///
/// ```
/// use bwavfile::WaveReader;
/// let mut r = WaveReader::open("tests/media/ff_silence.wav").unwrap();
///
/// let format = r.format().unwrap();
/// assert_eq!(format.sample_rate, 44100);
/// assert_eq!(format.channel_count, 1);
///
/// let mut frame_reader = r.audio_frame_reader().unwrap();
/// let mut buffer = format.create_frame_buffer::<i32>(1);
///
/// let read = frame_reader.read_frames(&mut buffer).unwrap();
///
/// assert_eq!(buffer, [0i32]);
/// assert_eq!(read, 1);
///
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

#[derive(Debug)]
pub struct WaveReader<R: Read + Seek> {
    pub inner: R,
}

impl WaveReader<BufReader<File>> {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, ParserError> {
        let f = File::open(path)?;
        let inner = BufReader::new(f);
        Self::new(inner)
    }
}

impl WaveReader<File> {
    /// Open a file for reading with unbuffered IO.
    ///
    /// A convenience that opens `path` and calls `Self::new()`
    pub fn open_unbuffered<P: AsRef<Path>>(path: P) -> Result<Self, ParserError> {
        let inner = File::open(path)?;
        Self::new(inner)
    }
}

impl<R: Read + Seek> WaveReader<R> {
    /// Wrap a `Read` struct in a new `WaveReader`.
    ///
    /// This is the primary entry point into the `WaveReader` interface. The
    /// stream passed as `inner` must be at the beginning of the header of the
    /// WAVE data. For a .wav file, this means it must be at the start of the
    /// file.
    ///
    /// This function does a minimal validation on the provided stream and
    /// will return an `Err(errors::Error)` immediately if there is a structural
    /// inconsistency that makes the stream unreadable or if it's missing
    /// essential components that make interpreting the audio data impossible.

    /// ```rust
    /// use std::fs::File;
    /// use std::io::{Error,ErrorKind};
    /// use bwavfile::{WaveReader, Error as WavError};
    ///
    /// let f = File::open("tests/media/error.wav").unwrap();
    ///
    /// let reader = WaveReader::new(f);
    ///
    /// match reader {
    ///      Ok(_) => panic!("error.wav should not be openable"),
    ///      Err( WavError::IOError( e ) ) => {
    ///          assert_eq!(e.kind(), ErrorKind::UnexpectedEof)
    ///      }
    ///      Err(e) => panic!("Unexpected error was returned {:?}", e)
    /// }
    ///
    /// ```
    pub fn new(inner: R) -> Result<Self, ParserError> {
        let mut retval = Self { inner };
        retval.validate_readable()?;
        Ok(retval)
    }

    /// Unwrap the inner reader.
    pub fn into_inner(self) -> R {
        self.inner
    }

    ///
    /// Create an `AudioFrameReader` for reading each audio frame and consume the `WaveReader`.
    ///
    pub fn audio_frame_reader(mut self) -> Result<AudioFrameReader<R>, ParserError> {
        let format = self.format()?;
        let audio_chunk_reader = self.get_chunk_extent_at_index(DATA_SIG, 0)?;
        AudioFrameReader::new(
            self.inner,
            format,
            audio_chunk_reader.0,
            audio_chunk_reader.1,
        )
    }

    /// The count of audio frames in the file.
    pub fn frame_length(&mut self) -> Result<u64, ParserError> {
        let (_, data_length) = self.get_chunk_extent_at_index(DATA_SIG, 0)?;
        let format = self.format()?;
        Ok(data_length / (format.block_alignment as u64))
    }

    /// Sample and frame format of this wave file.
    ///
    pub fn format(&mut self) -> Result<WaveFmt, ParserError> {
        let (start, _) = self.get_chunk_extent_at_index(FMT__SIG, 0)?;
        self.inner.seek(SeekFrom::Start(start))?;
        self.inner.read_wave_fmt()
    }

    /// The Broadcast-WAV metadata record for this file, if present.
    ///
    pub fn broadcast_extension(&mut self) -> Result<Option<Bext>, ParserError> {
        let mut bext_buff: Vec<u8> = vec![];
        let result = self.read_chunk(BEXT_SIG, 0, &mut bext_buff)?;
        if result > 0 {
            let mut bext_cursor = Cursor::new(bext_buff);
            Ok(Some(bext_cursor.read_bext()?))
        } else {
            Ok(None)
        }
    }

    /// Describe the channels in this file
    ///
    /// Returns a vector of channel descriptors, one for each channel
    ///
    /// ```rust
    /// use bwavfile::WaveReader;
    /// use bwavfile::ChannelMask;
    ///
    /// let mut f = WaveReader::open("tests/media/pt_24bit_51.wav").unwrap();
    ///
    /// let chans = f.channels().unwrap();
    /// assert_eq!(chans[0].index, 0);
    /// assert_eq!(chans[0].speaker, ChannelMask::FrontLeft);
    /// assert_eq!(chans[3].index, 3);
    /// assert_eq!(chans[3].speaker, ChannelMask::LowFrequency);
    /// assert_eq!(chans[4].speaker, ChannelMask::BackLeft);
    /// ```
    pub fn channels(&mut self) -> Result<Vec<ChannelDescriptor>, ParserError> {
        let format = self.format()?;
        let channel_masks: Vec<ChannelMask> = match (format.channel_count, format.extended_format) {
            (1, _) => vec![ChannelMask::FrontCenter],
            (2, _) => vec![ChannelMask::FrontLeft, ChannelMask::FrontRight],
            (n, Some(x)) => ChannelMask::channels(x.channel_mask, n),
            (n, _) => vec![ChannelMask::DirectOut; n as usize],
        };

        Ok((0..format.channel_count)
            .zip(channel_masks)
            .map(|(i, m)| ChannelDescriptor {
                index: i,
                speaker: m,
                adm_track_audio_ids: vec![],
            })
            .collect())
    }

    /// Read cue points.
    ///
    /// ```rust
    /// use bwavfile::WaveReader;
    /// use bwavfile::Cue;
    ///
    /// let mut f = WaveReader::open("tests/media/izotope_test.wav").unwrap();
    /// let cue_points = f.cue_points().unwrap();
    ///
    /// assert_eq!(cue_points.len(), 3);
    /// assert_eq!(cue_points[0].frame, 12532);
    /// assert_eq!(cue_points[0].length, None);
    /// assert_eq!(cue_points[0].label, Some(String::from("Marker 1")));
    /// assert_eq!(cue_points[0].note, Some(String::from("Marker 1 Comment")));
    ///
    /// assert_eq!(cue_points[1].frame, 20997);
    /// assert_eq!(cue_points[1].length, None);
    /// assert_eq!(cue_points[1].label, Some(String::from("Marker 2")));
    /// assert_eq!(cue_points[1].note, Some(String::from("Marker 2 Comment")));
    ///
    /// assert_eq!(cue_points[2].frame, 26711);
    /// assert_eq!(cue_points[2].length, Some(6465));
    /// assert_eq!(cue_points[2].label, Some(String::from("Timed Region")));
    /// assert_eq!(cue_points[2].note, Some(String::from("Region Comment")));
    ///
    /// ```
    pub fn cue_points(&mut self) -> Result<Vec<Cue>, ParserError> {
        let mut cue_buffer: Vec<u8> = vec![];
        let mut adtl_buffer: Vec<u8> = vec![];

        let cue_read = self.read_chunk(CUE__SIG, 0, &mut cue_buffer)?;
        let adtl_read = self.read_list(ADTL_SIG, &mut adtl_buffer)?;

        match (cue_read, adtl_read) {
            (0, _) => Ok(vec![]),
            (_, 0) => Ok(Cue::collect_from(&cue_buffer, None)?),
            (_, _) => Ok(Cue::collect_from(&cue_buffer, Some(&adtl_buffer))?),
        }
    }

    /// Read iXML data.
    ///
    /// The iXML data will be appended to `buffer`.
    /// If there are no iXML metadata present in the file,
    /// Ok(0) will be returned.
    pub fn read_ixml(&mut self, buffer: &mut Vec<u8>) -> Result<usize, ParserError> {
        self.read_chunk(IXML_SIG, 0, buffer)
    }

    /// Read AXML data.
    ///
    /// The axml data will be appended to `buffer`. By convention this will
    /// generally be ADM metadata.
    ///
    /// If there are no axml metadata present in the file,
    /// Ok(0) will be returned
    pub fn read_axml(&mut self, buffer: &mut Vec<u8>) -> Result<usize, ParserError> {
        self.read_chunk(AXML_SIG, 0, buffer)
    }

    /**
     * Validate file is readable.
     *
     *  `Ok(())` if the source meets the minimum standard of
     *  readability by a permissive client:
     *  - `fmt` chunk and `data` chunk are present
     *  - `fmt` chunk appears before `data` chunk
     */
    pub fn validate_readable(&mut self) -> Result<(), ParserError> {
        let (fmt_pos, _) = self.get_chunk_extent_at_index(FMT__SIG, 0)?;
        let (data_pos, _) = self.get_chunk_extent_at_index(DATA_SIG, 0)?;

        if fmt_pos < data_pos {
            Ok(())
        } else {
            Err(ParserError::FmtChunkAfterData)
        }
    }

    /// Validate minimal WAVE file.
    ///
    /// `Ok(())` if the source is `validate_readable()` AND
    ///
    ///   - Contains _only_ a `fmt` chunk and `data` chunk, with no other chunks present
    ///   - `fmt` chunk is exactly 16 bytes long and begins _exactly_ at file offset 12
    ///   - `data` content begins _exactly_ at file offset 36
    ///   - is not an RF64/BW64
    ///
    /// Some clients require a WAVE file to only contain format and data without any other
    /// metadata and this function is provided to validate this condition.
    ///
    /// ### Examples
    ///
    /// ```
    /// # use bwavfile::WaveReader;
    ///
    /// let mut w = WaveReader::open("tests/media/ff_minimal.wav").unwrap();
    /// w.validate_minimal().expect("Minimal wav did not validate not minimal!");
    /// ```
    ///
    /// ```
    /// # use bwavfile::WaveReader;
    ///
    /// let mut x = WaveReader::open("tests/media/pt_24bit_51.wav").unwrap();
    /// x.validate_minimal().expect_err("Complex WAV validated minimal!");
    /// ```
    pub fn validate_minimal(&mut self) -> Result<(), ParserError> {
        self.validate_readable()?;

        let chunk_fourccs: Vec<FourCC> = Parser::make(&mut self.inner)?
            .into_chunk_list()?
            .iter()
            .map(|c| c.signature)
            .collect();

        if chunk_fourccs == vec![FMT__SIG, DATA_SIG] {
            Ok(()) /* FIXME: finish implementation */
        } else {
            Err(ParserError::NotMinimalWaveFile)
        }
    }

    /// Validate Broadcast-WAVE file format
    ///
    /// Returns `Ok(())` if `validate_readable()` and file contains a
    /// Broadcast-WAV metadata record (a `bext` chunk).
    ///
    /// ### Examples
    ///
    /// ```
    /// # use bwavfile::WaveReader;
    ///
    /// let mut w = WaveReader::open("tests/media/ff_bwav_stereo.wav").unwrap();
    /// w.validate_broadcast_wave().expect("BWAVE file did not validate BWAVE");
    ///
    /// let mut x = WaveReader::open("tests/media/pt_24bit.wav").unwrap();
    /// x.validate_broadcast_wave().expect("BWAVE file did not validate BWAVE");
    ///
    /// let mut y = WaveReader::open("tests/media/audacity_16bit.wav").unwrap();
    /// y.validate_broadcast_wave().expect_err("Plain WAV file DID validate BWAVE");
    /// ```
    ///
    pub fn validate_broadcast_wave(&mut self) -> Result<(), ParserError> {
        self.validate_readable()?;
        let (_, _) = self.get_chunk_extent_at_index(BEXT_SIG, 0)?;
        Ok(())
    }

    ///
    /// Verify data is aligned to a block boundary.
    ///
    /// Returns `Ok(())` if `validate_readable()` and the start of the
    /// `data` chunk's content begins at 0x4000.
    pub fn validate_data_chunk_alignment(&mut self) -> Result<(), ParserError> {
        self.validate_readable()?;
        let (start, _) = self.get_chunk_extent_at_index(DATA_SIG, 0)?;
        if start == 0x4000 {
            Ok(())
        } else {
            Err(ParserError::DataChunkNotAligned)
        }
    }

    /// Verify audio data can be appended immediately to this file.
    ///
    /// Returns `Ok(())` if:
    ///  - `validate_readable()`
    ///  - there is a `JUNK` or `FLLR` immediately at the beginning of the chunk
    ///    list adequately large enough to be overwritten by a `ds64` (92 bytes)
    ///  - `data` is the final chunk
    pub fn validate_prepared_for_append(&mut self) -> Result<(), ParserError> {
        self.validate_readable()?;

        let chunks = Parser::make(&mut self.inner)?.into_chunk_list()?;
        let ds64_space_required = 92;

        let eligible_filler_chunks = chunks
            .iter()
            .take_while(|c| c.signature == JUNK_SIG || c.signature == FLLR_SIG);

        let filler = eligible_filler_chunks
            .enumerate()
            .fold(0, |accum, (n, item)| {
                if n == 0 {
                    accum + item.length
                } else {
                    accum + item.length + 8
                }
            });

        if filler < ds64_space_required {
            Err(ParserError::InsufficientDS64Reservation {
                expected: ds64_space_required,
                actual: filler,
            })
        } else {
            let data_pos = chunks.iter().position(|c| c.signature == DATA_SIG);

            match data_pos {
                Some(p) if p == chunks.len() - 1 => Ok(()),
                _ => Err(ParserError::DataChunkNotPreparedForAppend),
            }
        }
    }
}

impl<R: Read + Seek> WaveReader<R> {
    // Private implementation
    //
    // As time passes this get smore obnoxious because I haven't implemented recursive chunk
    // parsing in the raw parser and I'm working around it

    // fn chunk_reader(&mut self, signature: FourCC, at_index: u32) -> Result<RawChunkReader<R>, ParserError> {
    //     let (start, length) = self.get_chunk_extent_at_index(signature, at_index)?;
    //     Ok( RawChunkReader::new(&mut self.inner, start, length) )
    // }

    fn read_list(&mut self, ident: FourCC, buffer: &mut Vec<u8>) -> Result<usize, ParserError> {
        if let Some(index) = self.get_list_form(ident)? {
            self.read_chunk(LIST_SIG, index, buffer)
        } else {
            Ok(0)
        }
    }

    fn read_chunk(
        &mut self,
        ident: FourCC,
        at: u32,
        buffer: &mut Vec<u8>,
    ) -> Result<usize, ParserError> {
        match self.get_chunk_extent_at_index(ident, at) {
            Ok((start, length)) => {
                buffer.resize(length as usize, 0x0);
                self.inner.seek(SeekFrom::Start(start))?;
                self.inner.read(buffer).map_err(ParserError::IOError)
            }
            Err(ParserError::ChunkMissing { signature: _ }) => Ok(0),
            Err(any) => Err(any),
        }
    }

    /// Extent of every chunk with the given fourcc
    fn get_chunks_extents(&mut self, fourcc: FourCC) -> Result<Vec<(u64, u64)>, ParserError> {
        let p = Parser::make(&mut self.inner)?.into_chunk_list()?;

        Ok(p.iter()
            .filter(|item| item.signature == fourcc)
            .map(|item| (item.start, item.length))
            .collect())
    }

    /// Index of first LIST for with the given FORM fourcc
    fn get_list_form(&mut self, fourcc: FourCC) -> Result<Option<u32>, ParserError> {
        for (n, (start, _)) in self.get_chunks_extents(LIST_SIG)?.iter().enumerate() {
            self.inner.seek(SeekFrom::Start(*start))?;
            let this_fourcc = self.inner.read_fourcc()?;
            if this_fourcc == fourcc {
                return Ok(Some(n as u32));
            }
        }

        Ok(None)
    }

    fn get_chunk_extent_at_index(
        &mut self,
        fourcc: FourCC,
        index: u32,
    ) -> Result<(u64, u64), ParserError> {
        if let Some((start, length)) = self.get_chunks_extents(fourcc)?.get(index as usize) {
            Ok((*start, *length))
        } else {
            Err(ParserError::ChunkMissing { signature: fourcc })
        }
    }
}

#[test]
fn test_list_form() {
    let mut f = WaveReader::open("tests/media/izotope_test.wav").unwrap();
    let mut buf: Vec<u8> = vec![];

    f.read_list(ADTL_SIG, &mut buf).unwrap();

    assert_ne!(buf.len(), 0);
}
