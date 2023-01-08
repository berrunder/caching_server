use std::io;
use std::io::BufRead;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::net::{TcpStream};

use crate::db::MemoryCache;

pub fn parse_request(mut stream: TcpStream) -> Result<Request, io::Error> {
    let mut buffer = Vec::new();
    let mut method = String::new();
    let mut url = String::new();
    
    loop {
        let mut chunk = [0; 1024];
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..n]);
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);
        let status = req.parse(&buffer).expect("Wrong request format");
        if status.is_complete() {
            method = req.method.expect("No method found in request").to_string();
            url = req.path.expect("Path not found in request").to_string();
            break;
        }
    }

    Ok(Request {
        method,
        url,
        raw: buffer,
    })
}

fn process_request(request: Request, socket_addr: SocketAddr) -> Vec<u8> {
    // Connect to the third-party server
    let mut stream = TcpStream::connect(socket_addr).unwrap();

    // Send the request to the third-party server
    stream.write_all(&request.raw).unwrap();

    let mut reader = io::BufReader::new(&mut stream);
    // Read the response from the third-party server
    let response = reader.fill_buf().unwrap().to_vec();
    reader.consume(response.len());

    return response;
}

pub fn handle_request(database: &mut impl MemoryCache<Vec<u8>>, request: Request, socket_addr: SocketAddr) -> Vec<u8> {
    let key = format!("{} {}", request.method, request.url);

    if request.method != "GET" {
        return process_request(request, socket_addr);
    }
    // Check the cache to see if we have already processed this request
    if let Some(cached_response) = database.get(&key) {
        // Return the cached response if we have it
        return cached_response;
    }

    // Process the request and generate a response
    let response = process_request(request, socket_addr);

    // Cache the response for future requests
    database.set(key, response.clone());

    response
}

#[derive(Debug)]
pub struct Request {
method: String,
    url: String,
    raw: Vec<u8>,
}