use std::fmt::Debug;
use std::io;

/// A Four-character Code
/// 
/// For idetifying chunks, structured contiguous slices or segments
/// within a WAV file.
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct FourCC([u8; 4]);

impl FourCC {
    pub const fn make(s: &[u8; 4]) -> Self {
        Self(*s)
    }
}

impl From<[char; 4]> for FourCC {
    fn from(chars : [char; 4]) -> Self {
        Self([chars[0] as u8 , chars[1] as u8, chars[2] as u8, chars[3] as u8])
    }
}

impl From<[u8; 4]> for FourCC {
    fn from(bytes: [u8; 4]) -> Self {
        FourCC(bytes)
    }
}

impl From<FourCC> for [u8; 4] {
    fn from(fourcc: FourCC) -> Self {
        fourcc.0
    }
}


impl From<&FourCC> for [char;4] {
    fn from( f: &FourCC) -> Self {
        [f.0[0] as char, f.0[1] as char, f.0[2] as char, f.0[3] as char,]
    }
}

impl From<FourCC> for [char;4] {
    fn from( f: FourCC) -> Self {
        [f.0[0] as char, f.0[1] as char, f.0[2] as char, f.0[3] as char,]
    }
}


impl From<&FourCC> for String {
    fn from(f: &FourCC) -> Self { 
        let chars: [char;4] = f.into();
        chars.iter().collect::<String>() 
    }
}

impl From<FourCC> for String {
    fn from(f: FourCC) -> Self { 
        let chars: [char;4] = f.into();
        chars.iter().collect::<String>() 
    }
}

impl Debug for FourCC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let s : String = self.into();
        write!(f, "FourCC({})", s)
    }
}

pub trait ReadFourCC: io::Read {
    fn read_fourcc(&mut self) -> Result<FourCC, io::Error>;
}

pub trait WriteFourCC: io::Write {
    fn write_fourcc(&mut self, fourcc :FourCC) -> Result<(), io::Error>;
}

impl<T> ReadFourCC for T where T: io::Read { 
    fn read_fourcc(&mut self) -> Result<FourCC, io::Error> {
        let mut buf : [u8; 4] = [0 ; 4];
        self.read_exact(&mut buf)?;
        Ok( FourCC::from(buf) )
    }
}

impl<T> WriteFourCC for T where T: io::Write {
    fn write_fourcc(&mut self, fourcc :FourCC) -> Result<(), io::Error> {
        let buf : [u8; 4] = fourcc.into();
        self.write_all(&buf)?;
        Ok(())
    }
}


pub const RIFF_SIG: FourCC = FourCC::make(b"RIFF");
pub const WAVE_SIG: FourCC = FourCC::make(b"WAVE");
pub const RF64_SIG: FourCC = FourCC::make(b"RF64"); 
pub const DS64_SIG: FourCC = FourCC::make(b"ds64"); 
pub const BW64_SIG: FourCC = FourCC::make(b"BW64");

pub const DATA_SIG: FourCC = FourCC::make(b"data");
pub const FMT__SIG: FourCC = FourCC::make(b"fmt ");

pub const BEXT_SIG: FourCC = FourCC::make(b"bext");
pub const FACT_SIG: FourCC = FourCC::make(b"fact");

pub const JUNK_SIG: FourCC = FourCC::make(b"JUNK");
pub const FLLR_SIG: FourCC = FourCC::make(b"FLLR");
pub const ELM1_SIG: FourCC = FourCC::make(b"elm1");
pub const LIST_SIG: FourCC = FourCC::make(b"LIST");

pub const CUE__SIG: FourCC = FourCC::make(b"cue ");
pub const ADTL_SIG: FourCC = FourCC::make(b"adtl");
pub const LABL_SIG: FourCC = FourCC::make(b"labl");
pub const NOTE_SIG: FourCC = FourCC::make(b"note");
pub const LTXT_SIG: FourCC = FourCC::make(b"ltxt");


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_string() {
        let a = FourCC::make(b"a1b2");
        let s : String = a.into();
        assert_eq!(s, "a1b2");
    }
}