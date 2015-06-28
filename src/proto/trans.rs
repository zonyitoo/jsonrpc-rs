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

use proto::{Request, Response, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum ClientRequest {
    Single(Request),
    Batch(Vec<Request>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServerResponse {
    Single(Response),
    Batch(Vec<Response>),
}

pub trait SendRequest {
    fn request(&mut self, request: Request) -> Result<()>;
    fn batch_request(&mut self, requests: Vec<Request>) -> Result<()>;
}

pub trait GetResponse {
    fn get_response(&mut self) -> Result<ServerResponse>;
}

pub trait SendResponse {
    fn response(&mut self, response: Response) -> Result<()>;
    fn batch_response(&mut self, responses: Vec<Response>) -> Result<()>;
}

pub trait GetRequest {
    fn get_request(&mut self) -> Result<ClientRequest>;
}
