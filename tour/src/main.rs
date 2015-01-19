extern crate hyper;

use std::os;

// use hyper::Url;
// use hyper::client::Request;
// use hyper::method::Method;
use std::io::net::ip::Ipv4Addr;
use hyper::server::{Server,Request,Response,Fresh};
use hyper::status::StatusCode;

fn hello(_: Request, mut res: Response<Fresh>) {
    *res.status_mut() = StatusCode::Ok;
    let mut res = res.start().unwrap();
    res.write(b"<html><head></head><body><h1>Hello World!</h1></body></html>");
    res.end().unwrap();
}

fn main() {
    let server = Server::http(Ipv4Addr(127, 0, 0, 1), 1337);
    server.listen(hello).unwrap();
}
