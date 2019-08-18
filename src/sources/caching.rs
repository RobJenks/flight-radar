use std::collections::HashMap;
use std::io::ErrorKind;

const DEFAULT_LOCATION: &str = "cache";

pub struct FilesystemCache<V, F>
    where V: Clone,
          F: Fn(&String) -> V
{
    location: String,
    transform: F,
    data: HashMap<String, V>
}

impl<V, F> FilesystemCache<V, F>
    where V: Clone,
          F: Fn(&String) -> V
{
    pub fn new_from_location(location: &str, transform: F) -> Self {
        Self { location: String::from(location), transform, data: HashMap::new() }
    }

    pub fn new(transform: F) -> Self {
        Self::new_from_location(DEFAULT_LOCATION, transform)
    }

    pub fn get(&mut self, key: &String) -> std::io::Result<V> {
        self.data.get(key)
            .map(|x| Ok(x.clone()))                              // Either return locally-cached version
            .unwrap_or_else(|| {                                         // Or attempt to retrieve from filesystem
                std::fs::read_to_string(&self.cache_path(key))
                    .map(|x| (self.transform)(&x))
                    .and_then(|x| {
                        self.data.insert(key.clone(), x.clone());     // If we could read from disk, store locally
                        Ok(x)
                    })
                    .or_else(|e| Result::Err(e))
            })
    }

    fn cache_path(&self, key: &String) -> String {
        format!("{}/{}", self.location, key)
    }

    // Returns a normalised name suitable for use as key in the filesystem cache
    pub fn cache_key(&self, input_str: &str) -> String {
        input_str.chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect::<String>()
    }
}