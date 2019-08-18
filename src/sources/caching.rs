use std::collections::HashMap;

const DEFAULT_LOCATION: &str = "cache";

pub struct FilesystemCache {
    location: String,
    data: HashMap<String, String>
}

impl FilesystemCache {
    pub fn new_from_location(location: &str) -> Self {
        Self { location: String::from(location), data: HashMap::new() }
    }

    pub fn new() -> Self {
        Self::new_from_location(DEFAULT_LOCATION)
    }

    pub fn get(&mut self, key: &String) -> std::io::Result<String> {
        self.data.get(key)
            .map(|x| Ok(x.clone()))                              // Either return locally-cached version
            .unwrap_or_else(|| {                                         // Or attempt to retrieve from filesystem
                std::fs::read_to_string(&self.cache_path(key))
                    .and_then(|x| {
                        self.data.insert(key.clone(), x.clone());     // If we could read from disk, store locally
                        Ok(x)
                    })
            })
    }

    fn cache_path(&self, key: &String) -> String {
        format!("{}/{}", self.location, key)
    }

    // Returns a normalised name suitable for use as key in the filesystem cache
    pub fn cache_key(input_str: &str) -> String {
        input_str.chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect::<String>()
    }
}