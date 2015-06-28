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

pub use self::client::Client;
pub use self::server::Server;

use rustc_serialize::json::{self, Object, Json};

use proto::{self, InternalErrorKind, InternalError};

pub mod client;
pub mod server;

fn check_version(obj: &json::Object) -> proto::Result<()> {
    match obj.get("jsonrpc") {
        None => {
            let ierr = InternalError::new(InternalErrorKind::InvalidVersion,
                                          "Invalid JSON-RPC version",
                                          Some("Missing `jsonrpc` field".to_owned()));
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

#[cfg(test)]
mod test {
    use std::io::{Cursor, Write, Seek, SeekFrom};

    use proto::{Request, Response};
    use proto::trans::{self, SendRequest, GetRequest, GetResponse, SendResponse};

    use rustc_serialize::json::{Array, Json};

    use super::{Client, Server};

    #[test]
    fn test_spec20_request() {
        let params: Array = vec![
            Json::String("ping".to_owned()),
        ];

        let request = Request::new("echo".to_owned(),
                                   Some(Json::Array(params)),
                                   Json::U64(1));

        let mut buf = Cursor::new(vec![]);

        {
            let mut client = Client::new(&mut buf);
            client.request(request.clone()).unwrap();
        }
        buf.flush().unwrap();

        let expected = b"{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"echo\",\"params\":[\"ping\"]}";
        assert_eq!(&expected[..], &buf.get_ref()[..]);

        let request_svr = {
            buf.seek(SeekFrom::Start(0)).unwrap();
            let mut server = Server::new(&mut buf);
            server.get_request().unwrap()
        };

        assert_eq!(trans::Request::Single(request), request_svr);
    }

    #[test]
    fn test_spec20_server_response() {
        let result: Json = Json::String("pong".to_owned());

        let response = Response::new(Some(result), None, Json::U64(1));

        let mut buf = Cursor::new(vec![]);

        {
            let mut server = Server::new(&mut buf);
            server.response(response.clone()).unwrap();
        }

        let expected = b"{\"id\":1,\"jsonrpc\":\"2.0\",\"result\":\"pong\"}";
        assert_eq!(&expected[..], &buf.get_ref()[..]);

        let response_cli = {
            buf.seek(SeekFrom::Start(0)).unwrap();
            let mut client = Client::new(&mut buf);
            client.get_response().unwrap()
        };

        assert_eq!(trans::Response::Single(response), response_cli);
    }
}
