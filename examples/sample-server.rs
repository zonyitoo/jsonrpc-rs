extern crate jsonrpc;
extern crate rustc_serialize;
#[macro_use]
extern crate log;
extern crate bufstream;
extern crate fern;
extern crate chrono;

use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream, SocketAddr, ToSocketAddrs};

use rustc_serialize::json::Json;

use chrono::Local;

use bufstream::BufStream;

use jsonrpc::proto::{Request, Response};
use jsonrpc::proto::trans::{GetRequest, SendResponse, ClientRequest};
use jsonrpc::proto::spec20::{errors, ServerStream};
use jsonrpc::RpcServerResult;

trait Dispatcher {
    fn dispatch(&mut self, req: Request) -> Option<Response>;
}

trait Server {
    type Stream: Read + Write;

    fn accept(&mut self) -> io::Result<(Self::Stream, SocketAddr)>;
}

// Generated definition

trait CalculatorService {
    fn echo(&mut self, msg: String) -> RpcServerResult<String>;

    // Notify service
    fn touch(&mut self, msg: String) -> RpcServerResult<()>;
}

struct CalculatorServiceDispatcher<S: CalculatorService> {
    service: S,
}

impl<S: CalculatorService> CalculatorServiceDispatcher<S> {
    pub fn new(service: S) -> CalculatorServiceDispatcher<S> {
        CalculatorServiceDispatcher {
            service: service,
        }
    }

    fn echo(&mut self, req: Request) -> Option<Response> {
        let id = req.id;

        let params = match req.params {
            Some(params) => params,
            None =>
                return Some(errors::InvalidParams::new())
                    .map(|err| Response::error(err, id))
        };

        let result = match params {
            Json::Object(mut obj) => {
                let msg: String = match obj.remove("msg") {
                    Some(Json::String(msg)) => msg,
                    Some(..) | None =>
                        return Some(errors::InvalidParams::new())
                            .map(|err| Response::error(err, id))
                };

                self.service.echo(msg)
            },
            Json::Array(mut arr) => {
                match (arr.pop(), ) {
                    (Some(Json::String(msg)), ) => {
                        self.service.echo(msg)
                    },
                    _ => return Some(errors::InvalidParams::new())
                            .map(|err| Response::error(err, id))
                }
            },
            _ => {
                return Some(errors::InvalidParams::new())
                    .map(|err| Response::error(err, id))
            }
        };

        match result {
            Ok(r) => {
                Some(Response::result(r, id))
            },
            Err(err) => {
                Some(Response::error(err, id))
            }
        }
    }

    fn touch(&mut self, req: Request) -> Option<Response> {
        let id = req.id;

        let params = match req.params {
            Some(params) => params,
            None =>
                return Some(errors::InvalidParams::new())
                    .map(|err| Response::error(err, id))
        };

        let result = match params {
            Json::Object(mut obj) => {
                let msg: String = match obj.remove("msg") {
                    Some(Json::String(msg)) => msg,
                    Some(..) | None => return Some(errors::InvalidParams::new())
                                .map(|err| Response::error(err, id))
                };

                self.service.echo(msg)
            },
            Json::Array(mut arr) => {
                match (arr.pop(), ) {
                    (Some(Json::String(msg)), ) => {
                        self.service.echo(msg)
                    },
                    _ => return Some(errors::InvalidParams::new())
                            .map(|err| Response::error(err, id))
                }
            },
            _ => {
                return Some(errors::InvalidParams::new())
                        .map(|err| Response::error(err, id))
            }
        };

        match result {
            Ok(..) => None,
            Err(err) => {
                Some(Response::error(err, id))
            }
        }
    }
}

impl<S: CalculatorService> Dispatcher for CalculatorServiceDispatcher<S> {
    fn dispatch(&mut self, req: Request) -> Option<Response> {
        match &req.method[..] {
            "echo" => {
                self.echo(req)
            },
            "touch" => {
                self.touch(req)
            },
            _ => {
                Some(errors::MethodNotFound::with_detail(
                        Json::String(format!("Unknown method {:?}", req.method))))
                    .map(move|err| Response::error(err, req.id))
            }
        }
    }
}

struct CalculatorServer<D, S>
    where D: CalculatorService,
          S: Server
{
    dispatcher: CalculatorServiceDispatcher<D>,
    server: S,
}

impl<D, S> CalculatorServer<D, S>
    where D: CalculatorService,
          S: Server
{
    pub fn new(service: D, server: S) -> CalculatorServer<D, S> {
        CalculatorServer {
            dispatcher: CalculatorServiceDispatcher::new(service),
            server: server,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        loop {
            let (stream, peer_addr) = match self.server.accept() {
                Ok(s) => s,
                Err(err) => return Err(err)
            };

            info!("Got connection from {:?}", peer_addr);

            let mut stream = BufStream::new(stream);
            {
                let mut server = ServerStream::new(&mut stream);

                loop {
                    match server.get_request() {
                        Ok(Some(ClientRequest::Single(req))) => {
                            trace!("Request {:?}", req);
                            let resp = self.dispatcher.dispatch(req);

                            if let Some(resp) = resp {
                                trace!("Response {:?}", resp);
                                server.response(resp).unwrap();
                            }
                        },
                        Ok(Some(ClientRequest::Batch(reqs))) => {
                            trace!("Request {:?}", reqs);
                            let resps = reqs.into_iter()
                                            .map(|r| self.dispatcher.dispatch(r))
                                            .filter_map(|r| r)
                                            .collect::<Vec<Response>>();
                            trace!("Response {:?}", resps);
                            server.batch_response(resps).unwrap();
                        },
                        Ok(None) => {
                            // EOF
                            break;
                        },
                        Err(err) => {
                            error!("Err {:?}", err);
                            break;
                        }
                    }
                }
            }
        }
    }
}


// User implementation
struct StdTcpServer {
    listener: TcpListener,
}

impl Server for StdTcpServer {
    type Stream = TcpStream;

    fn accept(&mut self) -> io::Result<(TcpStream, SocketAddr)> {
        self.listener.accept()
    }
}

impl StdTcpServer {
    pub fn bind<A: ToSocketAddrs>(addrs: A) -> io::Result<StdTcpServer> {
        Ok(StdTcpServer {
            listener: try!(TcpListener::bind(addrs)),
        })
    }
}

struct MyCalculatorService;

impl CalculatorService for MyCalculatorService {
    fn echo(&mut self, msg: String) -> RpcServerResult<String> {
        Ok(msg)
    }

    fn touch(&mut self, msg: String) -> RpcServerResult<()> {
        println!("Touch {:?}", msg);

        Ok(())
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

    let server = StdTcpServer::bind("127.0.0.1:8080").unwrap();
    let mut rpc_server = CalculatorServer::new(MyCalculatorService, server);
    rpc_server.run().unwrap()
}
