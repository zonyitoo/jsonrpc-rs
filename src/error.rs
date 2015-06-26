
use std::io;
use std::convert::From;

#[derive(RustcEncodable, RustcDecodable)]
pub struct RpcError {
    code: i64,
    message: String,
    data: Option<String>,
}

impl RpcError {
    pub fn new(code: i64, message: String, data: Option<String>) -> RpcError {
        RpcError {
            code: code,
            message: message,
            data: data,
        }
    }
}

pub enum Error {
    RpcError(RpcError),
    IoError(io::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IoError(e)
    }
}

impl From<RpcError> for Error {
    fn from(e: RpcError) -> Error {
        Error::RpcError(e)
    }
}
