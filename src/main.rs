extern crate conduit_router;
extern crate conduit;
extern crate semver;
#[macro_use]
extern crate specs;
extern crate yaml_rust;

use conduit_router::{RequestParams, RouteBuilder};
use conduit::{Handler, Method, Scheme, Host, Headers, Extensions, TypeMap};
use std::collections::HashMap;
use std::env;
use std::io;
use std::net::SocketAddr;

mod world;

fn main() {
    let url = env::var("REQUEST_URI").unwrap_or("/".to_string());
    let remote_addr = env::var("REMOTE_ADDR").unwrap_or("127.0.0.1:80".to_string());
    let local_addr = env::var("SERVER_ADDR").unwrap_or("127.0.0.1:80".to_string());
    let headers_str = env::var("URL").unwrap_or("/".to_string());
    let body = env::var("URL").unwrap_or("/".to_string());
    let method_str = env::var("REQUEST_METHOD").unwrap_or("OPTIONS".to_string());
    let type_map_str = env::var("URL").unwrap_or("/".to_string());

    let method = match &*method_str {
        "GET" => Method::Get,
        "POST" => Method::Post,
        "PUT" => Method::Put,
        "DELETE" => Method::Delete,
        "HEAD" => Method::Head,
        "OPTIONS" => Method::Options,
        s @ _ => Method::Other(s.to_string()),
    };

    let mut router = RouteBuilder::new();
    router.get("/", handler);
    router.map(Method::Options, "/", handler);

    let m = router.recognize(&method, &*url).unwrap();

    println!("{:?}", m.params);

    ::std::process::exit(0);
}

fn handler(req: &mut conduit::Request) -> io::Result<conduit::Response> {
    let mut res = vec!();
    res.push(req.params()["id"].clone());
    res.push(format!("{:?}", req.method()));

    Ok(conduit::Response {
        status: (200, "OK"),
        headers: HashMap::new(),
        body: Box::new(io::Cursor::new(res.connect(", ").into_bytes()))
    })
}
