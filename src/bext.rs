
pub type LU = f32;
pub type LUFS = f32;
pub type Decibels = f32;


///  Broadcast-WAV metadata record.
///
///  The `bext` record contains information about the original recording of the 
///  Wave file, including a longish (256 ASCII chars) description field, 
///  originator identification fields, creation calendar date and time, a 
///  sample-accurate recording time field, and a SMPTE UMID. 
///
///  For a Wave file to be a complaint "Broadcast-WAV" file, it must contain
///  a `bext` metadata record.
///
/// ## Resources
/// - [EBU Tech 3285](https://tech.ebu.ch/docs/tech/tech3285.pdf).
/// - [EBU Tech R098](https://tech.ebu.ch/docs/r/r098.pdf) (1999) "Format for the &lt;CodingHistory&gt; field in Broadcast Wave Format files, BWF"
/// - [EBU Tech R099](https://tech.ebu.ch/docs/r/r099.pdf) (October 2011) "‘Unique’ Source Identifier (USID) for use in the 
///   &lt;OriginatorReference&gt; field of the Broadcast Wave Format"

// Note for me later:
// get env values: https://doc.rust-lang.org/std/macro.option_env.html
// Cargo env values: https://doc.rust-lang.org/cargo/reference/environment-variables.html

#[derive(Debug)]
pub struct Bext {

    /// 256 ASCII character field with free text.
    pub description: String,

    /// Originating application.
    pub originator: String,

    /// Application-specific UID.
    pub originator_reference: String,

    /// Creation date in format `YYYY-MM-DD`.
    pub origination_date: String,

    /// Creation time in format `HH:MM:SS`.
    pub origination_time: String,

    /// Time of the start of this wave file, expressed as the number of samples
    /// since local midnight.
    pub time_reference: u64,

    /// Bext chunk version.
    /// 
    /// Version 1 contains a UMID, version 2 contains a UMID and 
    /// loudness metadata.
    pub version: u16,

    /// SMPTE 330M UMID
    /// 
    /// This field is `None` if the version is less than 1.
    pub umid: Option<[u8; 64]>,

    /// Integrated loudness in LUFS.
    /// 
    /// This field is `None` if the version is less than 2.
    pub loudness_value: Option<LUFS>,

    /// Loudness range in LU.
    /// 
    /// This field is `None` if the version is less than 2.
    pub loudness_range: Option<LU>,

    /// Maximum True Peak Level in decibels True Peak.
    ///
    /// This field is `None` if the version is less than 2.
    pub max_true_peak_level: Option<Decibels>,

    /// Maximum momentary loudness in LUFS.
    ///
    /// This field is `None` if the version is less than 2.
    pub max_momentary_loudness: Option<LUFS>,

    /// Maximum short-term loudness in LUFS.
    /// 
    /// This field is `None` if the version is less than 2.
    pub max_short_term_loudness: Option<LUFS>,
    // 180 bytes of nothing

    /// Coding History.
    pub coding_history: String
}
