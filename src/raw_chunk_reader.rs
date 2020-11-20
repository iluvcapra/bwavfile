use std::cmp::min;

use std::io::SeekFrom;
use std::io::SeekFrom::{Start, Current, End};
use std::io::{Seek,Read,Error,ErrorKind};

// I'm not sure this hasn't already been written somewhere in
// std but I'm just doing this here as an exercise.
#[derive(Debug)]
pub struct RawChunkReader<'a, R: Read + Seek> {
    reader: &'a mut R,
    start: u64,
    length: u64,
    position: u64
}

impl<'a,R: Read + Seek> RawChunkReader<'a, R> {
    pub fn new(reader: &'a mut R, start: u64, length: u64) -> Self {
        return Self {
            reader: reader, 
            start: start, 
            length: length, 
            position: 0
        }
    }

    pub fn length(&self) -> u64 {
        self.length
    } 
}

impl<'a, R:Read + Seek> Read for RawChunkReader<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> { 
        if self.position >= self.length {
            Err(Error::new(ErrorKind::UnexpectedEof, "RawChunkReader encountered end-of-file"))
        } else {
            self.reader.seek(Start(self.start + self.position))?;
            let to_read = min(self.length - self.position, buf.len() as u64);
            self.reader.take(to_read).read(buf)?;
            self.position += to_read;
            Ok(to_read as usize)
        }
    }
}

impl<'a, R:Read + Seek> Seek for RawChunkReader<'_, R> {
    fn seek(&mut self, seek: SeekFrom) -> Result<u64, std::io::Error> { 
        match seek {
            Start(s) => {
                self.position = s;
                Ok(self.position)
            },
            Current(s) => {
                let new_position = s + self.position as i64;
                if new_position < 0 {
                    Err( Error::new(ErrorKind::Other, "Attempted seek before beginning of chunk") )
                } else {
                    self.position = new_position as u64;
                    Ok(self.position)
                }
            },
            End(s) => {
                let new_position = s + self.length as i64;
                if new_position < 0 {
                    Err( Error::new(ErrorKind::Other, "Attempted seek before beginning of chunk") )
                } else {
                    self.position = new_position as u64;
                    Ok(self.position)
                }
            }
        }
    }
}
