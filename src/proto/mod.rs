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

use std::io::{self, Read, Write};
use std::convert::From;

use rustc_serialize::json::{EncoderError, ParserError, Json};

pub mod spec20;

#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    method: String,
    params: Json,
    id: Json,
}

impl Request {
    pub fn new(method: String, params: Json, id: Json) -> Request {
        Request {
            method: method,
            params: params,
            id: id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProtocolError {
    code: i64,
    message: String,
    data: Option<Json>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Response {
    result: Option<Json>,
    error: Option<Json>,
    id: Json,
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

#[derive(Debug, Clone, PartialEq)]
pub enum ClientRequest {
    Single(Request),
    Batch(Vec<Request>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServerResponse {
    Single(Response),
    Batch(Vec<Response>),
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
}

#[derive(Debug)]
pub enum InternalErrorKind {
    InvalidVersion,
    InvalidResponse,
}

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    EncoderError(EncoderError),
    ParserError(ParserError),
    ProtocolError(ProtocolError),
    InternalError(InternalError),
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

pub trait ClientSender<W: Write> {
    fn request(&mut self, request: &Request) -> Result<()>;
    fn batch_request(&mut self, requests: &[Request]) -> Result<()>;
}

pub trait ClientReceiver<R: Read> {
    fn get_response(&mut self) -> Result<ServerResponse>;
}

pub trait ServerProtocol {
    fn response<W: Write>(writer: &mut W, response: &Response) -> Result<()>;
    fn batch_response<W: Write>(writer: &mut W, responses: &[Response]) -> Result<()>;

    fn request<R: Read>(reader: &mut R) -> Result<ClientRequest>;
}
