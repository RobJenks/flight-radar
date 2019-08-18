use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::time::Duration;
use std::error::Error;

use crate::data::aircraft::AircraftData;
use crate::sources::{sources, httpclient, caching};
use crate::sources::caching::FilesystemCache;


pub fn simulate(trigger: Receiver<sources::Source>, out: Sender<AircraftData>) {
    let mut cache = caching::FilesystemCache::new();
    loop {
        trigger.recv()
            .and_then(|source| {
                println!("Simulating...");

                let data = if source.should_use_cache() {
                    let key = FilesystemCache::cache_key(&source.get_underlying_path());
                    cache.get(&key)
                        .map(|data| Some(data))
                        .unwrap_or_else(|e| {
                            eprintln!("Cannot retrieve \"{}\" from cache: {}", key, e.description());
                            None
                        })
                } else {
                    httpclient::get(source.get_path().as_str())
                        .map(|data| Some(data))
                        .unwrap_or_else(|e| {
                            eprintln!("Remote retrieval error: {}", e.description());
                            None
                        })
                };

                if let Some(data) = data {
                    match out.send(parse_data(data)) {
                        Err(e) => eprintln!("Send error: {}", e.description()),
                        _ => ()
                    }
                }
                Ok(())
            })
            .expect("Failed to receive new data for simulation");
    }
}

pub fn periodic_trigger(trigger: Sender<sources::Source>, request: sources::Source, interval: u64) {
    loop {
        thread::sleep(Duration::from_secs(interval));
        trigger
            .send(request.clone())
            .expect("Failed to trigger periodic simulation");
    };
}

fn parse_data(data: String) -> AircraftData {
    serde_json::from_str(data.as_str())
        .unwrap_or_else(|e| panic!("Failed to deserialise response: {:?}", e))
}