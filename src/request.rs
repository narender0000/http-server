use anyhow::{Result, bail};
use log::{debug, error};

use std::io::{BufRead, BufReader};
use std::net::TcpStream;

type Key = String;
type Value = String;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RequestMethod {
    Get,
    Post,
}

impl TryFrom<String> for RequestMethod {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self> {
        match value.to_lowercase().as_str() {
            "get" => Ok(RequestMethod::Get),
            "post" => Ok(RequestMethod::Post),
            _ => bail!("Unsupported HTTP method: {value}"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Request {
    pub(crate) method: RequestMethod,
    pub(crate) path: String,
    pub(crate) http_version: String,
    pub(crate) headers: Vec<(Key, Value)>,
    pub(crate) body: Option<Vec<u8>>,
}

impl Request {
    pub(crate) fn get_header(&self, header_key: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k == header_key)
            .map(|(_, v)| v.as_str())
    }
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
                    RequestMethod::try_from(method.to_string())?,
                    path.to_string(),
                    http_version.to_string(),
                )
            }
            _ => {
                bail!("invalid request line: {}", request_line.join(" "));
            }
        };

        let headers = Headers::parse(&mut reader)?;

        let mut expected_bytes = 0;
        for (key, value) in &headers {
            if key.to_string() == "Content-Length" {
                expected_bytes = value.parse()?;
                break;
            }
        }

        let body = if expected_bytes == 0 {
            debug!("No Content-Length header found, assuming no body");
            None
        } else {
            debug!("Reading requets body...");
            let mut body = vec![0; expected_bytes];
            std::io::Read::read_exact(&mut reader, &mut body)?;

            debug!("Parsed body: {body:?}");
            Some(body)
        };

        Ok(Request {
            method,
            path,
            http_version,
            headers,
            body,
        })
    }
}

struct Headers;

impl Headers {
    fn parse(mut reader: impl BufRead) -> Result<Vec<(String, String)>, anyhow::Error> {
        let mut headers = Vec::new();
        loop {
            let mut header_line = String::new();
            let bytes_read = reader.read_line(&mut header_line)?;
            if bytes_read == 0 || header_line == "\r\n" {
                break;
            }
            if let Some((key, value)) = header_line.split_once(":") {
                headers.push((key.trim().to_string(), value.trim().to_string()));
            } else {
                error!("Invalid header line: {header_line}");
            }
        }
        debug!("Parsed headers: {headers:?}");
        Ok(headers)
    }
}
