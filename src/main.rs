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
#[macro_use]
extern crate mysql;
extern crate rustc_serialize;

use dotenv::dotenv;
use rouille::Request;
use rouille::Response;

use std::env;
use std::io::{self, Read, Write};
use std::panic;
use std::time::{Duration, Instant};

mod world;

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
        let k = k.replace("_", "-");
        match &*k {
            "CONTENT-TYPE" => return Some((k.into(), v)),
            "CONTENT-LENGTH" => return Some((k.into(), v)),
            _ => {}
        }
        match k.split("HTTP-").nth(1) {
            Some(k) => Some((k.into(), v)),
            None => None,
        }
    })
    .collect::<Vec<_>>()
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
    let mut request = envy::from_env::<EnvRequest>()?;
    //println!("{:?}", request);

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let pool = mysql::Pool::new(&*database_url)?;
    pool.prep_exec(r"CREATE TABLE IF NOT EXISTS entities (
            id INTEGER AUTO_INCREMENT PRIMARY KEY,
            name VARCHAR(50),
            health INTEGER NOT NULL
        )", ())?;
    pool.prep_exec(r"CREATE TABLE IF NOT EXISTS notes (
            id INTEGER AUTO_INCREMENT PRIMARY KEY,
            content TEXT NOT NULL
        )", ())?;

    // Read request body from stdin
    let mut data = Vec::new();
    io::stdin().read_to_end(&mut data)?;

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
        (GET) (/) => {
            use std::fs::File;
            
            if let Ok(file) = File::open("assets/index.html") {
                Response::from_file("text/html", file)
            } else {
                Response::empty_404()
            }
        },
        (GET) (/hello) => {
            Response::html("<p>Hello world</p>")
        },
        (GET) (/entity/{id: i32}) => {
            let selected_entities: String = pool.prep_exec(
                "SELECT id, health, name FROM entities WHERE id = :id",
                params!{ "id" => id })
                .map(|result| {
                    result.filter_map(Result::ok)
                        .map(|row| {
                            let (id, health, name): (i32, i32, String)
                                                     = mysql::from_row(row);
                //        Entity {
                //            id: id,
                //            health: health,
                //            name: name,
                //        }
                            format!(r"<ul>
                            <li>{}</li>
                            <li>{}</li>
                            <li>{}</li>
                            </ul>
                            ", id, name, health)
                        }).collect()
                    }).unwrap();

            Response::html(selected_entities)
        },

        (GET) (/notes) => {
            // This route returns the list of notes. We perform the query and output it as JSON.

            #[derive(RustcEncodable)]
            struct Elem { id: String }

            let out = pool.prep_exec("SELECT id FROM notes", ())
                .map(|result| {
                    result.filter_map(Result::ok)
                        .map(|row| {
                            let id: i32 = mysql::from_row(row);
                            Elem { id: format!("/note/{}", id) }
                        }).collect::<Vec<Elem>>()
                    });

            match out {
                Ok(o) => Response::json(&o),
                Err(_) => Response::text("error"),
            }
        },

        (GET) (/note/{id: i32}) => {
            // This route returns the content of a note, if it exists.
            let content = pool.prep_exec(
                "SELECT content FROM notes WHERE id = :id",
                params!{ "id" => id })
                .map(|result| {
                    result.filter_map(Result::ok)
                        .map(|row| {
                            let c: String = mysql::from_row(row);
                            c
                        }).collect::<String>()
                    });

            match content {
                Ok(content) => Response::text(content),
                Err(_) => Response::empty_404(),
            }
        },

        (PUT) (/note/{id: i32}) => {
            // This route modifies the content of an existing note.

            // We start by reading the body of the HTTP request into a `String`.
            if let Ok(body) = rouille::input::plain_text_body(&request) {
                let update_count = pool.prep_exec(
                    "UPDATE notes SET content = :content WHERE id = :id",
                    params!{ "id" => id, "content" => body })
                    .map(|result| {
                        result.filter_map(Result::ok)
                            .map(|row| {
                                let c: u64 = mysql::from_row(row);
                                c
                            }).collect::<Vec<u64>>()
                        });


                // We determine whether the note exists thanks to the number of rows that
                // were modified.
                match update_count {
                    Ok(c) => {
                        if c[0] > 0 {
                            Response::text("The note has been updated")
                        } else {
                            Response::empty_404()
                        }
                    }
                    Err(_) => Response::empty_400(),
                }
            } else {
                Response::empty_400()
            }
        },

        (POST) (/note) => {
            // This route creates a new node whose initial content is the body.

            // We start by reading the body of the HTTP request into a `String`.
            let body = post_input!(&request, {
                content: String,
            });
            
            match body {
                Ok(body) => {
                    pool.prep_exec(
                        "INSERT INTO notes (content) VALUES (:content)",
                        params!{ "content" => format!("{:?}", body.content) }).unwrap();
                    let id = pool.prep_exec("SELECT MAX(id) FROM notes", ())
                        .map(|result| {
                            result.filter_map(Result::ok)
                                .map(|row| {
                                    let id: i32 = mysql::from_row(row);
                                    id
                                }).collect::<Vec<i32>>()
                            });

                    // We determine whether the note exists thanks to the number of rows that
                    // were modified.
                    match id {
                        Ok(id) => {
                            let mut response = Response::text("The note has been created");
                            response.status_code = 201;
                            response.headers.push(
                                ("Location".into(),
                                format!("/note/{}", id[0]).into()));
                            response
                        }
                        Err(e) => Response::html(
                            format!("<h1>400 Not Found</h1><p>{}</p>", e)),
                    }
                }
                Err(e) => Response::html(
                    format!("<h1>400 Not Found</h1><p>{}</p>", e)),
            }
        },

        (DELETE) (/note/{id: i32}) => {
            // This route deletes a note. This line can only panic if the
            // SQL is malformed.
            match pool.prep_exec("DELETE FROM notes WHERE id = :id", params!{ "id" => id }) {
                Ok(_) => Response::text(""),
                Err(_) => Response::empty_400(),
            }
        },

        // If none of the other blocks matches the request, return a 404 response.
        _ => Response::empty_404()
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
            writeln!(output, "Status: {}", response.status_code)?;
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
