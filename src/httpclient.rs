use reqwest;
use std::error::Error;
use serde::Deserialize;

pub fn get(url: &str) -> Result<String, reqwest::Error> {
    Ok(reqwest::get(url)?.text()?)
}

