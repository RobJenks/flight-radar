fn source(cred: Option<String>, path: String) -> String {
    format!("https://{}opensky-network.org/api{}",
            cred.unwrap_or(String::new()),
            path
    )
}

pub fn source_state_vectors(cred: Option<String>) -> String {
    source(cred, "/states/all".to_string())
}
