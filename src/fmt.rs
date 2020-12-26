use uuid::Uuid;
use super::common_format::{CommonFormat, UUID_PCM};

#[allow(dead_code)]

/// ADM Audio ID record.
/// 
/// This structure relates a channel in the wave file to either a common ADM
/// channel definition or further definition in the WAV file's ADM metadata 
/// chunk.
/// 
/// An individual channel in a WAV file can have multiple Audio IDs in an ADM 
/// `AudioProgramme`.
/// 
/// See BS.2088-1 § 8, also BS.2094, also blahblahblah...
pub struct ADMAudioID {
    pub track_uid: [char; 12],
    pub channel_format_ref: [char; 14],
    pub pack_ref: [char; 11]
}

/// Describes a single channel in a WAV file.
pub struct ChannelDescriptor {
    /// Index, the offset of this channel's samples in one frame.
    pub index: u16,

    /// Channel assignment
    /// 
    /// This is either implied (in the case of mono or stereo wave files) or
    /// explicitly given in `WaveFormatExtentended` for files with more tracks.
    pub speaker: ChannelMask,

    /// ADM audioTrackUIDs
    pub adm_track_audio_ids: Vec<ADMAudioID>,
}


/// A bitmask indicating which channels are present in 
/// the file.
/// 
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelMask {
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

impl From<u32> for ChannelMask {
    
    fn from(value: u32) -> Self { 
        match value {
            0x1 => Self::FrontLeft,
            0x2 => Self::FrontRight,
            0x4 => Self::FrontCenter, 
            0x8 => Self::LowFrequency,
            0x10 => Self::BackLeft,
            0x20 => Self::BackRight,
            0x40 => Self::FrontCenterLeft,
            0x80 => Self::FrontCenterRight,
            0x100 => Self::BackCenter,
            0x200 => Self::SideLeft,
            0x400 => Self::SideRight,
            0x800 => Self::TopCenter,
            0x1000 => Self::TopFrontLeft, 
            0x2000 => Self::TopFrontCenter,
            0x4000 => Self::TopFrontRight,
            0x8000 => Self::TopBackLeft,
            0x10000 => Self::TopBackCenter,
            0x20000 => Self::TopBackRight,
            _ => Self::DirectOut 
        }
    }
}

impl ChannelMask {
    pub fn channels(input_mask : u32, channel_count: u16) -> Vec<ChannelMask> {
        let reserved_mask = 0xfff2_0000_u32;
        if (input_mask & reserved_mask) > 0 {
            vec![ ChannelMask::DirectOut ; channel_count as usize ]
        } else {
            (0..18).map(|i| 1 << i )
                .filter(|mask| mask & input_mask > 0)
                .map(|mask| Into::<ChannelMask>::into(mask))
                .collect()
        }
    }
}

/**
 * Extended Wave Format
 * 
 * https://docs.microsoft.com/en-us/windows/win32/api/mmreg/ns-mmreg-waveformatextensible
 */
#[derive(Debug, Copy, Clone)]
pub struct WaveFmtExtended {

    /// Valid bits per sample
    pub valid_bits_per_sample : u16,

    /// Channel mask
    /// 
    /// Identifies the speaker assignment for each channel in the file
    pub channel_mask : u32,

    /// Codec GUID
    /// 
    /// Identifies the codec of the audio stream
    pub type_guid : Uuid,
}

/**
 * WAV file data format record.
 * 
 * The `fmt` record contains essential information describing the binary
 * structure of the data segment of the WAVE file, such as sample 
 * rate, sample binary format, channel count, etc.
 *
 */
#[derive(Debug, Copy, Clone)]
pub struct WaveFmt {

    /// A tag identifying the codec in use.
    /// 
    /// If this is 0xFFFE, the codec will be identified by a GUID
    /// in `extended_format`
    pub tag: u16,

    /// Count of audio channels in each frame
    pub channel_count: u16,

    /// Playback rate of the audio data
    ///
    /// In frames per second.
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
    ///
    /// By rule, `bits_per_sample % 8 == 0` for Broadcast-Wave files.
    ///
    /// Modern clients will encode
    /// unusual sample sizes in normal byte sizes but will set the valid_bits
    /// flag in extended format record.
    ///
    /// Generally speaking this will be true for all modern wave files, though
    /// there was an historical "packed" stereo format of 20 bits per sample,
    /// 5 bytes per frame, 5 bytes block alignment.
    pub bits_per_sample: u16,

    /// Extended format description
    /// 
    /// Additional format metadata if `channel_count` is greater than 2,
    /// or if certain codecs are used.
    pub extended_format: Option<WaveFmtExtended>
}


impl WaveFmt {
    
    /// Create a new integer PCM format for a monoaural audio stream.
    pub fn new_pcm_mono(sample_rate: u32, bits_per_sample: u16) -> Self {
        Self::new_pcm_multichannel(sample_rate, bits_per_sample, 0x4)
    }

    /// Create a new integer PCM format for a standard Left-Right stereo audio 
    /// stream.
    pub fn new_pcm_stereo(sample_rate: u32, bits_per_sample: u16) -> Self {
        Self::new_pcm_multichannel(sample_rate, bits_per_sample, 0x3)
    }

    /// Create a new integer PCM format for ambisonic b-format.
    pub fn new_pcm_ambisonic(sample_rate: u32, bits_per_sample: u16, channel_count: u16) -> Self {
        todo!()
    }

    /// Create a new integer PCM format `WaveFmt` with a custom channel bitmap.
    /// 
    /// The order of `channels` is not important. When reading or writing 
    /// audio frames you must use the standard multichannel order for Wave 
    /// files, the numerical order of the cases of `ChannelMask`.
    pub fn new_pcm_multichannel(sample_rate: u32, bits_per_sample: u16, channel_bitmap: u32) -> Self {
        let container_bits_per_sample = bits_per_sample + (bits_per_sample % 8);
        let container_bytes_per_sample= container_bits_per_sample / 8;
        
        let channel_count: u16 = (0..=31).fold(0u16, |accum, n| accum + (0x1 & (channel_bitmap >> n) as u16) );

        let result : (u16, Option<WaveFmtExtended>) = match channel_bitmap {
            ch if bits_per_sample != container_bits_per_sample => (
                (0xFFFE, Some(WaveFmtExtended { valid_bits_per_sample: bits_per_sample, channel_mask: ch, 
                    type_guid: UUID_PCM }) )
            ),
            0b0100 => (0x0001, None),
            0b0011 => (0x0001, None),
            ch => (
                (0xFFFE, Some( WaveFmtExtended { valid_bits_per_sample: bits_per_sample, channel_mask: ch, 
                    type_guid: UUID_PCM}))
            )
        };

        let (tag, extformat) = result;

        WaveFmt {
            tag,
            channel_count,
            sample_rate,
            bytes_per_second: container_bytes_per_sample as u32 * sample_rate * channel_count as u32,
            block_alignment: container_bytes_per_sample * channel_count,
            bits_per_sample: container_bits_per_sample,
            extended_format: extformat
        }
    }

    /// Format or codec of the file's audio data.
    /// 
    /// The `CommonFormat` unifies the format tag and the format extension GUID. Use this
    /// method to determine the codec.
    pub fn common_format(&self) -> CommonFormat {
        CommonFormat::make( self.tag, self.extended_format.map(|ext| ext.type_guid))
    }

    /// Create a frame buffer sized to hold frames for a reader or writer
    /// 
    /// This is a conveneince method that creates a `Vec<i32>` with
    /// as many elements as there are channels in the underlying stream. 
    pub fn create_frame_buffer(&self) -> Vec<i32> {
        vec![0i32; self.channel_count as usize]
    }


    /// Channel descriptors for each channel.
    pub fn channels(&self) -> Vec<ChannelDescriptor> {
        match self.channel_count {
            1 => vec![
                ChannelDescriptor {
                    index: 0,
                    speaker: ChannelMask::FrontCenter,
                    adm_track_audio_ids: vec![]
                }
            ],
            2 => vec![
                ChannelDescriptor {
                    index: 0,
                    speaker: ChannelMask::FrontLeft,
                    adm_track_audio_ids: vec![]
                },
                ChannelDescriptor {
                    index: 1,
                    speaker: ChannelMask::FrontRight,
                    adm_track_audio_ids: vec![]
                }
            ],
            x if x > 2 => {
                let channel_mask = self.extended_format.map(|x| x.channel_mask).unwrap_or(0);
                let channels = ChannelMask::channels(channel_mask, self.channel_count);
                let channels_expanded = channels.iter().chain(std::iter::repeat(&ChannelMask::DirectOut));

                (0..self.channel_count)
                    .zip(channels_expanded)
                    .map(|(n,chan)| ChannelDescriptor {
                        index: n,
                        speaker: *chan, 
                        adm_track_audio_ids: vec![]
                    }).collect()
            },
            x => panic!("Channel count ({}) was illegal!", x),
        }
    }
}

