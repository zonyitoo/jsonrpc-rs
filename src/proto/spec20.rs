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

use std::io::{self, Read, Write, CharsError};
use std::convert::From;

use rustc_serialize::Encodable;
use rustc_serialize::json::{self, Object, Array, Json, Encoder, Builder};

use proto::{self, ClientSender, ClientReceiver, ServerProtocol, Request, Response, ClientRequest, ServerResponse};
use proto::{InternalErrorKind, InternalError};

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

impl<'a, S: Read + Write + 'a> ClientSender<S> for Client<'a, S> {
    fn request(&mut self, request: &Request) -> proto::Result<()> {
        let obj = request_to_json(&request);

        let encoded = try!(json::encode(&obj));
        self.stream.write_all(encoded.as_bytes())
            .map_err(|err| From::from(err))
    }

    fn batch_request(&mut self, requests: &[Request]) -> proto::Result<()> {
        let arr: Array = requests.iter().map(request_to_json).collect();

        let encoded = try!(json::encode(&arr));
        self.stream.write_all(encoded.as_bytes())
            .map_err(|err| From::from(err))
    }


}

impl<'a, S: Read + Write + 'a> ClientReceiver<S> for Client<'a, S> {
    fn get_response(&mut self) -> proto::Result<ServerResponse> {
        let mut builder = Builder::new(self.stream.chars()
                                             .take_while(|res| res.is_ok())
                                             .map(|res| res.unwrap()));
        let response = try!(builder.build());

        response_from_json(response)
    }
}

fn request_to_json(request: &Request) -> Json {
    let mut obj = Object::new();
    obj.insert("jsonrpc".to_owned(), Json::String("2.0".to_owned()));
    obj.insert("method".to_owned(), Json::String(request.method.clone()));
    obj.insert("params".to_owned(), request.params.clone());
    obj.insert("id".to_owned(), request.id.clone());

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

fn check_version(obj: &json::Object) -> proto::Result<()> {
    match obj.get("jsonrpc") {
        None => {
            let ierr = InternalError::new(InternalErrorKind::InvalidVersion,
                                          "Invalid JSON-RPC version",
                                          Some("Missing `jsonpc` field".to_owned()));
            Err(proto::Error::InternalError(ierr))
        },
        Some(&Json::String(ref ver)) => {
            match &ver[..] {
                "2.0" => Ok(()),
                _ => {
                    let ierr = InternalError::new(InternalErrorKind::InvalidVersion,
                                                  "Invalid JSON-RPC version",
                                                  Some(format!("Expecting JSON-RPC 2.0, but found {}", ver)));
                    Err(proto::Error::InternalError(ierr))
                }
            }
        },
        v => {
            let ierr = InternalError::new(InternalErrorKind::InvalidVersion,
                                          "Invalid JSON-RPC version",
                                          Some(format!("Expecting JSON-RPC 2.0, but found {:?}", v)));
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

#[cfg(test)]
mod test {
    use std::io::{Cursor, Read, Write};

    use proto::{Request, ClientSender, ClientReceiver, ServerResponse};

    use rustc_serialize::json::{Array, Json};

    use super::Client;

    #[test]
    fn test_spec20_client_request() {
        use std::str;

        let params: Array = vec![
            Json::String("ping".to_owned()),
        ];

        let request = Request::new("echo".to_owned(),
                                   Json::Array(params),
                                   Json::U64(1));

        let mut buf = Cursor::new(vec![]);

        {
            let mut client = Client::new(&mut buf);
            client.request(&request).unwrap();
        }

        let expected = b"{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"echo\",\"params\":[\"ping\"]}";
        assert_eq!(&expected[..], &buf.get_ref()[..]);
    }
}
