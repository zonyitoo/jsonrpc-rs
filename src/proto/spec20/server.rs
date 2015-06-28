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

use std::io::{Read, Write};
use std::convert::From;

use rustc_serialize::Encodable;
use rustc_serialize::json::{self, Object, Array, Json, Encoder};

use proto::{self, Request, Response};
use proto::{InternalErrorKind, InternalError};
use proto::trans::{self, GetRequest, SendResponse};

use proto::spec20::check_version;

pub struct Server<'a, S: Read + Write + 'a> {
    stream: &'a mut S,
}

impl<'a, S: Read + Write + 'a> Server<'a, S> {
    pub fn new(s: &'a mut S) -> Server<'a, S> {
        Server {
            stream: s,
        }
    }
}

impl<'a, S: Read + Write + 'a> SendResponse for Server<'a, S> {
    fn response(&mut self, response: Response) -> proto::Result<()> {
        let obj = response_to_json(response);

        {
            let mut encoder = Encoder::new(&mut self.stream);
            try!(obj.encode(&mut encoder));
        }

        self.stream.write_all(b"\r\n")
            .and(self.stream.flush())
            .map_err(From::from)
    }

    fn batch_response(&mut self, responses: Vec<Response>) -> proto::Result<()> {
        let arr: Array = responses.into_iter().map(response_to_json).collect();

        {
            let mut encoder = Encoder::new(&mut self.stream);
            try!(arr.encode(&mut encoder));
        }

        self.stream.write_all(b"\r\n")
            .and(self.stream.flush())
            .map_err(From::from)
    }
}

impl<'a, S: Read + Write + 'a> GetRequest for Server<'a, S> {
    fn get_request(&mut self) -> proto::Result<trans::Request> {
        let request = try!(Json::from_reader(&mut self.stream));
        request_from_json(request)
    }
}

fn response_to_json(resp: Response) -> Json {
    let mut obj = json::Object::new();
    obj.insert("jsonrpc".to_owned(), Json::String("2.0".to_owned()));

    if let Some(result) = resp.result {
        obj.insert("result".to_owned(), result);
    }

    if let Some(error) = resp.error {
        obj.insert("error".to_owned(), error);
    }

    obj.insert("id".to_owned(), resp.id);

    Json::Object(obj)
}

fn json_to_request(mut obj: json::Object) -> proto::Result<Request> {
    try!(check_version(&obj));

    let method = match obj.remove("method") {
        Some(Json::String(m)) => m,
        Some(obj) => {
            let ierr = InternalError::new(InternalErrorKind::InvalidRequest,
                                          "`method` must be a String",
                                          Some(format!("Expecting method, but found {:?}", obj)));
            return Err(proto::Error::InternalError(ierr));
        },
        None => {
            let ierr = InternalError::new(InternalErrorKind::InvalidRequest,
                                          "`method` is required",
                                          None);
            return Err(proto::Error::InternalError(ierr));
        }
    };

    let params = obj.remove("params");

    let id = match obj.remove("id") {
        Some(id) => id,
        None => {
            let ierr = InternalError::new(InternalErrorKind::InvalidRequest,
                                          "`id` is required",
                                          None);
            return Err(proto::Error::InternalError(ierr));
        }
    };

    Ok(Request::new(method, params, id))
}

fn request_from_json(req: Json) -> proto::Result<trans::Request> {
    match req {
        Json::Object(obj) => {
            json_to_request(obj).map(trans::Request::Single)
        },
        Json::Array(arr) => {
            let mut batch = Vec::with_capacity(arr.len());
            for obj in arr.into_iter() {
                match obj {
                    Json::Object(obj) =>
                        batch.push(try!(json_to_request(obj))),
                    _ => {
                        let ierr = InternalError::new(InternalErrorKind::InvalidResponse,
                                                      "Invalid JSON-RPC response",
                                                      Some(format!("Expecting a Response object, but found {:?}",
                                                                   obj)));
                        return Err(proto::Error::InternalError(ierr));
                    }
                }
            }

            Ok(trans::Request::Batch(batch))
        },
        _ => {
            let ierr = InternalError::new(InternalErrorKind::InvalidResponse,
                                          "Invalid JSON-RPC response",
                                          Some(format!("Expecting JSON-RPC response, but found {:?}", req)));
            Err(proto::Error::InternalError(ierr))
        }
    }
}
