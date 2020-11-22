use std::io::{Read, Write};

use super::errors::Error as ParserError;

use encoding::{DecoderTrap, EncoderTrap};
use encoding::{Encoding};
use encoding::all::ASCII;

use byteorder::LittleEndian;
use byteorder::{ReadBytesExt, WriteBytesExt};

/**
 * References:
 * - http://www-mmsp.ece.mcgill.ca/Documents/AudioFormats/WAVE/Docs/multichaudP.pdf
*/
#[derive(PartialEq)]
enum FormatTags {
    Integer = 0x0001,
    Float = 0x0003,
    Extensible = 0xFFFE
}

const PCM_SUBTYPE_UUID: [u8; 16] = [0x00, 0x00, 0x00, 0x01,
                                    0x00, 0x00, 0x00, 0x10,
                                    0x80, 0x00, 0x00, 0xaa,
                                    0x00, 0x38, 0x9b, 0x71];

const FLOAT_SUBTYPE_UUID: [u8; 16] = [0x00, 0x00, 0x00, 0x03,
                                      0x00, 0x00, 0x00, 0x10,
                                      0x80, 0x00, 0x00, 0xaa,
                                      0x00, 0x38, 0x9b, 0x71];

/*
https://docs.microsoft.com/en-us/windows-hardware/drivers/audio/subformat-guids-for-compressed-audio-formats
http://dream.cs.bath.ac.uk/researchdev/wave-ex/bformat.html

These are from http://dream.cs.bath.ac.uk/researchdev/wave-ex/mulchaud.rtf
*/

#[derive(Debug)]
pub enum WaveFmtExtendedChannelMask {
    FrontLeft        = 0x1,
    FrontRight       = 0x2,
    FrontCenter      = 0x4,
    LowFrequency     = 0x8,
    BackLeft         = 0x10,
    BackRight        = 0x20,
    FrontCenterLeft  = 0x40,
    FrontCenterRight = 0x80,
    BackCenter       = 0x100,
    SideLeft         = 0x200,
    SideRight        = 0x400,
    TopCenter        = 0x800,
    TopFrontLeft     = 0x1000,
    TopFrontCenter   = 0x2000,
    TopFrontRight    = 0x4000,
    TopBackLeft      = 0x8000,
    TopBackCenter    = 0x10000,
    TopBackRight     = 0x20000 
}


/**
 * Extended Wave Format
 * 
 * https://docs.microsoft.com/en-us/windows/win32/api/mmreg/ns-mmreg-waveformatextensible
 */
#[derive(Debug)]
pub struct WaveFmtExtended {

    /// Valid bits per sample
    pub valid_bits_per_sample : u16,

    /// Channel mask
    /// 
    /// Identifies the speaker assignment for each channel in the file
    pub channel_mask : WaveFmtExtendedChannelMask,

    /// Codec GUID
    /// 
    /// Identifies the codec of the audio stream
    pub type_guid : [u8; 16],
}

/**
 * WAV file data format record.
 * 
 * The `fmt` record contains essential information describing the binary
 * structure of the data segment of the WAVE file, such as sample 
 * rate, sample binary format, channel count, etc.
 *
 */
#[derive(Debug)]
pub struct WaveFmt {

    /// A tag identifying the codec in use.
    /// 
    /// If this is 0xFFFE, the codec will be identified by a GUID
    /// in `extended_format`
    pub tag: u16,

    /// Count of audio channels in each frame
    pub channel_count: u16,

    /// Sample rate of the audio data
    pub sample_rate: u32,

    /// Count of bytes per second
    /// 
    /// By rule, this is `block_alignment * sample_rate`
    pub bytes_per_second: u32,

    /// Count of bytes per audio frame
    /// 
    /// By rule, this is `channel_count * bits_per_sample / 8`
    pub block_alignment: u16,

    /// Count of bits stored in the file per sample
    pub bits_per_sample: u16,

    /// Extended format description
    /// 
    /// Additional format metadata if `channel_count` is greater than 2,
    /// or if certain codecs are used.
    pub extended_format: Option<WaveFmtExtended>
}


impl WaveFmt {

    /// Create a new integer PCM format `WaveFmt` 
    pub fn new_pcm(sample_rate: u32, bits_per_sample: u16, channel_count: u16) -> Self {
        let container_bits_per_sample = bits_per_sample + (bits_per_sample % 8);
        let container_bytes_per_sample= container_bits_per_sample / 8;

        let tag :u16 = match channel_count {
            0 => panic!("Error"),
            1..=2 => FormatTags::Integer as u16,
            _ => FormatTags::Extensible as u16,
        };

        WaveFmt {
            tag, 
            channel_count,
            sample_rate,
            bytes_per_second: container_bytes_per_sample as u32 * sample_rate * channel_count as u32,
            block_alignment: container_bytes_per_sample * channel_count,
            bits_per_sample: container_bits_per_sample,
            extended_format: None
        }
    }
}

/**
 * Broadcast-WAV metadata record.
 * 
 * The `bext` record contains information about the original recording of the 
 * Wave file, including a longish (256 ASCII chars) description field, 
 * originator identification fields, creation calendar date and time, a 
 * sample-accurate recording time field, and a SMPTE UMID. 
 * 
 * For a Wave file to be a complaint "Broadcast-WAV" file, it must contain
 * a `bext` metadata record.
 * 
 * For reference on the structure and use of the BEXT record
 * check out [EBU Tech 3285](https://tech.ebu.ch/docs/tech/tech3285.pdf).
 */
#[derive(Debug)]
pub struct Bext {
    pub description: String,
    pub originator: String,
    pub originator_reference: String,
    pub origination_date: String,
    pub origination_time: String,
    pub time_reference: u64,
    pub version: u16,
    pub umid: Option<[u8; 64]>,
    pub loudness_value: Option<f32>,
    pub loudness_range: Option<f32>,
    pub max_true_peak_level: Option<f32>,
    pub max_momentary_loudness: Option<f32>,
    pub max_short_term_loudness: Option<f32>,
    // 180 bytes of nothing
    pub coding_history: String
}

pub trait ReadBWaveChunks: Read {
    fn read_bext(&mut self) -> Result<Bext, ParserError>;
    fn read_bext_string_field(&mut self, length: usize) -> Result<String,ParserError>;
    fn read_wave_fmt(&mut self) -> Result<WaveFmt, ParserError>;
}

pub trait WriteBWaveChunks: Write {
    fn write_wave_fmt(&mut self, format : &WaveFmt) -> Result<(), ParserError>;
    fn write_bext_string_field(&mut self, string: &String, length: usize) -> Result<(),ParserError>;
    fn write_bext(&mut self, bext: &Bext) -> Result<(),ParserError>;
}

impl<T> WriteBWaveChunks for T where T: Write {
    fn write_wave_fmt(&mut self, format : &WaveFmt) -> Result<(), ParserError> {
        self.write_u16::<LittleEndian>(format.tag)?;
        self.write_u16::<LittleEndian>(format.channel_count)?;
        self.write_u32::<LittleEndian>(format.sample_rate)?;
        self.write_u32::<LittleEndian>(format.bytes_per_second)?;
        self.write_u16::<LittleEndian>(format.block_alignment)?;
        self.write_u16::<LittleEndian>(format.bits_per_sample)?;
        // self.write_u8(0)?;
        Ok(())
    }

    fn write_bext_string_field(&mut self, string: &String, length: usize) -> Result<(),ParserError> {
        let mut buf = ASCII.encode(&string, EncoderTrap::Ignore).expect("Error encoding text");
        buf.truncate(length);
        let filler_length = length - buf.len();
        if filler_length > 0{
            let mut filler = vec![0u8; filler_length ];
            buf.append(&mut filler);
        }

        self.write_all(&buf)?;
        Ok(())
    }

    fn write_bext(&mut self, bext: &Bext) -> Result<(),ParserError> {
        self.write_bext_string_field(&bext.description, 256)?;
        self.write_bext_string_field(&bext.originator, 32)?;
        self.write_bext_string_field(&bext.originator_reference, 32)?;
        self.write_bext_string_field(&bext.origination_date, 10)?;
        self.write_bext_string_field(&bext.origination_time, 8)?;
        self.write_u64::<LittleEndian>(bext.time_reference)?;
        self.write_u16::<LittleEndian>(bext.version)?;

        let buf = bext.umid.unwrap_or([0u8; 64]);
        self.write_all(&buf)?;

        self.write_i16::<LittleEndian>( 
            (bext.loudness_value.unwrap_or(0.0) * 100.0) as i16 )?;
        self.write_i16::<LittleEndian>( 
            (bext.loudness_range.unwrap_or(0.0) * 100.0) as i16 )?;
        self.write_i16::<LittleEndian>( 
            (bext.max_true_peak_level.unwrap_or(0.0) * 100.0) as i16 )?;
        self.write_i16::<LittleEndian>( 
            (bext.max_momentary_loudness.unwrap_or(0.0) * 100.0) as i16 )?;
        self.write_i16::<LittleEndian>( 
            (bext.max_short_term_loudness.unwrap_or(0.0) * 100.0) as i16 )?;
        
        let padding = [0u8; 180];
        self.write_all(&padding)?;
        
        let coding = ASCII.encode(&bext.coding_history, EncoderTrap::Ignore)
            .expect("Error");

        self.write_all(&coding)?;
        Ok(())
    }
}

impl<T> ReadBWaveChunks for T where T: Read {

    fn read_wave_fmt(&mut self) -> Result<WaveFmt, ParserError> {
        Ok(WaveFmt {
            tag:                self.read_u16::<LittleEndian>()?,
            channel_count:      self.read_u16::<LittleEndian>()?,
            sample_rate:        self.read_u32::<LittleEndian>()?,
            bytes_per_second:   self.read_u32::<LittleEndian>()?,
            block_alignment:    self.read_u16::<LittleEndian>()?,
            bits_per_sample:    self.read_u16::<LittleEndian>()?, 
            extended_format: None
        })
    }

    fn read_bext_string_field(&mut self, length: usize) -> Result<String,ParserError> {
        let mut buffer : Vec<u8> = vec![0; length];
        self.read(&mut buffer)?;
        let trimmed : Vec<u8> = buffer.iter().take_while(|c| **c != 0 as u8).cloned().collect();
        Ok(ASCII.decode(&trimmed, DecoderTrap::Ignore).expect("Error decoding text")) 
    }

    fn read_bext(&mut self) -> Result<Bext, ParserError> {
        let version : u16; 
        Ok( Bext { 
                description:            self.read_bext_string_field(256)?,
                originator:             self.read_bext_string_field(32)?,
                originator_reference :  self.read_bext_string_field(32)?,
                origination_date :      self.read_bext_string_field(10)?, 
                origination_time :      self.read_bext_string_field(8)?, 
                time_reference:         self.read_u64::<LittleEndian>()?,
                version: {
                    version = self.read_u16::<LittleEndian>()?;
                    version
                },
                umid: {
                    let mut buf = [0u8 ; 64];
                    self.read(&mut buf)?;
                    if version > 0 { Some(buf) } else { None }
                },
                loudness_value: {
                    let val = (self.read_i16::<LittleEndian>()? as f32) / 100f32;
                    if version > 1 { Some(val) } else { None }
                },
                loudness_range: {
                    let val = self.read_i16::<LittleEndian>()? as f32 / 100f32;
                    if version > 1 { Some(val) } else { None }
                },
                max_true_peak_level: {
                    let val = self.read_i16::<LittleEndian>()? as f32 / 100f32;
                    if version > 1 { Some(val) } else { None }
                },
                max_momentary_loudness: {
                    let val = self.read_i16::<LittleEndian>()? as f32 / 100f32;
                    if version > 1 { Some(val) } else { None }
                },
                max_short_term_loudness: {
                    let val = self.read_i16::<LittleEndian>()? as f32 / 100f32;
                    if version > 1 { Some(val) } else { None }
                }, 
                coding_history: {
                    for _ in 0..=180 { self.read_u8()?; }
                    let mut buf = vec![];
                    self.read_to_end(&mut buf)?;
                    ASCII.decode(&buf, DecoderTrap::Ignore).expect("Error decoding text")
                }
        })
     }
}