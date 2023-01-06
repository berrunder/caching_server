use std::collections::{HashMap, VecDeque};

pub trait MemoryCache {
    fn set(&mut self, url: String, response: String);
    fn get(&mut self, url: &str) -> Option<String>;
}

pub struct LRUMemoryCache {
    data: HashMap<String, String>,

    // Keeps track of the order in which the URLs were accessed, with the most recently accessed URL at the front of the list
    access_order: VecDeque<String>,

    // Maximum number of items that can be stored in the database
    max_size: usize,
}

impl LRUMemoryCache {
    pub fn new(max_size: usize) -> Self {
        LRUMemoryCache {
            data: HashMap::new(),
            access_order: VecDeque::new(),
            max_size,
        }
    }
}

impl MemoryCache for LRUMemoryCache {
    fn set(&mut self, url: String, response: String) {
        // If the item already exists in the database, remove it from the access order list
        if self.data.contains_key(&url) {
            self.access_order.retain(|x| x != &url);
        }

        // If the database is at capacity, remove the least recently used item
        if self.access_order.len() == self.max_size {
            let url_to_remove = self.access_order.pop_back().unwrap();
            self.data.remove(&url_to_remove);
        }

        // Insert the new item at the front of the access order list and store it in the database
        self.access_order.push_front(url.clone());
        self.data.insert(url, response);
    }

    fn get(&mut self, url: &str) -> Option<String> {
        // If the item exists in the database, move it to the front of the access order list
        if self.data.contains_key(url) {
            self.access_order.retain(|x| x != url);
            self.access_order.push_front(url.to_string());
        }

        // Return the item from the database, or None if it doesn't exist
        self.data.get(url).cloned()
    }
}
