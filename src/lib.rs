#![feature(io)]
extern crate rustc_serialize;
#[macro_use]
extern crate log;

use rustc_serialize::Encodable;

pub use error::{Error, RpcError};

pub mod error;
pub mod proto;
pub use proto::spec20::client;

pub type RpcResult<T: Encodable> = Result<T, Error>;
