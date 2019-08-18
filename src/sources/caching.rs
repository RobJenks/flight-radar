const DEFAULT_LOCATION: &str = "cache";

pub struct FilesystemCache {
    location: String,
}

impl FilesystemCache {
    pub fn new_from_location(location: &str) -> Self {
        Self { location: String::from(location) }
    }

    pub fn new() -> Self {
        Self::new_from_location(DEFAULT_LOCATION)
    }

    pub fn get(&self, key: &String) -> std::io::Result<String> {
        std::fs::read_to_string(&self.cache_path(key))
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