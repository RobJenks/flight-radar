use std::sync::mpsc::{Sender, Receiver};
use crate::aircraft::{Aircraft, AircraftData};
use std::thread;
use std::time::Duration;
use std::error::Error;

#[path = "./httpclient.rs"] mod httpclient;

const SOURCE: &str = "https://opensky-network.org/api/states/all";


pub fn simulate(trigger: Receiver<()>, out: Sender<AircraftData>) {
    loop {
        trigger.recv().and_then(|_| {
            println!("Simulating...");

            match httpclient::get(SOURCE) {
                Err(e) => println!("Retrieval error; {}", e.description()),
                Ok(data) =>
                    match out.send(parse_data(data)) {
                        Err(e) => println!("Send error: {}", e.description()),
                        _ => ()
                    }
            }
            Ok(())
        });
    }
}

pub fn periodic_trigger(trigger: Sender<()>, interval: u64) {
    loop {
        thread::sleep(Duration::from_secs(interval));
        trigger.send(());
    };
}

fn parse_data(data: String) -> AircraftData {
    serde_json::from_str(data.as_str())
        .unwrap_or_else(|e| panic!("Failed to deserialise response: {:?}", e))
}