extern crate jsonrpc;
extern crate rustc_serialize;
extern crate bufstream;

use std::net::TcpListener;

use rustc_serialize::json::{Json, ToJson};

use bufstream::BufStream;

use jsonrpc::proto::{self, Response, Server};
use jsonrpc::proto::trans::{GetRequest, SendResponse, Request};
use jsonrpc::proto::spec20::errors;

fn echo(req: proto::Request) -> Response {
    Response::new(req.params, None, req.id)
}

fn add(req: proto::Request) -> Response {
    let params = req.params.unwrap();
    let a = match params[0].as_i64() {
        Some(x) => x,
        None => {
            return Response::new(None,
                                 Some(errors::InvalidParams::new().to_json()),
                                 req.id);
        }
    };
    let b = match params[1].as_i64() {
        Some(x) => x,
        None => {
            return Response::new(None,
                                 Some(errors::InvalidParams::new().to_json()),
                                 req.id);
        }
    };

    Response::new(Some(Json::I64(a + b)),
                  None,
                  req.id)
}

fn dispatcher(req: proto::Request) -> Response {
    match &req.method[..] {
        "echo" => echo(req),
        "add" => add(req),
        _ => Response::new(None,
                           Some(errors::MethodNotFound::new().to_json()),
                           req.id)
    }
}

fn main() {
    let acceptor = TcpListener::bind("127.0.0.1:8007").unwrap();

    for incoming in acceptor.incoming() {
        let stream = incoming.unwrap();
        let mut stream = BufStream::new(stream);
        println!("Accepted new connection ...");
        let mut server = Server::new(&mut stream);

        loop {
            match server.get_request() {
                Ok(Request::Single(req)) => {
                    let resp = dispatcher(req);
                    server.response(resp).unwrap();
                },
                Ok(Request::Batch(reqs)) => {
                    let resps = reqs.into_iter().map(|r| dispatcher(r)).collect::<Vec<Response>>();
                    server.batch_response(resps).unwrap();
                },
                Err(..) => {
                    break;
                }
            }
        }
    }
}
