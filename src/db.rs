use std::collections::{HashMap, VecDeque};

pub trait MemoryCache<V: std::clone::Clone> {
    fn set(&mut self, key: String, value: V);
    fn get(&mut self, key: &str) -> Option<V>;
}

pub struct LRUMemoryCache<V> {
    data: HashMap<String, V>,

    // Keeps track of the order in which the URLs were accessed, with the most recently accessed URL at the front of the list
    access_order: VecDeque<String>,

    // Maximum number of items that can be stored in the database
    max_size: usize,
}

impl<V> LRUMemoryCache<V> {
    pub fn new(max_size: usize) -> Self {
        LRUMemoryCache {
            data: HashMap::new(),
            access_order: VecDeque::new(),
            max_size,
        }
    }
}

impl<V: std::clone::Clone> MemoryCache<V> for LRUMemoryCache<V> {
    fn set(&mut self, key: String, value: V) {
        // If the item already exists in the database, remove it from the access order list
        if self.data.contains_key(&key) {
            self.access_order.retain(|x| x != &key);
        }

        // If the database is at capacity, remove the least recently used item
        if self.access_order.len() == self.max_size {
            let key_to_remove = self.access_order.pop_back().unwrap();
            self.data.remove(&key_to_remove);
        }

        // Insert the new item at the front of the access order list and store it in the database
        self.access_order.push_front(key.clone());
        self.data.insert(key, value);
    }

    fn get(&mut self, key: &str) -> Option<V> {
        // If the item exists in the database, move it to the front of the access order list
        if self.data.contains_key(key) {
            self.access_order.retain(|x| *x != key);
            self.access_order.push_front(key.to_string());
        }

        // Return the item from the database, or None if it doesn't exist
        self.data.get(key).cloned()
    }
}

pub struct LFUMemoryCache<V> {
    data: HashMap<String, V>,
    frequency: HashMap<String, u32>,
    max_size: usize,
}

impl<V> LFUMemoryCache<V> {
    pub fn new(max_size: usize) -> Self {
        Self {
            data: HashMap::new(),
            frequency: HashMap::new(),
            max_size,
        }
    }

    fn insert(&mut self, key: String, value: V) {
        self.data.insert(key.clone(), value);
        self.frequency.insert(key, 1);
    }

    fn remove_lfu(&mut self) {
        let lfu_key = self.get_lfu_key();

        lfu_key.map(|key| {
            self.data.remove(&key);
            self.frequency.remove(&key);
        });
    }

    fn get_lfu_key(&self) -> Option<String> {
        let mut lfu_key = None;
        let mut min_frequency = u32::MAX;

        for key in self.data.keys() {
            let count = self.frequency.get(key.as_str()).unwrap();

            if *count < min_frequency {
                lfu_key = Some(key);
                min_frequency = *count;
            }
        }

        lfu_key.map(|key| key.to_string())
        
    }
}

impl<V: std::clone::Clone> MemoryCache<V> for LFUMemoryCache<V> {
    fn set(&mut self, key: String, value: V) {
        if self.data.len() == self.max_size {
            self.remove_lfu();
        }

        self.insert(key, value);
    }

    fn get(&mut self, key: &str) -> Option<V> {
        if let Some(value) = self.data.get(key) {
            let count = self.frequency.get(key).unwrap();
            self.frequency.insert(key.to_string(), count + 1);
            Some(value.clone())
        } else {
            None
        }
    }
}