use std::sync::mpsc::Sender;
use crate::aircraft::{Aircraft, AircraftData};
use std::thread;
use std::time::Duration;
use std::error::Error;

#[path = "./httpclient.rs"] mod httpclient;

const SOURCE: &str = "https://opensky-network.org/api/states/all";


pub fn simulate(out: Sender<Vec<Aircraft>>) {

    loop {
        println!("Simulating...");

        match httpclient::get(SOURCE) {
            Err(e) => println!("Retrieval error; {}", e.description()),
            Ok(data) =>
                match out.send(parse_data(data).data) {
                    Err(e) => println!("Send error: {}", e.description()),
                    _ => ()
            }
        }

        thread::sleep(Duration::from_secs(10));
    }
}

fn parse_data(data: String) -> AircraftData {
    serde_json::from_str(data.as_str())
        .unwrap_or_else(|e| panic!("Failed to deserialise response: {:?}", e))
}