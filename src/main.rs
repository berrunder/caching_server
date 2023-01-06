mod db;
mod request;

use db::LRUMemoryCache;
use request::{handle_request, parse_request};
use std::fs;
use std::io::Write;
use std::net::{TcpListener, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;

// Constant for maximum cache size
const MAX_CACHE_SIZE: usize = 1024;
// Constant for number of woring threads
const N_THREADS: usize = 4;

fn main() {
    // Create a listener to accept incoming connections
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    // Read the third-party server address and port from the config file
    let back_addr = fs::read_to_string("config.txt").expect(
        "config.txt with backend server address and port (format 127.0.0.1:3000) must be present",
    );
    let socket_addr = back_addr.trim().to_socket_addrs().unwrap().next().unwrap();

    // Create an in-memory database to store cached responses
    let database = Arc::new(Mutex::new(LRUMemoryCache::new(MAX_CACHE_SIZE)));
    let pool = ThreadPool::new(N_THREADS);

    println!("Starting server at port 8080...");

    // Accept incoming connections and handle them in separate threads
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let database = Arc::clone(&database);

                // Parse the request from the stream
                let request =
                    parse_request(stream.try_clone().unwrap()).expect("Unknown request format");

                // Handle the request in a separate thread
                pool.execute(move || {
                    let mut db = database.lock().unwrap();
                    // Generate a response for the request
                    let response = handle_request(&mut *db, request, socket_addr);

                    // Send the response back to the client
                    stream.write_all(&response).unwrap();
                });
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}
