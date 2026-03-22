use anyhow::{Result, bail};
use log::{debug, error, info};

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

const SUCCESS_RESPONSE: &str = "HTTP/1.1 200 OK\r\n\r\n";
const ERROR_RESPONSE: &str = "HTTP/1.1 400 NOT FOUND\r\n\r\n";

type Key = String;
type Value = String;

#[derive(Debug)]
struct Request {
    method: String,
    path: String,
    http_version: String,
    headers: Vec<(Key, Value)>,
    body: Option<String>,
}

impl TryFrom<TcpStream> for Request {
    type Error = anyhow::Error;
    fn try_from(stream: TcpStream) -> Result<Self> {
        let mut reader = BufReader::new(stream);
        let mut request_line = String::new();
        reader.read_line(&mut request_line)?;

        //split the first line by space
        //secont totken is the path
        //Example:
        //GET /index.html HTTP/1.1
        let request_line: Vec<&str> = request_line.split_whitespace().collect();

        //we expect three fields in the request line:
        //1.Method(ex.g GET/POST)
        //2. Path (e.g /index.html)
        //3. HTTP version (e.g HTTP/1.1)

        let (method, path, http_version) = match request_line[..] {
            [method, path, http_version] => {
                debug!("Correct format found: {}", request_line.join(" "));
                (
                    method.to_string(),
                    path.to_string(),
                    http_version.to_string(),
                )
            }
            _ => {
                bail!("invalid request line: {}", request_line.join(" "));
            }
        };

        let mut headers = Vec::new();
        loop {
            let mut header_line = String::new();
            let bytes_read = reader.read_line(&mut header_line)?;
            if bytes_read == 0 || header_line == "\r\n" {
                break;
            }
            if let Some((key, value)) = header_line.split_once(":") {
                headers.push((key.to_string(), value.to_string()));
            } else {
                error!("Invalid header line: {header_line}");
            }
        }

        //For simplicity, we are not handling the bpdy in this example
        Ok(Request {
            method,
            path,
            http_version,
            headers,
            body: None,
        })
    }
}

fn handle_request(mut stream: TcpStream) -> Result<()> {
    debug!("accepted new connection");

    let request = Request::try_from(stream.try_clone()?)?;

    let response = if request.path == "/" {
        debug!("root path requested");
        SUCCESS_RESPONSE.to_string()
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
                ERROR_RESPONSE.to_string()
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
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    let mut response_handles = Vec::new();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let handle = std::thread::spawn(|| handle_request(stream));
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
