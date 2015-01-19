extern crate hyper;

use std::os;

use hyper::Url;
use hyper::client::Request;
use hyper::method::Method;

fn main() {
    let s = if os::args {
        "http://httpbin.org/status/200".to_string()
    } else {
        os::args()[1].clone()
    };

    let url = match Url::parse(&s[]) {
        Ok(url) => url,
        Err(_) => panic!("Uh oh.")
    };

    println!("> get: {}", url);
    let fresh_request = match Request::new(Method::Get, url) {
        Ok(request) => request,
        Err(_) => panic!("Whoops.")
    };
    let streaming_request = match fresh_request.start() {
        Ok(request) => request,
        Err(_) => panic!("Noooo.")
    };
    let mut response = match streaming_request.send() {
        Ok(response) => response,
        Err(_) => panic!("So close...")
    };

    println!("< status code: {}", response.status);
    let content = match response.read_to_string() {
        Ok(content) => content,
        Err(_) => panic!("I give up.")
    };
    println!("{}", content);
}
