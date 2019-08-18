use reqwest;

pub fn get(url: &str) -> Result<String, reqwest::Error> {
    Ok(reqwest::get(url)?.text()?)
}

