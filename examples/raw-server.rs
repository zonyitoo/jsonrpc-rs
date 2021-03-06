extern crate jsonrpc;
extern crate rustc_serialize;
extern crate bufstream;
#[macro_use]
extern crate log;
extern crate fern;
extern crate chrono;

use std::net::TcpListener;
use std::thread;

use rustc_serialize::json::Json;

use bufstream::BufStream;

use chrono::Local;

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

fn dispatcher(req: Request) -> Option<Response> {
    debug!("Dispatching for request: {:?}", req);
    match &req.method[..] {
        "echo" => Some(echo(req)),
        "add" => Some(add(req)),
        "notify" => None,
        _ => Some(Response::error(errors::MethodNotFound::new(), req.id))
    }
}

fn main() {
    let logger_config = fern::DispatchConfig {
        format: Box::new(move|msg: &str, level: &log::LogLevel, location: &log::LogLocation| {
            format!("[{}][{}] [{}] {}", Local::now().format("%Y-%m-%d][%H:%M:%S"),
                    level, location.__module_path, msg)
        }),
        output: vec![fern::OutputConfig::stderr()],
        level: log::LogLevelFilter::Trace
    };

    fern::init_global_logger(logger_config, log::LogLevelFilter::Debug).unwrap();

    let acceptor = TcpListener::bind("127.0.0.1:8007").unwrap();

    for incoming in acceptor.incoming() {
        let stream = incoming.unwrap();

        thread::spawn(move|| {
            let peer_addr = stream.peer_addr().unwrap();
            info!("Accepted new connection {:?} ...", peer_addr);
            let mut stream = BufStream::new(stream);
            {
                let mut server = ServerStream::new(&mut stream);

                loop {
                    match server.get_request() {
                        Ok(Some(ClientRequest::Single(req))) => {
                            let resp = dispatcher(req);
                            match resp {
                                Some(res) => {
                                    debug!("Send response to {:?}: {:?}", peer_addr, res);
                                    server.response(res).unwrap();
                                }
                                None => {
                                    debug!("It's a notify");
                                }
                            }

                        },
                        Ok(Some(ClientRequest::Batch(reqs))) => {
                            let resps = reqs.into_iter().map(|r| dispatcher(r)).
                                collect::<Vec<Option<Response>>>();

                            let mut final_response = vec![];
                            for res in resps {
                                match res {
                                    Some(rr) => final_response.push(rr),
                                    None => debug!("no notify response")
                                }
                            }

                            debug!("Send response to {:?}: {:?}", peer_addr, final_response);
                            server.batch_response(final_response).unwrap();
                        },
                        Ok(None) => {
                            // EOF
                            break;
                        },
                        Err(err) => {
                            error!("Err {:?}: {:?}", peer_addr, err);
                            break;
                        }
                    }
                }
            }

            info!("Client {:?} finished", peer_addr);
        });
    }
}
