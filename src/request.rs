use anyhow::{Result, bail};
use log::{debug, error};

use std::io::{BufRead, BufReader};
use std::net::TcpStream;

type Key = String;
type Value = String;

#[derive(Debug)]
pub(crate) struct Request {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) http_version: String,
    pub(crate) headers: Vec<(Key, Value)>,
    pub(crate) body: Option<String>,
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

        //For simplicity, we are not handling the body in this example
        Ok(Request {
            method,
            path,
            http_version,
            headers,
            body: None,
        })
    }
}
