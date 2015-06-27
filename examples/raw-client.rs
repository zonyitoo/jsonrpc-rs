extern crate jsonrpc;
extern crate rustc_serialize;

use std::net::TcpStream;

use rustc_serialize::json::Json;

use jsonrpc::proto::{Request, Client};
use jsonrpc::proto::trans::{SendRequest, GetResponse};

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8007").unwrap();

    let mut client = Client::new(&mut stream);

    let request = Request::new("time".to_owned(),
                               None,
                               Json::U64(1));
    println!("Request: {:?}", request);

    client.request(request).unwrap();

    let response = client.get_response();

    println!("Response: {:?}", response);
}
