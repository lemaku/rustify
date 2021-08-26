mod http;
mod logger;

use std::{
    error::Error,
    io::{self, Read, Write},
    net::TcpListener,
};

#[macro_use]
extern crate simple_error;

// Use default port 80. Can be overrode by command line argument
const DEFAULT_PORT: i32 = 80;

/// Start server on localhost
const DEFAULT_HOST: &str = "127.0.0.1";
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let port: i32 = args
        .first()
        .unwrap_or(&"".to_string())
        .parse()
        .unwrap_or(DEFAULT_PORT);

    log!("Starting web server on port {}...", port);

    let listener = TcpListener::bind(format!("{}:{}", DEFAULT_HOST, port))
        .unwrap_or_else(|_| panic!("Could not bind {}:{}", DEFAULT_HOST, port));

    log!("Ready to accept connections...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                log!("Handling request from {:?}", stream.peer_addr().unwrap());
                wrap_error(stream).unwrap();
            }
            Err(ref e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    continue;
                }
                panic!("Ran into IO Error: {}", e)
            }
        }
    }
}

fn wrap_error(stream: std::net::TcpStream) -> Result<(), Box<dyn Error>> {
    if handle_request(&stream).is_err() {
        let mut response = http::Response {
            status: 0,
            status_message: String::new(),
            headers: vec![],
            body: String::new(),
        };
        response.body = String::from("An error occured on the server during the request.");
        response.status = 500;
        response.status_message = String::from("Internal Server Error");
        response.headers.push(http::Header {
            key: String::from("Content-Type"),
            value: String::from("text/plain"),
        });
        response.headers.push(http::Header {
            key: String::from("Content-Length"),
            value: response.body.as_bytes().len().to_string(),
        });
        write_response(&stream, response)?;
    }
    Ok(())
}

#[allow(clippy::unused_io_amount)]
fn handle_request(mut stream: &std::net::TcpStream) -> Result<(), Box<dyn Error>> {
    // Read buffer from stream
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;

    // Parse request from buffer
    let request = parse_request_from_buffer(buffer)?;
    log!("HTTP {} {}", request.method, request.path);

    match request.method.as_str() {
        "GET" => {
            let path = try_find_path(request.path);
            let mut response = http::Response {
                status: 0,
                status_message: String::new(),
                headers: vec![],
                body: String::new(),
            };
            match path {
                Some(path) => {
                    let file_content = read_file(path.as_str());
                    if let Ok(file_content) = file_content {
                        response.status = 200;
                        response.status_message = String::from("OK");
                        response.body = file_content;
                        response.headers.push(http::Header {
                            key: String::from("Content-Type"),
                            value: get_content_type(path.as_str()),
                        });
                        response.headers.push(http::Header {
                            key: String::from("Content-Length"),
                            value: response.body.as_bytes().len().to_string(),
                        });
                    }
                }
                None => {
                    response.body = String::from("Could not find resource.");
                    response.status = 404;
                    response.status_message = String::from("NOT FOUND");
                    response.headers.push(http::Header {
                        key: String::from("Content-Type"),
                        value: String::from("text/plain"),
                    });
                    response.headers.push(http::Header {
                        key: String::from("Content-Length"),
                        value: response.body.as_bytes().len().to_string(),
                    });
                }
            };
            write_response(&stream, response)?;
        }
        _ => bail!(format!("HTTP method {} is not supported", request.method)),
    };
    Ok(())
}

/// Provides just some basic MIME types to serve a simple angular app
fn get_content_type(path: &str) -> String {
    String::from(match path.split('.').last().unwrap_or("") {
        "html" => "text/html;charset=UTF-8",
        "js" => "application/javascript",
        "json" => "application/json",
        "ico" => "image/x-icon",
        "css" => "text/css",
        _ => "text/plain",
    })
}

/// Writes response to client
fn write_response(
    mut stream: &std::net::TcpStream,
    response: http::Response,
) -> Result<(), Box<dyn Error>> {
    let mut response_string: String =
        format!("HTTP/1.1 {} {}\n", response.status, response.status_message);
    for header in response.headers {
        response_string.push_str(format!("{}: {}\n", header.key, header.value).as_str());
    }
    response_string.push_str(format!("\n{}", response.body).as_str());
    stream.write_all(response_string.as_bytes())?;
    Ok(())
}

/// Read file from file system
fn read_file(path: &str) -> Result<String, Box<dyn Error>> {
    let file = std::fs::read_to_string(path)?;
    Ok(file)
}

/// Check if given path correlates to an existing file
fn try_find_path(mut path: String) -> Option<String> {
    if path.ends_with('/') {
        path.push_str("index.html");
    }
    dotenv::dotenv().ok();
    let mut absolute_path: String =
        std::env::var("APP_FOLDER").expect("Please set environment variable APP_FOLDER");
    absolute_path.push_str(path.as_str());
    if std::path::Path::new(&absolute_path).exists() {
        return Some(absolute_path);
    }
    None
}

/// Parse buffer to Request struct
fn parse_request_from_buffer(buffer: [u8; 1024]) -> Result<http::Request, Box<dyn Error>> {
    let mut lines = std::str::from_utf8(&buffer)?.lines();

    let mut request = http::Request {
        method: String::new(),
        path: String::new(),
        headers: vec![],
    };

    // Read http method and path from request
    let mut first_line = lines.next().ok_or("wat ze fak")?.split_ascii_whitespace();
    request.method = first_line.next().ok_or("wat ze fak")?.into();
    request.path = first_line.next().ok_or("wat ze fak")?.into();

    // Read headers
    for line in lines {
        if let Some(arr) = line.split_once(":") {
            request.headers.push(http::Header {
                key: arr.0.into(),
                value: arr.1.into(),
            });
        }
    }

    Ok(request)
}
