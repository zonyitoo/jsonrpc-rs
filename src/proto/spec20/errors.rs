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

pub const ERRCODE_PARSE_ERROR: i64 = -32700;
pub const ERRCODE_INVALID_REQUEST: i64 = -32600;
pub const ERRCODE_METHOD_NOT_FOUND: i64 = -32601;
pub const ERRCODE_INVALID_PARAMS: i64 = -32602;
pub const ERRCODE_INTERNAL_ERROR: i64 = -32603;

#[allow(non_snake_case)]
pub mod ParseError {
    use rustc_serialize::json::{ToJson, Json};
    use proto::ProtocolError;

    use super::*;

    pub fn new() -> ProtocolError {
        with_detail(None::<Json>)
    }

    pub fn with_detail<D: ToJson>(detail: Option<D>) -> ProtocolError {
        ProtocolError::new(ERRCODE_PARSE_ERROR, "Parse error".to_owned(), detail)
    }
}

#[allow(non_snake_case)]
pub mod InvalidRequest {
    use rustc_serialize::json::{ToJson, Json};
    use proto::ProtocolError;

    use super::*;

    pub fn new() -> ProtocolError {
        with_detail(None::<Json>)
    }

    pub fn with_detail<D: ToJson>(detail: Option<D>) -> ProtocolError {
        ProtocolError::new(ERRCODE_INVALID_REQUEST, "Invalid Request".to_owned(), detail)
    }
}

#[allow(non_snake_case)]
pub mod MethodNotFound {
    use rustc_serialize::json::{ToJson, Json};
    use proto::ProtocolError;

    use super::*;

    pub fn new() -> ProtocolError {
        with_detail(None::<Json>)
    }

    pub fn with_detail<D: ToJson>(detail: Option<D>) -> ProtocolError {
        ProtocolError::new(ERRCODE_METHOD_NOT_FOUND, "Method not found".to_owned(), detail)
    }
}

#[allow(non_snake_case)]
pub mod InvalidParams {
    use rustc_serialize::json::{ToJson, Json};
    use proto::ProtocolError;

    use super::*;

    pub fn new() -> ProtocolError {
        with_detail(None::<Json>)
    }

    pub fn with_detail<D: ToJson>(detail: Option<D>) -> ProtocolError {
        ProtocolError::new(ERRCODE_INVALID_PARAMS, "Invalid params".to_owned(), detail)
    }
}

#[allow(non_snake_case)]
pub mod InternalError {
    use rustc_serialize::json::{ToJson, Json};
    use proto::ProtocolError;

    use super::*;

    pub fn new() -> ProtocolError {
        with_detail(None::<Json>)
    }

    pub fn with_detail<D: ToJson>(detail: Option<D>) -> ProtocolError {
        ProtocolError::new(ERRCODE_INTERNAL_ERROR, "Internal error".to_owned(), detail)
    }
}

#[allow(non_snake_case)]
pub mod ServerError {
    use rustc_serialize::json::{ToJson, Json};
    use proto::ProtocolError;

    pub fn new(code: i64) -> ProtocolError {
        with_detail(code, None::<Json>)
    }

    pub fn with_detail<D: ToJson>(code: i64, detail: Option<D>) -> ProtocolError {
        assert!(code <= -32000 && code >= -32099, "ServerError code must be in [-32099, -32000]");

        ProtocolError::new(code, "Server error".to_owned(), detail)
    }
}
