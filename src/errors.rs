use std::io;
use super::fourcc::FourCC;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    HeaderNotRecognized,
    MissingRequiredDS64,
    ChunkMissing { signature : FourCC },
    FmtChunkAfterData,
    NotMinimalWaveFile,
    DataChunkNotAligned,
    InsufficientDS64Reservation {expected: u64, actual: u64},
    DataChunkNotPreparedForAppend
}


impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::IOError(error)
    }
}