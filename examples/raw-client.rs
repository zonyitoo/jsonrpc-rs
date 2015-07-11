extern crate jsonrpc;
extern crate rustc_serialize;
extern crate bufstream;
extern crate chrono;
extern crate rand;
#[macro_use]
extern crate log;
extern crate fern;

use std::net::TcpStream;

use rustc_serialize::json::Json;

use bufstream::BufStream;

use chrono::{UTC, Local};

use jsonrpc::proto::Request;
use jsonrpc::proto::spec20::ClientStream;
use jsonrpc::proto::trans::{SendRequest, GetResponse};

fn generate_id() -> u64 {
    UTC::now().timestamp() as u64 + rand::random::<u64>()
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

    let mut stream = BufStream::new(TcpStream::connect("127.0.0.1:8007").unwrap());
    let mut client = ClientStream::new(&mut stream);

    {

        let request = Request::new("echo".to_owned(),
                                   Some(Json::Array(vec![
                                        Json::String("ping".to_owned()),
                                   ])),
                                   Some(Json::U64(generate_id())));

        debug!("Request: {:?}", request);

        client.request(request).unwrap();

        let response = client.get_response();

        debug!("Response: {:?}", response);
    }

    {
        let request = Request::new("add".to_owned(),
                                   Some(Json::Array(vec![
                                            Json::U64(1),
                                            Json::U64(2),
                                        ])),
                                   Some(Json::U64(generate_id())));
        debug!("Request: {:?}", request);

        client.request(request).unwrap();

        let response = client.get_response();

        debug!("Response: {:?}", response);
    }

    // {
    //     let requests = (0..3).map(|_| {
    //         Request::new("echo".to_owned(),
    //                      Some(Json::Array(vec![
    //                         Json::String("ping".to_owned()),
    //                      ])),
    //                      Some(Json::U64(generate_id())))
    //     }).collect::<Vec<Request>>();
    //     debug!("Request: {:?}", requests);

    //     client.batch_request(requests).unwrap();

    //     let response = client.get_response();

    //     debug!("Response: {:?}", response);
    // }

    {
        let request = Request::new_notify("notify".to_owned(),
                                          Some(Json::Array(vec![
                                                    Json::U64(1),
                                                    Json::U64(2),
                                               ])));
        debug!("Request: {:?}", request);
        client.request(request).unwrap();

        let response = client.get_response();

        debug!("Response: {:?}", response);
    }
}
