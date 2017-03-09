extern crate tiny_http;
#[macro_use]
extern crate rouille;
#[macro_use]
extern crate specs;
extern crate yaml_rust;
#[macro_use]
extern crate serde_derive;
extern crate dotenv;
extern crate envy;

use dotenv::dotenv;
use rouille::{Request};

use std::ascii::AsciiExt;
use std::env;
use std::io::{self, Read, Write};
use std::net::SocketAddr;

mod world;

#[derive(Deserialize, Debug)]
struct CGIRequest {
    request_method: String,
    request_uri: String,
    headers: Vec<(String, String)>,
    #[serde(rename = "HTTP_UPGRADE_INSECURE_REQUESTS")]
    https: bool,
    remote_addr: SocketAddr,
}

fn main() {
    dotenv::dotenv().ok();
    match envy::from_env::<CGIRequest>() {
       Ok(config) => println!("{:#?}", config),
       Err(error) => panic!("{:#?}", error),
    }
    let mut req_builder = CGIRequest {
        request_method: "OPTIONS".into(),
        request_uri: "/".into(),
        headers: Vec::new(),
        https: false,
        remote_addr: "127.0.0.1:80".parse().unwrap(),
    };

    for (k, v) in env::vars() {
        //println!("{:?}: {:?}", k, v);
        match &*k {
            "AUTH_TYPE" | "CONTENT_LENGTH" | "CONTENT_TYPE" | "GATEWAY_INTERFACE" | "PATH_INFO" | "PATH_TRANSLATED" | "QUERY_STRING" | "REMOTE_HOST" | "REMOTE_IDENT" | "REMOTE_USER" | "SCRIPT_NAME" | "SERVER_NAME" | "SERVER_PORT" | "SERVER_SOFTWARE" => req_builder.headers.push((k, v)),
    "SERVER_PROTOCOL" => {
        let version = v.chars().filter(|c| !c.is_numeric())
            .map(|c| c as u8).collect::<Vec<u8>>();
        tiny_http::HTTPVersion(version[0], version[1]);
    }
    "REQUEST_METHOD" => req_builder.request_method = v,
    "REQUEST_URI" => req_builder.request_uri = v,
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

    // I know it's fake but I'm not sure how to build a request from environment variables
    let request = Request::fake_http_from(
        req_builder.remote_addr,
        req_builder.request_method,
        req_builder.request_uri,
        req_builder.headers,
        body.into(),
    );

    let rouille_response = router!{request,
                          (GET) (/) => {
                              rouille::Response::redirect_302("/hello")
                          },
                          (GET) (/hello) => {
                              rouille::Response::text("hello")
                          },
                          _ => rouille::Response::text("")
                      };

    let mut upgrade_header = "".into();

    // writing the response
    let (res_data, res_len) = rouille_response.data.into_reader_and_size();
    let mut response = tiny_http::Response::empty(rouille_response.status_code)
        .with_data(res_data, res_len);
    let mut response_headers = Vec::new();
    for (key, value) in request.headers() {
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
    let mut writer = stdout.lock();
    response.raw_print(
        writer,
        tiny_http::HTTPVersion(1, 1),
        &response_headers[..],
        true,
        None,
        );

    ::std::process::exit(0);
}
