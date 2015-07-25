#![feature(io)]
extern crate rustc_serialize;
#[macro_use]
extern crate log;

use rustc_serialize::json::ToJson;

pub use error::Error;

pub mod error;
pub mod proto;
// pub mod client;

pub type RpcResult<T: ToJson> = Result<T, Error>;

pub type RpcServerResult<T: ToJson> = Result<T, proto::ProtocolError>;
