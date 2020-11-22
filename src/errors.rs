use std::io;
use super::fourcc::FourCC;

/// Errors returned by methods in this crate.
#[derive(Debug)]
pub enum Error {
    /// An `io::Error` occurred
    IOError(io::Error),
    /// The file does not begin with a recognized WAVE header
    HeaderNotRecognized,
    /// A wave file with a 64-bit header does not contain
    /// the required `ds64` metadata element
    MissingRequiredDS64,
    /// A data chunk required to complete the operation
    /// is not present in the file
    ChunkMissing { signature : FourCC },
    /// The file is formatted improperly
    FmtChunkAfterData,
    /// The file did not validate as a minimal WAV file
    NotMinimalWaveFile,
    /// The `data` chunk is not aligned to the desired page
    /// boundary
    DataChunkNotAligned,
    /// The file cannot be converted into an RF64 file due
    /// to its internal structure
    InsufficientDS64Reservation {expected: u64, actual: u64},
    /// The file is not optimized for writing new data
    DataChunkNotPreparedForAppend
}


impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::IOError(error)
    }
}