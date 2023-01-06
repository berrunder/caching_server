use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};

use crate::db::MemoryCache;

pub fn parse_request(stream: TcpStream) -> Result<Request, io::Error> {
    let mut headers: HashMap<String, String> = HashMap::new();
    let mut body = String::new();

    let mut is_body = false;
    let request_str = parse_request_str(stream)?;
    for line in request_str.lines() {
        if line.is_empty() {
            is_body = true;
            continue;
        }

        if is_body {
            body.push_str(line);
        } else {
            let parts: Vec<&str> = line.split(": ").collect();
            if parts.len() < 2 {
                continue;
            }
            let key = parts[0];
            let value = parts[1];
            headers.insert(key.to_string(), value.to_string());
        }
    }

    let request_line = request_str.lines().next().unwrap();

    let request_parts: Vec<&str> = request_line.split(" ").collect();
    let method = request_parts[0].to_string();
    let url = request_parts[1].to_string();

    Ok(Request {
        method,
        url,
        headers,
        body,
    })
}

fn parse_request_str(mut stream: TcpStream) -> Result<String, io::Error> {
    let mut buffer = Vec::new();
    
    loop {
        let mut chunk = [0; 1024];
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..n]);
        let temp_buffer = buffer.clone();
        if is_complete(&temp_buffer) {
            break;
        }
    }

    let request_str = String::from_utf8(buffer).unwrap();
    Ok(request_str)
} 

fn is_complete(buffer: &Vec<u8>) -> bool {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    let status = req.parse(buffer).expect("Wrong request format");
    status.is_complete()
}

fn process_request(request: Request) -> String {
    // Read the third-party server address and port from the config file
    let addr = fs::read_to_string("config.txt").expect(
        "config.txt with backend server address and port (format 127.0.0.1:3000) must be present",
    );
    let socket_addr = addr.trim().to_socket_addrs().unwrap().next().unwrap();

    // Connect to the third-party server
    let mut stream = TcpStream::connect(socket_addr).unwrap();

    // Send the request to the third-party server
    let request_str = request.to_string();
    stream.write_all(request_str.as_bytes()).unwrap();

    // Read the response from the third-party server
    let mut response_str = String::new();
    stream.read_to_string(&mut response_str).unwrap();

    return response_str;
}

pub fn handle_request(database: &mut impl MemoryCache, request: Request) -> String {
    let key = format!("{} {}", request.method, request.url);

    // Check the cache to see if we have already processed this request
    if let Some(cached_response) = database.get(&key) {
        // Return the cached response if we have it
        return cached_response;
    }

    // Process the request and generate a response
    let response = process_request(request);

    // Cache the response for future requests
    database.set(key, response.clone());

    response
}

#[derive(Debug)]
pub struct Request {
    method: String,
    headers: HashMap<String, String>,
    url: String,
    body: String,
}

impl Request {
    pub fn to_string(&self) -> String {
        let mut request_str = String::new();

        // Add the request line
        request_str.push_str(&self.method);
        request_str.push_str(" ");
        request_str.push_str(&self.url);
        request_str.push_str("\r\n");

        // Add the headers
        for (key, value) in &self.headers {
            request_str.push_str(key);
            request_str.push_str(": ");
            request_str.push_str(value);
            request_str.push_str("\r\n");
        }

        // Add the body
        request_str.push_str("\r\n");
        request_str.push_str(&self.body);

        request_str
    }
}
