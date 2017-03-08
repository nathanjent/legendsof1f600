extern crate tiny_http;
#[macro_use]
extern crate rouille;
#[macro_use]
extern crate specs;
extern crate yaml_rust;

use rouille::{Request};

use std::env;
use std::net::SocketAddr;
use std::io::{self, Write};

mod world;

struct RequestBuilder {
        method: String,
        url: String,
        headers: Vec<(String, String)>,
        https: bool,
        remote_addr: SocketAddr,
}

fn main() {
    let mut req_builder = RequestBuilder {
        method: "OPTIONS".into(),
        url: "/".into(),
        headers: Vec::new(),
        https: false,
        remote_addr: "127.0.0.1:80".parse().unwrap(),
    };

    for (k, v) in env::vars() {
        //println!("{:?}: {:?}", k, v);
        match &*k {
            "AUTH_TYPE" | "CONTENT_LENGTH" | "CONTENT_TYPE" | "GATEWAY_INTERFACE" | "PATH_INFO" | "PATH_TRANSLATED" | "QUERY_STRING" | "REMOTE_HOST" | "REMOTE_IDENT" | "REMOTE_USER" | "SCRIPT_NAME" | "SERVER_NAME" | "SERVER_PORT" | "SERVER_PROTOCOL" | "SERVER_SOFTWARE" => req_builder.headers.push((k, v)),
    "REQUEST_METHOD" => req_builder.method = v,
    "REQUEST_URI" => req_builder.url = v,
    "REMOTE_ADDR" => req_builder.remote_addr = v.parse().unwrap(),
    _ => {},
        }
    }

    // TODO get body
    let body = String::new();

    //let req = tiny_http::request::new_request(
    //    false,
    //    Method::from_str(req_builder.method),
    //    req_builder.url,
    //    HTTPVersion(1, 1),
    //    req_builder.headers,
    //    req_builder.remote_addr,

    //    );

                                             );
    // I know it's fake but I'm not sure how to build a request from environment variables
    let request = Request::fake_http_from(
        req_builder.remote_addr,
        req_builder.method,
        req_builder.url,
        req_builder.headers,
        body.into(),
    );

    let rouille_response = router!{request,
                          (GET) (/) => {
                              rouille::Response::text("")
                          },
                          (GET) (/hello) => {
                              rouille::Response::text("hello")
                          },
                          _ => rouille::Response::text("")
                      };


    // writing the response
    let (res_data, res_len) = rouille_response.data.into_reader_and_size();
    let mut response = tiny_http::Response::empty(rouille_response.status_code)
        .with_data(res_data, res_len);
    let mut response_headers = Vec::new();
    for (key, value) in req_builder.headers {
        if key.eq_ignore_ascii_case("Content-Length") {
            continue;
        }

        if key.eq_ignore_ascii_case("Upgrade") {
            upgrade_header = value;
            continue;
        }

        if let Ok(header) = tiny_http::Header::from_bytes(key.as_bytes(), value.as_bytes()) {
            response_headers.push(header);
        } else {
            // TODO: ?
        }
    }

    let stdout = io::stdout();
    let writer = stdout.lock();
    response.raw_print(
        writer,
        HTTPVersion(1, 1),
        response_headers,
        true,
        None,
        );
    writer.flush();

    ::std::process::exit(0);
}
