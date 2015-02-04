extern crate hyper;
#[macro_use] extern crate log;

use std::old_io::util::copy;
use std::old_io::net::ip::Ipv4Addr;

use hyper::{Get, Post};
use hyper::header::ContentLength;
use hyper::server::{Server,Request,Response,Fresh};
use hyper::status::StatusCode;
use hyper::uri::RequestUri::AbsolutePath;

macro_rules! try_return {
    ($e:expr) => ({
        match $e {
            Ok(v) => v,
            Err(e) => {
                error!("Error: {}", e);
                return
            }
        }
    })
}

fn echo(mut req: Request, mut res: Response<Fresh>) {
    match req.uri {
        AbsolutePath(ref path) => match (&req.method, &path[]) {
            (&Get, "/") => {
                let out = b"<!DOCTYPE><html><head></head><body><h1>Hello World!</h1></body></html>";
                res.headers_mut().set(ContentLength(out.len() as u64));
                let mut res = try_return!(res.start());
                try_return!(res.write(out));
                try_return!(res.end());
                return;
            }
            (&Get, "/echo") => {
                let out = b"Try POST /echo";
                res.headers_mut().set(ContentLength(out.len() as u64));
                let mut res = try_return!(res.start());
                try_return!(res.write(out));
                try_return!(res.end());
                return;
            }
            (&Post, _) => (),
            _ => {
                *res.status_mut() = hyper::NotFound;
                try_return!(res.start().and_then(|res| res.end()));
                return
            }
        },
        _ => {
            try_return!(res.start().and_then(|res| res.end()));
            return
        }
    }

    let mut res = try_return!(res.start());
    try_return!(copy(&mut req, &mut res));
    try_return!(res.end());
}

fn main() {
    let server = Server::http(Ipv4Addr(127, 0, 0, 1), 1337);
    let mut listening = server.listen(echo).unwrap();
    println!("Listening on port 1337");
    listening.await();
}
