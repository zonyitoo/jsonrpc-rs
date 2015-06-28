extern crate jsonrpc;
extern crate rustc_serialize;
extern crate bufstream;

use std::net::TcpListener;
use std::thread;

use rustc_serialize::json::Json;

use bufstream::BufStream;

use jsonrpc::proto::{Request, Response};
use jsonrpc::proto::trans::{GetRequest, SendResponse, ClientRequest};
use jsonrpc::proto::spec20::{errors, ServerStream};

fn echo(req: Request) -> Response {
    Response::result(req.params, req.id)
}

fn add(req: Request) -> Response {
    let params = match req.params {
        Some(Json::Array(ref p)) if p.len() == 2 => p,
        _ => {
            return Response::error(errors::InvalidParams::new(), req.id);
        }
    };

    let a = match params[0].as_i64() {
        Some(x) => x,
        None => {
            return Response::error(errors::InvalidParams::new(), req.id);
        }
    };

    let b = match params[1].as_i64() {
        Some(x) => x,
        None => {
            return Response::error(errors::InvalidParams::new(), req.id);
        }
    };

    Response::result(Json::I64(a + b), req.id)
}

fn dispatcher(req: Request) -> Response {
    match &req.method[..] {
        "echo" => echo(req),
        "add" => add(req),
        _ => Response::error(errors::MethodNotFound::new(), req.id)
    }
}

fn main() {
    let acceptor = TcpListener::bind("127.0.0.1:8007").unwrap();

    for incoming in acceptor.incoming() {
        let stream = incoming.unwrap();

        thread::spawn(move|| {
            let mut stream = BufStream::new(stream);
            println!("Accepted new connection ...");
            let mut server = ServerStream::new(&mut stream);

            loop {
                match server.get_request() {
                    Ok(ClientRequest::Single(req)) => {
                        let resp = dispatcher(req);
                        server.response(resp).unwrap();
                    },
                    Ok(ClientRequest::Batch(reqs)) => {
                        let resps = reqs.into_iter().map(|r| dispatcher(r)).collect::<Vec<Response>>();
                        server.batch_response(resps).unwrap();
                    },
                    Err(..) => {
                        break;
                    }
                }
            }
        });
    }
}
