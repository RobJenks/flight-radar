pub struct SourceProvider {
    cred: Option<String>,
    use_cache: bool
}

pub struct Source {
    path: String,
    use_cache: bool
}

impl SourceProvider {
    pub fn new(cred: Option<String>, use_cache: bool) -> Self {
        Self { cred, use_cache }
    }

    pub fn _should_use_cache(&self) -> bool { return self.use_cache; }  // @Unused
    pub fn is_authenticated(&self) -> bool { return self.cred.is_some() }

    fn source(&self, path: String) -> Source {
        Source::new(format!("https://{}opensky-network.org/api{}",
                            *self.cred.as_ref().unwrap_or(&String::new()),
                            path),
                    self.use_cache)
    }

    pub fn source_state_vectors(&self) -> Source {
        self.source("/states/all".to_string())
    }
}

impl Source {
    pub fn new(path: String, use_cache: bool) -> Self {
        Self { path, use_cache }
    }

    pub fn get_path(&self) -> String { return self.path.clone() }
    pub fn should_use_cache(&self) -> bool { return self.use_cache; }

    pub fn _get_cache_key(&self) -> String {    // @Unused
        self.path.chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect::<String>()
    }
}

impl Clone for Source {
    fn clone(&self) -> Self {
        Self { path: self.path.clone(), use_cache: self.use_cache }
    }
}









