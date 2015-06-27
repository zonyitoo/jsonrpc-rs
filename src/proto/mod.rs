// The MIT License (MIT)

// Copyright (c) 2015 Y. T. Chung <zonyitoo@gmail.com>

//  Permission is hereby granted, free of charge, to any person obtaining a
//  copy of this software and associated documentation files (the "Software"),
//  to deal in the Software without restriction, including without limitation
//  the rights to use, copy, modify, merge, publish, distribute, sublicense,
//  and/or sell copies of the Software, and to permit persons to whom the
//  Software is furnished to do so, subject to the following conditions:
//
//  The above copyright notice and this permission notice shall be included in
//  all copies or substantial portions of the Software.
//
//  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
//  OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//  FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//  AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//  LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
//  FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
//  DEALINGS IN THE SOFTWARE.

use std::io;
use std::convert::From;

use rustc_serialize::json::{Object, EncoderError, ParserError, Json, ToJson};

pub use self::spec20::{Client, Server};

pub mod spec20;
pub mod trans;

#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    pub method: String,
    pub params: Option<Json>,
    pub id: Json,
}

impl Request {
    pub fn new(method: String, params: Option<Json>, id: Json) -> Request {
        Request {
            method: method,
            params: params,
            id: id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProtocolError {
    pub code: i64,
    pub message: String,
    pub data: Option<Json>,
}

impl ToJson for ProtocolError {
    fn to_json(&self) -> Json {
        let mut obj = Object::new();
        obj.insert("code".to_owned(), Json::I64(self.code));
        obj.insert("message".to_owned(), Json::String(self.message.clone()));
        if let Some(data) = self.data.clone() {
            obj.insert("data".to_owned(), data);
        }

        Json::Object(obj)
    }
}

impl ProtocolError {
    pub fn new(code: i64, message: String, data: Option<Json>) -> ProtocolError {
        ProtocolError {
            code: code,
            message: message,
            data: data,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Response {
    pub result: Option<Json>,
    pub error: Option<Json>,
    pub id: Json,
}

impl Response {
    pub fn new(result: Option<Json>, error: Option<Json>, id: Json) -> Response {
        Response {
            result: result,
            error: error,
            id: id,
        }
    }
}

#[derive(Debug)]
pub struct InternalError {
    kind: InternalErrorKind,
    desc: &'static str,
    detail: Option<String>,
}

impl InternalError {
    pub fn new(kind: InternalErrorKind, desc: &'static str, detail: Option<String>) -> InternalError {
        InternalError {
            kind: kind,
            desc: desc,
            detail: detail,
        }
    }

    pub fn kind(&self) -> InternalErrorKind {
        self.kind
    }

    pub fn desc(&self) -> &'static str {
        self.desc
    }

    pub fn detail(&self) -> Option<&str> {
        self.detail.as_ref().map(|d| &d[..])
    }
}

#[derive(Debug, Clone, Copy)]
pub enum InternalErrorKind {
    InvalidVersion,
    InvalidResponse,
    MethodNotFound,
    InvalidRequest,
}

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    EncoderError(EncoderError),
    ParserError(ParserError),
    ProtocolError(ProtocolError),
    InternalError(InternalError),
    NotUtf8,
}

impl Error {
    pub fn to_protocol_error(&self) -> ProtocolError {
        match self {
            &Error::IoError(ref err) => {
                ProtocolError::new(-32000, "I/O error".to_owned(),
                    Some(Json::String(<io::Error as ::std::error::Error>::description(&err).to_owned())))
            },
            &Error::EncoderError(ref err) => {
                ProtocolError::new(-32001, "Encoder error".to_owned(),
                    Some(Json::String(<EncoderError as ::std::error::Error>::description(&err).to_owned())))
            },
            &Error::ParserError(ref err) => {
                ProtocolError::new(-32700, "Parse error".to_owned(),
                    Some(Json::String(<ParserError as ::std::error::Error>::description(&err).to_owned())))
            },
            &Error::ProtocolError(ref err) => err.clone(),
            &Error::InternalError(ref err) => {
                match err.kind() {
                    InternalErrorKind::InvalidVersion
                        | InternalErrorKind::InvalidRequest => {
                        ProtocolError::new(-32600, "Invalid Request".to_owned(), None)
                    },
                    InternalErrorKind::InvalidResponse => {
                        ProtocolError::new(-32603, "Internal error".to_owned(), None)
                    },
                    InternalErrorKind::MethodNotFound => {
                        ProtocolError::new(-32601, "Method not found".to_owned(), None)
                    }
                }
            },
            &Error::NotUtf8 => ProtocolError::new(-32600, "Invalid Request".to_owned(), None)
        }
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<EncoderError> for Error {
    fn from(err: EncoderError) -> Error {
        Error::EncoderError(err)
    }
}

impl From<ParserError> for Error {
    fn from(err: ParserError) -> Error {
        Error::ParserError(err)
    }
}

impl From<io::CharsError> for Error {
    fn from(err: io::CharsError) -> Error {
        match err {
            io::CharsError::Other(err) => Error::IoError(err),
            io::CharsError::NotUtf8 => Error::NotUtf8,
        }
    }
}
