mod db;
mod request;

use clap::{Arg, Command};
use db::LFUMemoryCache;
use db::{LRUMemoryCache, MemoryCache};
use request::{handle_request, parse_request};
use std::env;
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
    let matches = Command::new("Caching server")
        .version("0.1.0")
        .author("Alexander Cherkashin <alexander.cherkashin@csssr.com>")
        .about("Caches incoming get requests")
        .arg(
            Arg::new("algo")
                .short('a')
                .long("algo")
                .value_parser(["LFU", "LRU"])
                .default_value("LRU")
                .help("Caching algorithm to use, can be LFU or LRU"),
        )
        .get_matches();

    let algo = matches.get_one::<String>("algo").unwrap();
    let size: usize = env::var("MAX_CACHE_SIZE")
        .unwrap_or(MAX_CACHE_SIZE.to_string())
        .parse()
        .unwrap_or(MAX_CACHE_SIZE);
    let n_threads: usize = env::var("N_THREADS")
        .unwrap_or(N_THREADS.to_string())
        .parse()
        .unwrap_or(N_THREADS);

    // Create a listener to accept incoming connections
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    // Read the third-party server address and port from the config file
    let back_addr = fs::read_to_string("config.txt").expect(
        "config.txt with backend server address and port (format 127.0.0.1:3000) must be present",
    );
    let socket_addr = back_addr.trim().to_socket_addrs().unwrap().next().unwrap();
    let pool = ThreadPool::new(n_threads);
    let db_instance = get_database_instance(&algo, size);
    let database = Arc::new(Mutex::new(db_instance));

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
                    let db = database.lock().unwrap();
                    // Generate a response for the request
                    let response = handle_request(db, request, socket_addr);

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

fn get_database_instance(algo: &str, size: usize) -> Box<dyn MemoryCache<Vec<u8>> + Send> {
    match algo {
        "LFU" => Box::new(LFUMemoryCache::new(size)),
        "LRU" => Box::new(LRUMemoryCache::new(size)),
        _ => Box::new(LRUMemoryCache::new(size)),
    }
}