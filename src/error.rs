
use std::io;
use std::convert::From;

use proto::ProtocolError;

pub enum Error {
    ProtocolError(ProtocolError),
    IoError(io::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IoError(e)
    }
}

impl From<ProtocolError> for Error {
    fn from(e: ProtocolError) -> Error {
        Error::ProtocolError(e)
    }
}
