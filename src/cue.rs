use super::fourcc::{FourCC,ReadFourCC, LABL_SIG, NOTE_SIG, LTXT_SIG};
use super::list_form::collect_list_form;

use byteorder::{ReadBytesExt, LittleEndian};

use encoding::{DecoderTrap};
use encoding::{Encoding};
use encoding::all::ASCII;

use std::io::{Cursor, Error, Read};

#[derive(Copy,Clone, Debug)]
struct RawCue {
    cue_point_id : u32,
    frame : u32,
    chunk_id : FourCC,
    chunk_start : u32,
    block_start : u32,
    frame_offset : u32
}

impl RawCue {

    fn write_to(cues : Vec<Self>) -> Vec<u8> {
        let mut writer = Cursor::new(vec![0u8; 0]);
        
        
        todo!()

    }

    fn read_from(data : &[u8]) -> Result<Vec<Self>,Error> {
        let mut rdr = Cursor::new(data);
        let count = rdr.read_u32::<LittleEndian>()?;
        let mut retval : Vec<Self> = vec![];

        for _ in 0..count {
            retval.push( Self {
                cue_point_id : rdr.read_u32::<LittleEndian>()?,
                frame : rdr.read_u32::<LittleEndian>()?,
                chunk_id : rdr.read_fourcc()?,
                chunk_start : rdr.read_u32::<LittleEndian>()?,
                block_start : rdr.read_u32::<LittleEndian>()?,
                frame_offset : rdr.read_u32::<LittleEndian>()?
            })
        }

        Ok( retval )
    }
}

#[derive(Clone, Debug)]
struct RawLabel {
    cue_point_id : u32,
    text : Vec<u8>
}

impl RawLabel {
    fn read_from(data : &[u8]) -> Result<Self, Error> {
        let mut rdr = Cursor::new(data);
        let length = data.len();

        Ok( Self {
            cue_point_id : rdr.read_u32::<LittleEndian>()?,
            text : {
                let mut buf = vec![0u8; (length - 4) as usize ];
                rdr.read_exact(&mut buf)?;
                buf
            }
        })
    }
}

#[derive(Clone, Debug)]
struct RawNote {
    cue_point_id : u32,
    text : Vec<u8>
}

impl RawNote {
    fn read_from(data : &[u8]) -> Result<Self, Error> {
        let mut rdr = Cursor::new(data);
        let length = data.len();

        Ok( Self {
            cue_point_id : rdr.read_u32::<LittleEndian>()?,
            text : {
                let mut buf = vec![0u8; (length - 4) as usize ];
                rdr.read_exact(&mut buf)?;
                buf
            }
        })
    }
}

#[derive(Clone, Debug)]
struct RawLtxt { 
    cue_point_id : u32,
    frame_length : u32,
    purpose : FourCC,
    country : u16,
    language : u16,
    dialect : u16,
    code_page : u16,
    text: Option<Vec<u8>>
}

impl RawLtxt {
    fn read_from(data : &[u8]) -> Result<Self, Error> {
        let mut rdr = Cursor::new(data);
        let length = data.len();

        Ok( Self {
            cue_point_id : rdr.read_u32::<LittleEndian>()?,
            frame_length : rdr.read_u32::<LittleEndian>()?,
            purpose : rdr.read_fourcc()?,
            country : rdr.read_u16::<LittleEndian>()?,
            language : rdr.read_u16::<LittleEndian>()?,
            dialect : rdr.read_u16::<LittleEndian>()?,
            code_page : rdr.read_u16::<LittleEndian>()?,
            text : {
                if length - 20 > 0 {
                    let mut buf = vec![0u8; (length - 20) as usize];
                    rdr.read_exact(&mut buf)?;
                    Some( buf )
                } else {
                    None
                }
            }
        })
    }
}

#[derive(Clone, Debug)]
enum RawAdtlMember {
    Label(RawLabel),
    Note(RawNote),
    LabeledText(RawLtxt),
    Unrecognized(FourCC)
}

impl RawAdtlMember {
    fn collect_from(chunk : &[u8]) -> Result<Vec<RawAdtlMember>,Error> {
        let chunks = collect_list_form(chunk)?;
        let mut retval : Vec<RawAdtlMember> = vec![];

        for chunk in chunks.iter() {
            retval.push( 
                match chunk.signature {
                    LABL_SIG => RawAdtlMember::Label( RawLabel::read_from(&chunk.contents)? ),
                    NOTE_SIG => RawAdtlMember::Note( RawNote::read_from(&chunk.contents)? ),
                    LTXT_SIG => RawAdtlMember::LabeledText( RawLtxt::read_from(&chunk.contents)? ),
                    x => RawAdtlMember::Unrecognized(x)
                }
            )
        }
        Ok( retval )
    }
}

trait AdtlMemberSearch {
    fn labels_for_cue_point(&self, id: u32) -> Vec<&RawLabel>;
    fn notes_for_cue_point(&self, id : u32) -> Vec<&RawNote>;
    fn ltxt_for_cue_point(&self, id: u32) -> Vec<&RawLtxt>;
}

impl AdtlMemberSearch for Vec<RawAdtlMember> {

    fn labels_for_cue_point(&self, id: u32) -> Vec<&RawLabel> {
        self.iter().filter_map(|item| {
            match item {
                RawAdtlMember::Label(x) if x.cue_point_id == id => Some(x),
                _ => None
            }
        })
        .collect()
    }
    
    fn notes_for_cue_point(&self, id: u32) -> Vec<&RawNote> { 
        self.iter().filter_map(|item| {
            match item {
                RawAdtlMember::Note(x) if x.cue_point_id == id => Some(x),
                _ => None
            }
        })
        .collect()  
    }

    fn ltxt_for_cue_point(&self, id: u32) -> Vec<&RawLtxt> {
        self.iter().filter_map(|item| {
            match item {
                RawAdtlMember::LabeledText(x) if x.cue_point_id == id => Some(x),
                _ => None
            }
        })
        .collect()
    }
}

/// A cue point recorded in the `cue` and `adtl` metadata.
/// 
/// ## Resources
/// - [Cue list, label and other metadata](https://sites.google.com/site/musicgapi/technical-documents/wav-file-format#smpl)
/// 
/// ### Not Implemented
/// - [EBU 3285 Supplement 2](https://tech.ebu.ch/docs/tech/tech3285s2.pdf) (July 2001): Quality chunk and cuesheet
pub struct Cue {

    /// Unique numeric identifier for this cue
    //pub ident : u32,

    /// The time of this marker
    pub frame : u32,

    /// The length of this marker, if it is a range
    pub length : Option<u32>,

    /// The text "label"/name of this marker if provided
    pub label : Option<String>,

    /// The text "note"/comment of this marker if provided
    pub note : Option<String>
}


fn convert_to_cue_string(buffer : &[u8]) -> String {
    let trimmed : Vec<u8> = buffer.iter().take_while(|c| **c != 0 as u8).cloned().collect();
    ASCII.decode(&trimmed, DecoderTrap::Ignore).expect("Error decoding text")
}

impl Cue {

    pub fn collect_from(cue_chunk : &[u8], adtl_chunk : Option<&[u8]>) -> Result<Vec<Cue>, Error> {
        let raw_cues = RawCue::read_from(cue_chunk)?;
        let raw_adtl : Vec<RawAdtlMember>;

        if let Some(adtl) = adtl_chunk {
            raw_adtl = RawAdtlMember::collect_from(adtl)?;
        } else {
            raw_adtl = vec![];
        }
        

        Ok( 
            raw_cues.iter()
            .map(|i| {
                Cue {
                    //ident : i.cue_point_id,
                    frame : i.frame,
                    length: {
                        raw_adtl.ltxt_for_cue_point(i.cue_point_id).first()
                        .filter(|x| x.purpose == FourCC::make(b"rgn "))
                        .map(|x| x.frame_length)
                    },
                    label: {
                        raw_adtl.labels_for_cue_point(i.cue_point_id).iter()
                            .map(|s| convert_to_cue_string(&s.text))
                            .next()
                    },
                    note : {
                        raw_adtl.notes_for_cue_point(i.cue_point_id).iter()
                            //.filter_map(|x| str::from_utf8(&x.text).ok())
                            .map(|s| convert_to_cue_string(&s.text))
                            .next()
                    }
                }
            }).collect() 
        )
    }

}