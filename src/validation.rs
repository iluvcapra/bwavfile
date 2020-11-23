use super::parser::{Parser};
use super::fourcc::{FourCC, FMT__SIG,DATA_SIG, BEXT_SIG, JUNK_SIG, FLLR_SIG};
use super::errors::Error as ParserError;
use super::wavereader::{WaveReader};

use std::io::{Read,Seek};


impl<R:Read + Seek> WaveReader<R> {
    /**
    * Validate file is readable.
    * 
    *  `Ok(())` if the source meets the minimum standard of 
    *  readability by a permissive client:
    *  - `fmt` chunk and `data` chunk are present
    *  - `fmt` chunk appears before `data` chunk
    */
    pub fn validate_readable(&mut self) -> Result<(), ParserError> {
        let (fmt_pos, _)  = self.get_chunk_extent_at_index(FMT__SIG, 0)?;
        let (data_pos, _) = self.get_chunk_extent_at_index(DATA_SIG, 0)?;

        if fmt_pos < data_pos {
            Ok(())
        } else {
            Err( ParserError::FmtChunkAfterData)
        }
    }

    /** 
     * Validate minimal WAVE file.
     * 
     * `Ok(())` if the source is `validate_readable()` AND
     * 
     *   - Contains _only_ a `fmt` chunk and `data` chunk, with no other chunks present
     *   - is not an RF64/BW64
     * 
     * Some clients require a WAVE file to only contain format and data without any other
     * metadata and this function is provided to validate this condition.
     * 
     * ### Examples
     * 
     * ```
     * # use bwavfile::WaveReader;
     * 
     * let mut w = WaveReader::open("tests/media/ff_minimal.wav").unwrap();
     * w.validate_minimal().expect("Minimal wav did not validate not minimal!");
     * ```
     * 
     * ```
     * # use bwavfile::WaveReader;
     * 
     * let mut x = WaveReader::open("tests/media/pt_24bit_51.wav").unwrap();
     * x.validate_minimal().expect_err("Complex WAV validated minimal!");
     * ```
    */
    pub fn validate_minimal(&mut self) -> Result<(), ParserError>  {
        self.validate_readable()?;

        let chunk_fourccs : Vec<FourCC> = Parser::make(&mut self.inner)?
            .into_chunk_list()?.iter().map(|c| c.signature ).collect();

        if chunk_fourccs == vec![FMT__SIG, DATA_SIG] {
            Ok(())
        } else {
            Err( ParserError::NotMinimalWaveFile )
        }
    }

    /**
     * Validate Broadcast-WAVE file format
     * 
     * Returns `Ok(())` if `validate_readable()` and file contains a 
     * Broadcast-WAV metadata record (a `bext` chunk).
     * 
     * ### Examples
     * 
     * ```
     * # use bwavfile::WaveReader;
     * 
     * let mut w = WaveReader::open("tests/media/ff_bwav_stereo.wav").unwrap();
     * w.validate_broadcast_wave().expect("BWAVE file did not validate BWAVE");
     * 
     * let mut x = WaveReader::open("tests/media/pt_24bit.wav").unwrap();
     * x.validate_broadcast_wave().expect("BWAVE file did not validate BWAVE");
     * 
     * let mut y = WaveReader::open("tests/media/audacity_16bit.wav").unwrap();
     * y.validate_broadcast_wave().expect_err("Plain WAV file DID validate BWAVE");
     * ```
    */
    pub fn validate_broadcast_wave(&mut self) -> Result<(), ParserError> {
        self.validate_readable()?;
        let (_, _) = self.get_chunk_extent_at_index(BEXT_SIG, 0)?;
        Ok(())
    } 

    /**
     * Verify data is aligned to a block boundary.
     * 
     * Returns `Ok(())` if `validate_readable()` and the start of the 
     * `data` chunk's content begins at 0x4000.
    */
    pub fn validate_data_chunk_alignment(&mut self) -> Result<() , ParserError> {
        self.validate_readable()?;
        let (start, _) = self.get_chunk_extent_at_index(DATA_SIG, 0)?;
        if start == 0x4000 {
            Ok(())
        } else {
            Err(ParserError::DataChunkNotAligned)
        }
    }

    /**
     * Verify audio data can be appended immediately to this file.
     * 
     * Returns `Ok(())` if:
     *  - `validate_readable()`
     *  - there is a `JUNK` or `FLLR` immediately at the beginning of the chunk 
     *    list adequately large enough to be overwritten by a `ds64` (92 bytes)
     *  - `data` is the final chunk
    */
    pub fn validate_prepared_for_append(&mut self) -> Result<(), ParserError> {
        self.validate_readable()?;

        let chunks = Parser::make(&mut self.inner)?.into_chunk_list()?;
        let ds64_space_required = 92;

        let eligible_filler_chunks = chunks.iter()
            .take_while(|c| c.signature == JUNK_SIG || c.signature == FLLR_SIG);

        let filler = eligible_filler_chunks
            .enumerate()
            .fold(0, |accum, (n, item)| if n == 0 { accum + item.length } else {accum + item.length + 8});

        if filler < ds64_space_required {
            Err(ParserError::InsufficientDS64Reservation {expected: ds64_space_required, actual: filler})
        } else {
            let data_pos = chunks.iter().position(|c| c.signature == DATA_SIG);
        
            match data_pos {
                Some(p) if p == chunks.len() - 1 => Ok(()),
                _ => Err(ParserError::DataChunkNotPreparedForAppend)
            }
        }
    }
}








