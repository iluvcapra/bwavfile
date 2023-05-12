#![allow(dead_code)]
use super::fourcc::{
    FourCC, ReadFourCC, WriteFourCC, ADTL_SIG, DATA_SIG, LABL_SIG, LTXT_SIG, NOTE_SIG,
};

use super::list_form::collect_list_form;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use encoding::all::ASCII;
use encoding::Encoding;
use encoding::{DecoderTrap, EncoderTrap};

use std::io::{Cursor, Error, Read, Write};

#[derive(Copy, Clone, Debug)]
struct RawCue {
    cue_point_id: u32,
    frame: u32,
    chunk_id: FourCC,
    chunk_start: u32,
    block_start: u32,
    frame_offset: u32,
}

impl RawCue {
    fn write_to(cues: Vec<Self>) -> Vec<u8> {
        let mut writer = Cursor::new(vec![0u8; 0]);

        writer.write_u32::<LittleEndian>(cues.len() as u32).unwrap();
        for cue in cues.iter() {
            writer.write_u32::<LittleEndian>(cue.cue_point_id).unwrap();
            writer.write_u32::<LittleEndian>(cue.frame).unwrap();
            writer.write_fourcc(cue.chunk_id).unwrap();
            writer.write_u32::<LittleEndian>(cue.chunk_start).unwrap();
            writer.write_u32::<LittleEndian>(cue.block_start).unwrap();
            writer.write_u32::<LittleEndian>(cue.frame_offset).unwrap();
        }

        writer.into_inner()
    }

    fn read_from(data: &[u8]) -> Result<Vec<Self>, Error> {
        let mut rdr = Cursor::new(data);
        let count = rdr.read_u32::<LittleEndian>()?;
        let mut retval: Vec<Self> = vec![];

        for _ in 0..count {
            retval.push(Self {
                cue_point_id: rdr.read_u32::<LittleEndian>()?,
                frame: rdr.read_u32::<LittleEndian>()?,
                chunk_id: rdr.read_fourcc()?,
                chunk_start: rdr.read_u32::<LittleEndian>()?,
                block_start: rdr.read_u32::<LittleEndian>()?,
                frame_offset: rdr.read_u32::<LittleEndian>()?,
            })
        }

        Ok(retval)
    }
}

#[derive(Clone, Debug)]
struct RawLabel {
    cue_point_id: u32,
    text: Vec<u8>,
}

impl RawLabel {
    fn write_to(&self) -> Vec<u8> {
        let mut writer = Cursor::new(vec![0u8; 0]);
        writer
            .write_u32::<LittleEndian>(self.cue_point_id as u32)
            .unwrap();
        writer.write(&self.text).unwrap();
        writer.into_inner()
    }

    fn read_from(data: &[u8]) -> Result<Self, Error> {
        let mut rdr = Cursor::new(data);
        let length = data.len();

        Ok(Self {
            cue_point_id: rdr.read_u32::<LittleEndian>()?,
            text: {
                let mut buf = vec![0u8; (length - 4) as usize];
                rdr.read_exact(&mut buf)?;
                buf
            },
        })
    }
}

#[derive(Clone, Debug)]
struct RawNote {
    cue_point_id: u32,
    text: Vec<u8>,
}

impl RawNote {
    fn write_to(&self) -> Vec<u8> {
        let mut writer = Cursor::new(vec![0u8; 0]);
        writer.write_u32::<LittleEndian>(self.cue_point_id).unwrap();
        writer.write(&self.text).unwrap();
        writer.into_inner()
    }

    fn read_from(data: &[u8]) -> Result<Self, Error> {
        let mut rdr = Cursor::new(data);
        let length = data.len();

        Ok(Self {
            cue_point_id: rdr.read_u32::<LittleEndian>()?,
            text: {
                let mut buf = vec![0u8; (length - 4) as usize];
                rdr.read_exact(&mut buf)?;
                buf
            },
        })
    }
}

#[derive(Clone, Debug)]
struct RawLtxt {
    cue_point_id: u32,
    frame_length: u32,
    purpose: FourCC,
    country: u16,
    language: u16,
    dialect: u16,
    code_page: u16,
    text: Option<Vec<u8>>,
}

impl RawLtxt {
    fn write_to(&self) -> Vec<u8> {
        let mut writer = Cursor::new(vec![0u8; 0]);
        writer.write_u32::<LittleEndian>(self.cue_point_id).unwrap();
        writer.write_u32::<LittleEndian>(self.frame_length).unwrap();
        writer.write_fourcc(self.purpose).unwrap();
        writer.write_u16::<LittleEndian>(self.country).unwrap();
        writer.write_u16::<LittleEndian>(self.language).unwrap();
        writer.write_u16::<LittleEndian>(self.dialect).unwrap();
        writer.write_u16::<LittleEndian>(self.code_page).unwrap();
        if let Some(ext_text) = &self.text {
            writer.write(ext_text).unwrap();
        }
        writer.into_inner()
    }

    fn read_from(data: &[u8]) -> Result<Self, Error> {
        let mut rdr = Cursor::new(data);
        let length = data.len();

        Ok(Self {
            cue_point_id: rdr.read_u32::<LittleEndian>()?,
            frame_length: rdr.read_u32::<LittleEndian>()?,
            purpose: rdr.read_fourcc()?,
            country: rdr.read_u16::<LittleEndian>()?,
            language: rdr.read_u16::<LittleEndian>()?,
            dialect: rdr.read_u16::<LittleEndian>()?,
            code_page: rdr.read_u16::<LittleEndian>()?,
            text: {
                if length - 20 > 0 {
                    let mut buf = vec![0u8; (length - 20) as usize];
                    rdr.read_exact(&mut buf)?;
                    Some(buf)
                } else {
                    None
                }
            },
        })
    }
}

#[derive(Clone, Debug)]
enum RawAdtlMember {
    Label(RawLabel),
    Note(RawNote),
    LabeledText(RawLtxt),
    Unrecognized(FourCC),
}

impl RawAdtlMember {
    fn compile_adtl(members: &[Self]) -> Vec<u8> {
        let mut w = Cursor::new(vec![0u8; 0]);
        // It seems like all this casing could be done with traits
        for member in members.iter() {
            let (fcc, buf) = match member {
                RawAdtlMember::Label(l) => (LABL_SIG, l.write_to()),
                RawAdtlMember::Note(n) => (NOTE_SIG, n.write_to()),
                RawAdtlMember::LabeledText(t) => (LTXT_SIG, t.write_to()),
                RawAdtlMember::Unrecognized(f) => (*f, vec![0u8; 0]), // <-- this is a dopey case but here for completeness
            };
            w.write_fourcc(fcc).unwrap();
            w.write_u32::<LittleEndian>(buf.len() as u32).unwrap();
            w.write(&buf).unwrap();
            if buf.len() % 2 == 1 {
                w.write_u8(0).unwrap();
            }
        }

        let chunk_content = w.into_inner();
        let mut writer = Cursor::new(vec![0u8; 0]);
        writer.write_fourcc(ADTL_SIG).unwrap();
        writer
            .write_u32::<LittleEndian>(chunk_content.len() as u32)
            .unwrap();
        writer.write(&chunk_content).unwrap();
        writer.into_inner()
    }

    fn collect_from(chunk: &[u8]) -> Result<Vec<RawAdtlMember>, Error> {
        let chunks = collect_list_form(chunk)?;
        let mut retval: Vec<RawAdtlMember> = vec![];

        for chunk in chunks.iter() {
            retval.push(match chunk.signature {
                LABL_SIG => RawAdtlMember::Label(RawLabel::read_from(&chunk.contents)?),
                NOTE_SIG => RawAdtlMember::Note(RawNote::read_from(&chunk.contents)?),
                LTXT_SIG => RawAdtlMember::LabeledText(RawLtxt::read_from(&chunk.contents)?),
                x => RawAdtlMember::Unrecognized(x),
            })
        }
        Ok(retval)
    }
}

trait AdtlMemberSearch {
    fn labels_for_cue_point(&self, id: u32) -> Vec<&RawLabel>;
    fn notes_for_cue_point(&self, id: u32) -> Vec<&RawNote>;
    fn ltxt_for_cue_point(&self, id: u32) -> Vec<&RawLtxt>;
}

impl AdtlMemberSearch for Vec<RawAdtlMember> {
    fn labels_for_cue_point(&self, id: u32) -> Vec<&RawLabel> {
        self.iter()
            .filter_map(|item| match item {
                RawAdtlMember::Label(x) if x.cue_point_id == id => Some(x),
                _ => None,
            })
            .collect()
    }

    fn notes_for_cue_point(&self, id: u32) -> Vec<&RawNote> {
        self.iter()
            .filter_map(|item| match item {
                RawAdtlMember::Note(x) if x.cue_point_id == id => Some(x),
                _ => None,
            })
            .collect()
    }

    fn ltxt_for_cue_point(&self, id: u32) -> Vec<&RawLtxt> {
        self.iter()
            .filter_map(|item| match item {
                RawAdtlMember::LabeledText(x) if x.cue_point_id == id => Some(x),
                _ => None,
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
    /// The time of this marker
    pub frame: u32,

    /// The length of this marker, if it is a range
    pub length: Option<u32>,

    /// The text "label"/name of this marker if provided
    pub label: Option<String>,

    /// The text "note"/comment of this marker if provided
    pub note: Option<String>,

    /// The offser of this marker
    ///
    /// **Note:** Applications use the `frame` and `offset` fields
    /// in different ways. iZotope RX Audio Editor writes the
    /// marker position to *both* fields, while a Sound Devices
    /// recorder writes the marker position to *only* the `offset`
    /// field.
    pub offset: u32,
}

fn convert_to_cue_string(buffer: &[u8]) -> String {
    let trimmed: Vec<u8> = buffer
        .iter()
        .take_while(|c| **c != 0 as u8)
        .cloned()
        .collect();
    ASCII
        .decode(&trimmed, DecoderTrap::Ignore)
        .expect("Error decoding text")
}

fn convert_from_cue_string(val: &str) -> Vec<u8> {
    ASCII
        .encode(&val, EncoderTrap::Ignore)
        .expect("Error encoding text")
}

impl Cue {
    /// Take a list of `Cue`s and convert it into `RawCue` and `RawAdtlMember`s
    fn compile_to(cues: &[Cue]) -> (Vec<RawCue>, Vec<RawAdtlMember>) {
        cues.iter()
            .enumerate()
            .map(|(n, cue)| {
                let raw_cue = RawCue {
                    cue_point_id: n as u32,
                    frame: cue.frame,
                    chunk_id: DATA_SIG,
                    chunk_start: 0,
                    block_start: 0,
                    frame_offset: cue.offset,
                };

                let raw_label = cue.label.as_ref().map(|val| RawLabel {
                    cue_point_id: n as u32,
                    text: convert_from_cue_string(&val),
                });

                let raw_note = cue.note.as_ref().map(|val| RawNote {
                    cue_point_id: n as u32,
                    text: convert_from_cue_string(&val),
                });

                let raw_ltxt = cue.length.map(|val| RawLtxt {
                    cue_point_id: n as u32,
                    frame_length: val,
                    purpose: FourCC::make(b"rgn "),
                    country: 0,
                    language: 0,
                    dialect: 0,
                    code_page: 0,
                    text: None,
                });

                (raw_cue, raw_label, raw_note, raw_ltxt)
            })
            .fold(
                (Vec::<RawCue>::new(), Vec::<RawAdtlMember>::new()),
                |(mut cues, mut adtls), (cue, label, note, ltxt)| {
                    cues.push(cue);
                    label.map(|l| adtls.push(RawAdtlMember::Label(l)));
                    note.map(|n| adtls.push(RawAdtlMember::Note(n)));
                    ltxt.map(|m| adtls.push(RawAdtlMember::LabeledText(m)));
                    (cues, adtls)
                },
            )
    }

    pub fn collect_from(cue_chunk: &[u8], adtl_chunk: Option<&[u8]>) -> Result<Vec<Cue>, Error> {
        let raw_cues = RawCue::read_from(cue_chunk)?;
        let raw_adtl: Vec<RawAdtlMember>;

        if let Some(adtl) = adtl_chunk {
            raw_adtl = RawAdtlMember::collect_from(adtl)?;
        } else {
            raw_adtl = vec![];
        }

        Ok(raw_cues
            .iter()
            .map(|i| {
                Cue {
                    //ident : i.cue_point_id,
                    frame: i.frame,
                    length: {
                        raw_adtl
                            .ltxt_for_cue_point(i.cue_point_id)
                            .first()
                            .filter(|x| x.purpose == FourCC::make(b"rgn "))
                            .map(|x| x.frame_length)
                    },
                    label: {
                        raw_adtl
                            .labels_for_cue_point(i.cue_point_id)
                            .iter()
                            .map(|s| convert_to_cue_string(&s.text))
                            .next()
                    },
                    note: {
                        raw_adtl
                            .notes_for_cue_point(i.cue_point_id)
                            .iter()
                            //.filter_map(|x| str::from_utf8(&x.text).ok())
                            .map(|s| convert_to_cue_string(&s.text))
                            .next()
                    },
                    offset: i.frame_offset,
                }
            })
            .collect())
    }
}
