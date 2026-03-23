use anyhow::Result;
use log::{debug, error, info};

use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;

mod request;

use request::Request;

fn handle_request(file_directory: PathBuf, mut stream: TcpStream) -> Result<()> {
    debug!("accepted new connection");

    let request = Request::try_from(stream.try_clone()?)?;

    let response = if request.path == "/" {
        debug!("root path requested");
        "HTTP/1.1 200 OK\r\n\r\n".to_string()
    } else if request.path == "/user-agent/" {
        let mut user_agent = None;
        for (key, value) in &request.headers {
            if key == "User-Agent" {
                user_agent = Some(value);
                break;
            }
        }
        match user_agent {
            None => {
                error!("User-Agent header not found in request. Request: {request:?}");
                "HTTP/1.1 400 NOT FOUND\r\n\r\n".to_string()
            }
            Some(user_agent) => {
                let response = format!(
                    "HTTP1.1 200 OK\r\nContent-Type: text/plain\r\nContent=length: {}\r\n\r\n{}",
                    user_agent.len(),
                    user_agent
                );
                response
            }
        }
    } else if request.path.starts_with("/echo/") {
        let echo_path = request.path.split_once("/echo/");
        match echo_path {
            Some((_, path)) => {
                debug!("echo path requested: {path}");
                let response = format!(
                    "HTTP1.1 200 OK\r\nContent-Type: text/plain\r\nContent=length: {}\r\n\r\n{}",
                    path.len(),
                    path
                );
                response
            }

            _ => {
                error!("Invalid echo path in request. Request: {request:?}");
                format!("HTTP/1.1 400 Bad request\r\n\r\n")
            }
        }
    } else if request.path.starts_with("/files/") {
        let file_path = request.path.split_once("/files");
        match file_path {
            Some((_, path)) => {
                debug!("file path requested: {path}");

                match std::fs::read_to_string(file_directory.join(path)) {
                    Ok(body) => {
                        format!(
                            "HTTP1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent=length: {}\r\n\r\n{}",
                            body.len(),
                            body
                        )
                    }
                    Err(_e) => {
                        format!("HTTP/1.1 400 Not Found\r\n\r\n")
                    }
                }
            }

            _ => {
                error!("Invalid echo path in request. Request: {request:?}");
                format!("HTTP/1.1 400 Bad request\r\n\r\n")
            }
        }
    } else {
        debug!("nnkown path requested: `{}`", request.path);
        format!("HTTP/1.1 400 Not Found\r\n\r\n")
    };

    stream.write(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}
fn main() -> Result<()> {
    env_logger::init();
    info!("Server started");

    // TODO: make args parsing more flexible
    // Right now, this expects an invoation like the following:
    // ./servr --directory /tmp/
    let file_directory = std::env::args().nth(2).unwrap_or("/".to_string());
    let file_directory = PathBuf::from(file_directory);
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    let mut response_handles = Vec::new();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let file_directory = file_directory.clone();
                let handle = std::thread::spawn(|| handle_request(file_directory, stream));
                response_handles.push(handle);
            }
            Err(e) => {
                error!("error: {e}");
            }
        }
    }
    for handle in response_handles {
        handle.join().unwrap()?;
    }
    Ok(())
}
