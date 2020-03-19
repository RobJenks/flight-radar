use std::sync::mpsc::{Sender, Receiver, SendError};
use std::thread;
use std::time::Duration;

use crate::data::aircraft::AircraftData;
use crate::data::flight::FlightData;
use crate::sources::{sources, httpclient, caching};

pub enum RetrievalError {
    HttpRequestError(reqwest::Error),
    JsonParsingError(serde_json::error::Error),
    FlightChannelSendError(SendError<FlightData>)
}


pub fn simulate(trigger: Receiver<sources::Source>, out: Sender<AircraftData>) {
    let mut cache = caching::FilesystemCache::new(|x| x.clone());
    loop {
        trigger.recv()
            .and_then(|source| {
                println!("Simulating...");

                let data = if source.should_use_cache() {
                    let key = cache.cache_key(&source.get_underlying_path());
                    cache.get(&key)
                        .map(|data| Some(data))
                        .unwrap_or_else(|e| {
                            eprintln!("Cannot retrieve \"{}\" from cache: {}", key, e.to_string());
                            None
                        })
                } else {
                    httpclient::get(source.get_path().as_str())
                        .map(|data| Some(data))
                        .unwrap_or_else(|e| {
                            eprintln!("Remote retrieval error: {}", e.to_string());
                            None
                        })
                };

                if let Some(data) = data {
                    match out.send(parse_data(data)) {
                        Err(e) => eprintln!("Send error: {}", e.to_string()),
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

pub fn retrieve_flight_data(request: Receiver<(String, sources::Source)>, out: Sender<FlightData>) {
    loop {
        request.recv()
            .map_or_else(
                |e| println!("Failed to receive flight data ({:?})", e),
                |(icao24, source)| {
                    println!("Retrieving flight data for \"{}\"...", icao24);
                    perform_flight_data_lookup(source)
                        .map(|x| out.send(x).unwrap_or_else(|e| {
                            println!("Failed to return flight data for \"{}\" to simulation ({})", icao24, e.to_string())
                        }))
                        .and_then(|_| Some(println!("Retrieved flight data successfully")));
            });
    }
}

fn perform_flight_data_lookup(source: sources::Source) -> Option<FlightData> {
    let data = httpclient::get(source.get_path().as_str());
    println!("SOURCE: {:?}, RESULT: {:?}", source, data);
    data.map_or_else(|_| None, |x| serde_json::from_str::<FlightData>(x.as_str())
        .map_or_else(|_| None, |x| Some(x)))
}


fn parse_data(data: String) -> AircraftData {
    serde_json::from_str(data.as_str())
        .unwrap_or_else(|e| panic!("Failed to deserialise response: {:?}", e))
}



impl From<reqwest::Error> for RetrievalError {
    fn from(error: reqwest::Error) -> RetrievalError {
        RetrievalError::HttpRequestError(error)
    }
}
impl From<serde_json::error::Error> for RetrievalError {
    fn from(error: serde_json::error::Error) -> RetrievalError {
        RetrievalError::JsonParsingError(error)
    }
}
impl From<SendError<FlightData>> for RetrievalError {
    fn from(error: SendError<FlightData>) -> RetrievalError {
        RetrievalError::FlightChannelSendError(error)
    }
}















