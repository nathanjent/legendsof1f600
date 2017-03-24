#[macro_use]
extern crate rouille;
extern crate chrono;
#[macro_use]
extern crate specs;
extern crate yaml_rust;
#[macro_use]
extern crate serde_derive;
extern crate dotenv;
extern crate envy;
//#[macro_use]
//extern crate diesel;
//#[macro_use]
//extern crate diesel_codegen;
#[macro_use]
extern crate mysql;

use dotenv::dotenv;
use std::io::{self, Read, Write};
use std::panic;
use std::time::{Duration, Instant};
use rouille::Request;
use rouille::Response;

//use diesel::prelude::*;
//use diesel::mysql::MysqlConnection;

use std::ascii::AsciiExt;
use std::env;

mod world;

#[derive(Debug, PartialEq, Eq)]
struct Entity {
    id: i32,
    name: Option<String>,
    health: i32,
}

#[derive(Deserialize, Debug)]
struct EnvRequest {
    #[serde(rename = "REQUEST_METHOD")]
    request_method: String,
    #[serde(rename = "REQUEST_URI")]
    request_uri: String,
    #[serde(rename = "REMOTE_ADDR")]
    remote_addr: String,
    #[serde(rename = "REMOTE_PORT")]
    remote_port: u64,
    #[serde(rename = "CONTENT_LENGTH", default)]
    content_length: u64,
    #[serde(default = "http_headers")]
    headers: Vec<(String, String)>,
    #[serde(rename = "HTTP_UPGRADE_INSECURE_REQUESTS", default)]
    https: u8,
}

fn http_headers() -> Vec<(String, String)> {
    ::std::env::vars().filter_map(|(k, v)| {
       match k.split("HTTP_").nth(1) {
           Some(k) => Some((k.into(), v)),
           None => None,
       }
    }).collect::<Vec<_>>()
}

fn main() {
    dotenv().ok();
    //println!("{:?}", ::std::env::vars().collect::<Vec<_>>());

    let status = match handle() {
        Ok(_) => 0,
        Err(e) => {
            writeln!(io::stdout(), "Status: 500\r\n\r\n
                     <h1>500 Internal Server Error</h1> <p>{}</p>", e)
                .expect("Panic at write to STDOUT!");
            1
        }
    };
    ::std::process::exit(status);
}

fn handle() -> Result<(), Box<::std::error::Error>> {
    // Deserialize request from environment variables
    let request = envy::from_env::<EnvRequest>()?;
    //println!("{:?}", request);

    let database_url = env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set");
    //let connection = MysqlConnection::establish(&database_url)
    //        .expect(&format!("Error connecting to {}", database_url));

    let pool = mysql::Pool::new(&*database_url)?;
    pool.prep_exec(r"CREATE TEMPORARY TABLE tmp.entity (
            id int not null,
            name text,
            health int not null
        )", ())?;

    let entities = vec![
        Entity { id: 1, name: Some("Tree".into()), health: 100 },
        Entity { id: 2, name: None, health: 100 },
        Entity { id: 3, name: Some("Path".into()), health: 0 },
        Entity { id: 4, name: Some("Player".into()), health: 100 },
    ];

    for mut stmt in pool.prepare(r"INSERT INTO tmp.entity ( id, health, name)
                                VALUES (:id, :health, :name)").into_iter()
    {
        for e in entities.iter() {
            // `execute` takes ownership of `params` so we pass account name by reference.
            // Unwrap each result just to make sure no errors happened.
            stmt.execute(params!{
                "id" => e.id,
                "health" => e.health,
                "name" => &e.name,
            })?;
        }
    }
    println!("{:?}", pool.prep_exec("SHOW SCHEMAS", ())
            .map(|result| {
                result.map(|x| x.unwrap())
                    .map(|row| {
                        let row: String = mysql::from_row(row);
                        format!("{} ", row)
                    })
                    .collect::<String>()
            }).unwrap());

    let selected_entities: Vec<Entity> = pool.prep_exec("SELECT id, health, name FROM tmp.entity", ())
        .map(|result| {
            // In this closure we will map `QueryResult` to `Vec<Payment>`
            // `QueryResult` is iterator over `MyResult<row, err>` so first call to `map`
            // will map each `MyResult` to contained `row` (no proper error handling)
            // and second call to `map` will map each `row` to `Payment`
            result.map(|x| x.unwrap()).map(|row| {
                let (id, health, name) = mysql::from_row(row);
                Entity {
                    id: id,
                    health: health,
                    name: name,
                }
            }).collect() // Collect payments so now `QueryResult` is mapped to `Vec<Payment>`
        }).unwrap(); // Unwrap `Vec<Payment>`

    // Mysql gives no guarantees on order of returned rows without `ORDER BY`
    // so assume we are lukky.
    assert_eq!(entities, selected_entities);

    // Read request body from stdin
    let mut data = Vec::new();
    io::stdin().take(request.content_length).read_to_end(&mut data)?;

    // Generate a Rouille Request from the EnvRequest and body data from STDIN
    // These fake_http methods are most-likely for testing but serve our purposes.
    let request = match request.https {
        1 => Request::fake_https_from(
            format!("{}:{}", request.remote_addr, request.remote_port).parse()?,
            request.request_method,
            request.request_uri,
            request.headers,
            data),
        _ => Request::fake_http_from(
            format!("{}:{}", request.remote_addr, request.remote_port).parse()?,
            request.request_method,
            request.request_uri,
            request.headers,
            data),
    };

    // Route request 
    let _response = router!(request,
        // first route
        (GET) (/) => {
            // print the http headers from the request for fun!
            let mut s = String::new();
            for (k, v) in request.headers() {
                s.push_str(&*format!("{}: {}\r\n", k, v));
            }
            Response::text(s)
        },

        // second route
        (GET) (/hello) => {
            Response::html("<p>Hello world</p>")
        },

        // ... other routes here ...

        // default route
        _ => {
            Response::text("Default Space")
        }
    );

    // Send resulting response after routing
    send(&request, io::stdout(), || _response)?;
    Ok(())
}

/// Sends a response to STDOUT.
///
/// The CGI server receives the response through the pipe and sends it along.
/// TODO this is a modified version of the rouille log function, does it need to catch panics?
fn send<W, F>(rq: &Request, mut output: W, f: F)
    -> Result<(), Box<::std::error::Error>>
    where W: Write,
          F: FnOnce() -> Response
{
    let start_instant = Instant::now();
    let rq_line = format!("{} UTC - {} {}",
                          chrono::UTC::now().format("%Y-%m-%d %H:%M:%S%.6f"),
                          rq.method(),
                          rq.raw_url());

    // Calling the handler and catching potential panics.
    // Note that this we always resume unwinding afterwards, we can ignore the small panic-safety
    // mecanism of `catch_unwind`.
    let response = panic::catch_unwind(panic::AssertUnwindSafe(f));

    let elapsed_time = format_time(start_instant.elapsed());

    match response {
        Ok(response) => {
            for &(ref k, ref v) in response.headers.iter() {
                writeln!(output, "{}: {}", k, v)?;
            }
            //writeln!(output, "Status: {}", response.status_code)?;
            let (mut response_body, content_length) = response.data.into_reader_and_size();
            if let Some(content_length) = content_length {
                writeln!(output, "Content-Length: {}",  content_length)?;
            }
            writeln!(output, "")?;
            io::copy(&mut response_body, &mut output)?;
            writeln!(output, "")?;
        }
        Err(payload) => {
            // There is probably no point in printing the payload, as this is done by the panic
            // handler.
            let _ = writeln!(output, "{} - {} - PANIC!", rq_line, elapsed_time);
            panic::resume_unwind(payload);
        }
    }
    Ok(())
}

// copied from the rouille log module
fn format_time(duration: Duration) -> String {
    let secs_part = match duration.as_secs().checked_mul(1_000_000_000) {
        Some(v) => v,
        None => return format!("{}s", duration.as_secs() as f64),
    };

    let duration_in_ns = secs_part + duration.subsec_nanos() as u64;

    if duration_in_ns < 1_000 {
        format!("{}ns", duration_in_ns)
    } else if duration_in_ns < 1_000_000 {
        format!("{:.1}us", duration_in_ns as f64 / 1_000.0)
    } else if duration_in_ns < 1_000_000_000 {
        format!("{:.1}ms", duration_in_ns as f64 / 1_000_000.0)
    } else {
        format!("{:.1}s", duration_in_ns as f64 / 1_000_000_000.0)
    }
}
