mod db;
mod request;

use db::LRUMemoryCache;
use request::{handle_request, parse_request};
use std::io::Write;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;

// Constant for maximum cache size
const MAX_CACHE_SIZE: usize = 1024;
// Constant for number of woring threads
const N_THREADS: usize = 4;

fn main() {
    // Create a listener to accept incoming connections
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

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
                    let response = handle_request(&mut *db, request);

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
