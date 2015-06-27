#![feature(io)]

extern crate rustc_serialize;

use rustc_serialize::Encodable;

pub use error::{Error, RpcError};

pub mod error;
pub mod proto;

pub type RpcResult<T: Encodable> = Result<T, Error>;
