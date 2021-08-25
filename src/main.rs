mod logger;

use std::{
    error::Error,
    io::{self, Write},
    net::TcpListener,
};

const DEFAULT_PORT: i32 = 80;
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

    listener
        .set_nonblocking(true)
        .expect("Could not set non-blocking");

    log!("Ready to accept connections...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                log!("Handling request from {:?}", stream.peer_addr().unwrap());
                handle_request(stream).expect("An error occured at a client request");
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

fn handle_request(mut stream: std::net::TcpStream) -> Result<(), Box<dyn Error>> {
    stream.write_all(b"A message")?;
    Ok(())
}
