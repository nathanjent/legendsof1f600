extern crate iron;
#[macro_use]
extern crate rouille;
extern crate route_recognizer;
#[macro_use]
extern crate router;
extern crate specs;
extern crate yaml_rust;

use iron::prelude::*;
use iron::status;
use router::Router;
//use rouille::Server;
//use rouille::Response;
//use route_recognizer::{Router};

mod world;

fn main() {
    let url = ::std::env::var("URL").unwrap_or("/".to_string());
    let remote_addr = ::std::env::var("REMOTE_ADDR").unwrap_or("127.0.0.1:8080".to_string());
    let local_addr = ::std::env::var("LOCAL_ADDR").unwrap_or("127.0.0.1:8080".to_string());
    let headers_str = ::std::env::var("URL").unwrap_or("/".to_string());
    let body = ::std::env::var("URL").unwrap_or("/".to_string());
    let method = ::std::env::var("URL").unwrap_or("/".to_string());
    let type_map_str = ::std::env::var("URL").unwrap_or("/".to_string());

    let mut headers = Headers::new();
    let mut type_map = TypeMap::new();

//    let server = Server::new("localhost:0", |request| {
//        Response::text(format!("hello world {:?}", request.method()))
//    }).unwrap();
//    server.poll();

    let mut req = Request {
        url: Url::parse(url).unwrap(),
        remote_addr: SocketAddr::from_str(remote_addr).unwrap(),
        local_addr: SocketAddr::from_str(local_addr).unwrap(),
        headers: Headers,
        body: Body<'a, 'b>,
        method: Method::from_str(method).unwrap(),
        extensions: TypeMap,
    };

    let router = router!(index: get "/" => handler,
                         query: get "/:query" => handler);

    fn handler(req: &mut Request) -> IronResult<Response> {
        let ref query = req.extensions.get::<Router>().unwrap().find("query").unwrap_or("/");
        Ok(Response::with((status::Ok, *query)))
    }

    //let mut router = Router::new();
    //router.add("/", "ContentType: text/html; charset=UTF-8\n\nHello World!");

    //let m = router.recognize(&*url).unwrap();

    //println!("{}", m.handler);
    ::std::process::exit(0);
}
