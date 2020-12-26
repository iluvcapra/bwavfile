use uuid::Uuid;

const BASIC_PCM: u16        = 0x0001;
const BASIC_FLOAT: u16      = 0x0003;
const BASIC_MPEG: u16       = 0x0050;
const BASIC_EXTENDED: u16   = 0xFFFE;

/* RC 2361 §4:

 WAVE Format IDs are converted to GUIDs by inserting the hexadecimal
   value of the WAVE Format ID into the XXXXXXXX part of the following
   template: {XXXXXXXX-0000-0010-8000-00AA00389B71}. For example, a WAVE
   Format ID of 123 has the GUID value of {00000123-0000-0010-8000-
   00AA00389B71}.

*/

pub const UUID_PCM: Uuid =           Uuid::from_bytes([0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 
    0x80, 0x00, 0x00, 0xaa, 0x00, 0x38, 0x9b, 0x71]);

pub const UUID_FLOAT: Uuid =         Uuid::from_bytes([0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00,
    0x80, 0x00, 0x00, 0xaa, 0x00, 0x38, 0x9b, 0x71]);

pub const UUID_MPEG: Uuid =          Uuid::from_bytes([0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00,
    0x80, 0x00, 0x00, 0xaa, 0x00, 0x38, 0x9b, 0x71]);

pub const UUID_BFORMAT_PCM: Uuid =   Uuid::from_bytes([0x01, 0x00, 0x00, 0x00, 0x21, 0x07, 0xd3, 0x11, 
    0x86, 0x44, 0xc8, 0xc1, 0xca, 0x00, 0x00, 0x00]);

pub const UUID_BFORMAT_FLOAT: Uuid = Uuid::from_bytes([0x03, 0x00, 0x00, 0x00, 0x21, 0x07, 0xd3, 0x11,
    0x86, 0x44, 0xc8, 0xc1, 0xca, 0x00, 0x00, 0x00]);


fn uuid_from_basic_tag(tag: u16) -> Uuid {
    let tail : [u8; 6] = [0x00,0xaa,0x00,0x38,0x9b,0x71];
    Uuid::from_fields_le(tag as u32, 0x0000, 0x0010, &tail).unwrap()
}

/// Sample format of the Wave file.
/// 
/// 
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CommonFormat {
    /// Integer linear PCM
    IntegerPCM,
 
    /// IEEE Floating-point Linear PCM
    IeeeFloatPCM,
 
    /// MPEG
    Mpeg,
 
    /// Ambisonic B-Format Linear PCM
    AmbisonicBFormatIntegerPCM,
 
    /// Ambisonic B-Format Float PCM
    AmbisonicBFormatIeeeFloatPCM,
 
    /// An unknown format identified by a basic format tag.
    UnknownBasic(u16),
 
    /// An unknown format identified by an extension UUID.
    UnknownExtended(Uuid),
}

impl CommonFormat {
    /// Resolve a tag and Uuid to a `CommonFormat`.
    pub fn make(basic: u16, uuid: Option<Uuid>) -> Self {
        match (basic, uuid) {
            (BASIC_PCM, _) => Self::IntegerPCM,
            (BASIC_FLOAT, _) => Self::IeeeFloatPCM,
            (BASIC_MPEG, _) => Self::Mpeg,
            (BASIC_EXTENDED, Some(UUID_PCM))  => Self::IntegerPCM,
            (BASIC_EXTENDED, Some(UUID_FLOAT))=> Self::IeeeFloatPCM,
            (BASIC_EXTENDED, Some(UUID_BFORMAT_PCM)) => Self::AmbisonicBFormatIntegerPCM,
            (BASIC_EXTENDED, Some(UUID_BFORMAT_FLOAT)) => Self::AmbisonicBFormatIeeeFloatPCM,
            (BASIC_EXTENDED, Some(x)) => CommonFormat::UnknownExtended(x),
            (x, _) => CommonFormat::UnknownBasic(x)
        }
    }

    /// Get the appropriate tag and `Uuid` for the callee.
    /// 
    /// If there is no appropriate tag for the format of the callee, the 
    /// returned tag will be 0xFFFE and the `Uuid` will describe the format.
    pub fn take(self) -> (u16, Uuid) {
        match self {
            Self::IntegerPCM => (BASIC_PCM, UUID_PCM),
            Self::IeeeFloatPCM => (BASIC_FLOAT, UUID_FLOAT),
            Self::Mpeg => (BASIC_MPEG, UUID_MPEG),
            Self::AmbisonicBFormatIntegerPCM => (BASIC_EXTENDED, UUID_BFORMAT_PCM),
            Self::AmbisonicBFormatIeeeFloatPCM => (BASIC_EXTENDED, UUID_BFORMAT_FLOAT),
            Self::UnknownBasic(x) => ( x, uuid_from_basic_tag(x) ),
            Self::UnknownExtended(x) => ( BASIC_EXTENDED, x)
        }
    }
}
