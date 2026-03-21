use anyhow::Result;
use log::{debug, error, info};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

const SUCCESS_RESPONSE: &[u8] = "HTTP/1.1 200 OK\r\n\r\n".as_bytes();
const ERROR_RESPONSE: &[u8] = "HTTP/1.1 400 NOT FOUND\r\n\r\n".as_bytes();
fn main() -> Result<()> {
    env_logger::init();
    info!("Server started");
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                debug!("accepted new connection");

                let mut request_buffer = BufReader::new(&stream);
                let mut request_line = String::new();

                request_buffer.read_line(&mut request_line)?;

                //split the first line by space
                //secont totken is the path
                //Example:
                //GET /index.html HTTP/1.1
                let path: Vec<&str> = request_line.split_whitespace().collect();

                match path[..] {
                    ["GET", path, "HTTP/1.1"] => {
                        if path == "/" {
                            debug!("root path requested");
                            stream.write(SUCCESS_RESPONSE)?;
                            stream.flush()?;
                        } else if path.starts_with("/echo/") {
                            let echo_path = path.split_once("/echo/");
                            match echo_path {
                                Some((_, path)) => {
                                    debug!("echo path requested: {path}");
                                    let response = format!(
                                        "HTTP1.1 200 OK\r\nContent-Type: text/plain\r\nContent=length: {}\r\n\r\n{}",
                                        path.len(),
                                        path
                                    );
                                    stream.write(response.as_bytes())?;
                                    stream.flush()?;
                                }

                                _ => {
                                    error!("Invalid echo path in request. Request: {request_line}");
                                }
                            }
                        } else {
                            debug!("Unkown path: {path}");
                            stream.write(ERROR_RESPONSE)?;
                            stream.flush()?;
                        }
                    }
                    _ => {
                        error!("No Path in request, Input: {request_line}");

                        stream.write(ERROR_RESPONSE)?;
                        stream.flush()?;
                    }
                }
            }
            Err(e) => {
                error!("error: {e}");
            }
        }
    }
    Ok(())
}
