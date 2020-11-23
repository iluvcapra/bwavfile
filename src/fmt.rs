
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


const PCM_SUBTYPE_UUID: [u8; 16] = [0x00, 0x00, 0x00, 0x01,0x00, 0x00, 0x00, 0x10, 0x80, 0x00, 0x00, 0xaa,0x00, 0x38, 0x9b, 0x71];

const FLOAT_SUBTYPE_UUID: [u8; 16] = [0x00, 0x00, 0x00, 0x03,0x00, 0x00, 0x00, 0x10, 0x80, 0x00, 0x00, 0xaa,0x00, 0x38, 0x9b, 0x71];

/*

http://dream.cs.bath.ac.uk/researchdev/wave-ex/bformat.html

Integer format: 
SUBTYPE_AMBISONIC_B_FORMAT_PCM 
 {00000001-0721-11d3-8644-C8C1CA000000}

Floating-point format:

SUBTYPE_AMBISONIC_B_FORMAT_IEEE_FLOAT 
{00000003-0721-11d3-8644-C8C1CA000000}

In the case of ambisonics, I'm guessing we'd ignore the channel map and implied
channels W, X, Y, Z
*/
                          
/// ADM Audio ID record
/// 
/// This structure relates a channel in the wave file to either a common ADM
/// channel definition or further definition in the WAV file's ADM metadata 
/// chunk.
/// 
/// An individial channel in a WAV file can have multiple Audio IDs in an ADM 
/// AudioProgramme.
/// 
/// See BS.2088-1 § 8, also BS.2094, also blahblahblah...
pub struct ADMAudioID {
    track_uid: [char; 12],
    channel_format_ref: [char; 14],
    pack_ref: [char; 11]
}

/// Describes a single channel in a WAV file.
pub struct ChannelDescriptor {
    /// Index, the offset of this channel's samples in one frame.
    index: u16,

    /// Channel assignment
    /// 
    /// This is either implied (in the case of mono or stereo wave files) or
    /// explicitly given in `WaveFormatExtentended` for files with more tracks.
    speaker: WaveFmtExtendedChannelMask,

    /// ADM audioTrackUIDs
    adm_track_audio_ids: Vec<ADMAudioID>,
}



/*
https://docs.microsoft.com/en-us/windows-hardware/drivers/audio/subformat-guids-for-compressed-audio-formats

These are from http://dream.cs.bath.ac.uk/researchdev/wave-ex/mulchaud.rtf
*/

#[derive(Debug)]
pub enum WaveFmtExtendedChannelMask {
    DirectOut        = 0x0,
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
    TopBackRight     = 0x20000,
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

