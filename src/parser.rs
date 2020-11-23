
use std::io;
use std::io::SeekFrom::{Current, Start};
use std::io::{Seek, Read};
use std::collections::HashMap;

use byteorder::LittleEndian;
use byteorder::ReadBytesExt;

use super::errors::Error;
use super::fourcc::{FourCC, ReadFourCC};
use super::fourcc::{RIFF_SIG, RF64_SIG, BW64_SIG, WAVE_SIG, DS64_SIG, DATA_SIG};

// just for your reference...
// RF64 documentation https://www.itu.int/dms_pubrec/itu-r/rec/bs/R-REC-BS.2088-1-201910-I!!PDF-E.pdf

// EBU long files being with RF64, and the ITU recommends using BW64, so we recorgnize both.

const RF64_SIZE_MARKER: u32 = 0xFF_FF_FF_FF;

#[derive(Debug)]
pub enum Event {
    StartParse,
    ReadHeader { signature: FourCC, length_field: u32 },
    ReadRF64Header { signature: FourCC },
    ReadDS64 {file_size: u64, long_sizes: HashMap<FourCC,u64> },
    BeginChunk { signature: FourCC, content_start: u64, content_length: u64 },
    Failed { error: Error },
    FinishParse
}

#[derive(Debug)]
enum State {
    New,
    ReadyForHeader,
    ReadyForDS64,
    ReadyForChunk { at: u64, remaining: u64 },
    Error,
    Complete
}

pub struct Parser<R: Read + Seek> {
    stream: R,
    state: State,
    ds64state: HashMap<FourCC,u64>
}

pub struct ChunkIteratorItem {
    pub signature: FourCC,
    pub start: u64,
    pub length: u64
}

impl<R: Read + Seek> Parser<R> {
    
    // wraps a stream
    pub fn make(stream: R) -> Result<Self, Error> {
        let newmap: HashMap<FourCC, u64> = HashMap::new();
        let mut the_stream = stream;
        the_stream.seek(Start(0))?;
        return Ok(Parser {
            stream: the_stream, 
            state: State::New,
            ds64state: newmap,
        })
    }

    // pub fn into_inner(self) -> R {
    //     self.stream
    // }

    pub fn into_chunk_iterator(self) -> impl Iterator<Item = Result<ChunkIteratorItem, Error>>{
        self.filter_map({|event|
            if let Event::BeginChunk {signature , content_start, content_length } = event {
                Some(Ok(ChunkIteratorItem {signature, start: content_start, length: content_length }))
            } else if let Event::Failed { error }  = event {
                Some(Err(error))
            } else {
                None
            }
        })
    }

    pub fn into_chunk_list(self) -> Result<Vec<ChunkIteratorItem>,Error> {
        let mut error = Ok(());

        let chunks = self.into_chunk_iterator()
            .scan(&mut error, |err, res| match res {
                Ok(ok) => Some(ok),
                Err(e) => { **err = Err(e); None }
            })
            .collect();
            
        error?;

        Ok( chunks )
    }

}

impl<R: Read + Seek> Iterator for Parser<R> {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        let (event, next_state) = self.advance();
        self.state = next_state;
        return event;
    }
}

impl<R: Read + Seek> Parser<R> {

    fn parse_header(&mut self) -> Result<(Event,State),io::Error> {
        let file_sig = self.stream.read_fourcc()?;
        let length = self.stream.read_u32::<LittleEndian>()?;
        let list_sig = self.stream.read_fourcc()?;

        let event : Event;
        let next_state: State;

        match (file_sig, length, list_sig) {
            (RIFF_SIG, size, WAVE_SIG) => {
                event = Event::ReadHeader {
                    signature: file_sig,
                    length_field: size
                };

                next_state = State::ReadyForChunk {
                    at: 12,
                    remaining: (length - 4) as u64,
                };
            },
            (RF64_SIG, RF64_SIZE_MARKER, WAVE_SIG) | (BW64_SIG, RF64_SIZE_MARKER, WAVE_SIG) => {
                event = Event::ReadRF64Header {
                    signature: file_sig
                };

                next_state = State::ReadyForDS64;
            },
            _ => {
                event = Event::Failed {
                    error: Error::HeaderNotRecognized
                };
                next_state = State::Error;
            }
        }

        return Ok( (event, next_state) );
    }

    fn parse_ds64(&mut self) -> Result<(Event, State), Error> {
        let at :u64 = 12;

        let ds64_sig = self.stream.read_fourcc()?;
        let ds64_size = self.stream.read_u32::<LittleEndian>()? as u64;
        let mut read :u64 = 0;

        if ds64_sig != DS64_SIG {
            return Err(Error::MissingRequiredDS64);

        } else {
            let long_file_size = self.stream.read_u64::<LittleEndian>()?;
            let long_data_size = self.stream.read_u64::<LittleEndian>()?;
            let _long_frame_count = self.stream.read_u64::<LittleEndian>(); // dead frame count field
            read += 24;

            let field_count = self.stream.read_u32::<LittleEndian>()?;
            read += 4;

            for _ in 0..field_count {
                let this_fourcc = self.stream.read_fourcc()?;
                let this_field_size = self.stream.read_u64::<LittleEndian>()?;
                self.ds64state.insert(this_fourcc, this_field_size);
                read += 12;
            }

            self.ds64state.insert(DATA_SIG, long_data_size);
            
            if read < ds64_size {
                /*  for some reason the ds64 chunk returned by Pro Tools is longer than
                    it should be but it's all zeroes so... skip. 

                    For the record libsndfile seems to do the same thing...
                    https://github.com/libsndfile/libsndfile/blob/08d802a3d18fa19c74f38ed910d9e33f80248187/src/rf64.c#L230
                */
                let _ = self.stream.seek(Current((ds64_size - read) as i64));
            }

            let event = Event::ReadDS64 {
                file_size: long_file_size,
                long_sizes : self.ds64state.clone(),
            };

            let state = State::ReadyForChunk {
                at: at + 8 + ds64_size,
                remaining: long_file_size - (4 + 8 + ds64_size),
            };

            return Ok( (event, state) );
        }
    }

    fn enter_chunk(&mut self, at :u64, remaining: u64) -> Result<(Event, State), io::Error> {

        let event;
        let state;

        if remaining == 0 {
            event = Event::FinishParse;
            state = State::Complete;

        } else {
            let this_fourcc = self.stream.read_fourcc()?;
            let this_size: u64;

            if self.ds64state.contains_key(&this_fourcc) {
                this_size = self.ds64state[&this_fourcc];
                let _skip = self.stream.read_u32::<LittleEndian>()? as u64;
            } else {
                this_size = self.stream.read_u32::<LittleEndian>()? as u64;
            }

            let this_displacement :u64 = if this_size % 2 == 1 { this_size + 1 } else { this_size }; 
            self.stream.seek(Current(this_displacement as i64))?;

            event = Event::BeginChunk {
                signature: this_fourcc,
                content_start: at + 8,
                content_length: this_size
            };
            
            state = State::ReadyForChunk {
                at: at + 8 + this_displacement,
                remaining: remaining - 8 - this_displacement
            }
        }

        return Ok( (event, state) );
    }

    fn handle_state(&mut self) -> Result<(Option<Event>, State), Error> {
        match self.state {
            State::New => {
                return Ok( ( Some(Event::StartParse) , State::ReadyForHeader) );
            },
            State::ReadyForHeader => {
                let (event, state) = self.parse_header()?;
                return Ok( ( Some(event), state ) );
            },
            State::ReadyForDS64 => {
                let (event, state) = self.parse_ds64()?;
                return Ok( ( Some(event), state ) );
            },
            State::ReadyForChunk { at, remaining } => {
                let (event, state) = self.enter_chunk(at, remaining)?;
                return Ok( ( Some(event), state ) );
            },
            State::Error => {
                return Ok( ( Some(Event::FinishParse) , State::Complete ) );
            },
            State::Complete => {
                return Ok( ( None, State::Complete ) );
            }
        }
    }

    fn advance(&mut self) -> (Option<Event>, State) {
        match self.handle_state() {
            Ok(( event , state) ) => {
                return (event, state);
            },
            Err(error) => {
                return (Some(Event::Failed { error: error.into() } ), State::Error );
            }
        }
    }
}

