const CRED_TOKEN: &str = "{CRED}";

pub struct SourceProvider {
    cred: Option<String>,
    use_cache: bool
}

pub struct Source {
    path: String,
    cred: Option<String>,
    use_cache: bool
}

impl SourceProvider {
    pub fn new(cred: &Option<String>, use_cache: bool) -> Self {
        Self { cred: cred.clone(), use_cache }
    }

    pub fn _should_use_cache(&self) -> bool { return self.use_cache; }  // @Unused
    pub fn is_authenticated(&self) -> bool { return self.cred.is_some() }

    fn source(&self, path: String) -> Source {
        Source::new(format!("https://{}opensky-network.org/api{}",
                            CRED_TOKEN,
                            path),
                    self.cred.clone(), self.use_cache)
    }

    pub fn source_state_vectors(&self) -> Source {
        self.source("/states/all".to_string())
    }
}

impl Source {
    pub fn new(path: String, cred: Option<String>, use_cache: bool) -> Self {
        Self { path, cred, use_cache }
    }

    fn resolve_path(&self, cred: &Option<String>) -> String {
        return self.path
            .replace(CRED_TOKEN,
                cred.as_ref().map(|s| s.as_str()).unwrap_or_else(|| "")
            )
    }

    pub fn get_path(&self) -> String {
        self.resolve_path(&self.cred)
    }

    pub fn get_underlying_path(&self) -> String {
        self.resolve_path(&None)
    }

    pub fn should_use_cache(&self) -> bool { return self.use_cache; }
}

impl Clone for Source {
    fn clone(&self) -> Self {
        Self { path: self.path.clone(), cred: self.cred.clone(), use_cache: self.use_cache }
    }
}









