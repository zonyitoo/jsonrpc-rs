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
use proto::trans::{ServerResponse, SendRequest, GetResponse};

use proto::spec20::check_version;

pub struct Client<'a, S: Read + Write + 'a> {
    stream: &'a mut S,
}

impl<'a, S: Read + Write + 'a> Client<'a, S> {
    pub fn new(stream: &'a mut S) -> Client<'a, S> {
        Client {
            stream: stream,
        }
    }
}

impl<'a, S: Read + Write + 'a> SendRequest for Client<'a, S> {
    fn request(&mut self, request: Request) -> proto::Result<()> {
        let obj = request_to_json(request);

        {
            let mut encoder = Encoder::new(&mut self.stream);
            try!(obj.encode(&mut encoder));
        }

        self.stream.write_all(b"\r\n")
            .and(self.stream.flush())
            .map_err(From::from)
    }

    fn batch_request(&mut self, requests: Vec<Request>) -> proto::Result<()> {
        let arr: Array = requests.into_iter().map(request_to_json).collect();

        {
            let mut encoder = Encoder::new(&mut self.stream);
            try!(arr.encode(&mut encoder));
        }

        self.stream.write_all(b"\r\n")
            .and(self.stream.flush())
            .map_err(From::from)
    }
}

impl<'a, S: Read + Write + 'a> GetResponse for Client<'a, S> {
    fn get_response(&mut self) -> proto::Result<ServerResponse> {
        let response = try!(Json::from_reader(&mut self.stream));

        response_from_json(response)
    }
}

fn request_to_json(request: Request) -> Json {
    let mut obj = Object::new();
    obj.insert("jsonrpc".to_owned(), Json::String("2.0".to_owned()));
    obj.insert("method".to_owned(), Json::String(request.method));
    if let Some(params) = request.params {
        obj.insert("params".to_owned(), params);
    }
    obj.insert("id".to_owned(), request.id);

    Json::Object(obj)
}

fn response_from_json(resp: Json) -> proto::Result<ServerResponse> {
    match resp {
        Json::Object(obj) => {
            json_to_response(obj).map(ServerResponse::Single)
        },
        Json::Array(arr) => {
            let mut batch = Vec::with_capacity(arr.len());
            for obj in arr.into_iter() {
                match obj {
                    Json::Object(obj) =>
                        batch.push(try!(json_to_response(obj))),
                    _ => {
                        let ierr = InternalError::new(InternalErrorKind::InvalidResponse,
                                                      "Invalid JSON-RPC response",
                                                      Some(format!("Expecting a Response object, but found {:?}",
                                                                   obj)));
                        return Err(proto::Error::InternalError(ierr));
                    }
                }
            }

            Ok(ServerResponse::Batch(batch))
        },
        _ => {
            let ierr = InternalError::new(InternalErrorKind::InvalidResponse,
                                          "Invalid JSON-RPC response",
                                          Some(format!("Expecting JSON-RPC response, but found {:?}", resp)));
            Err(proto::Error::InternalError(ierr))
        }
    }
}

fn json_to_response(mut obj: json::Object) -> proto::Result<Response> {
    try!(check_version(&obj));

    let result = obj.remove("result");
    let error = obj.remove("error");

    let id = match obj.remove("id") {
        Some(id) => id,
        None => {
            let ierr = InternalError::new(InternalErrorKind::InvalidResponse,
                                          "Invalid JSON-RPC response",
                                          Some("`id` is required".to_owned()));
            return Err(proto::Error::InternalError(ierr));
        }
    };

    Ok(Response::new(result, error, id))
}
