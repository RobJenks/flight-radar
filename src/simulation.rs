use std::sync::mpsc::{Sender, Receiver};
use crate::aircraft::{Aircraft, AircraftData};
use std::thread;
use std::time::Duration;
use std::error::Error;

#[path = "./httpclient.rs"] mod httpclient;


pub fn simulate(trigger: Receiver<String>, out: Sender<AircraftData>) {
    loop {
        trigger.recv().and_then(|url| {
            println!("Simulating...");

            match httpclient::get(url.as_str()) {
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

pub fn periodic_trigger(trigger: Sender<String>, request: String, interval: u64) {
    loop {
        thread::sleep(Duration::from_secs(interval));
        trigger.send(request.clone());
    };
}

fn parse_data(data: String) -> AircraftData {
    serde_json::from_str(data.as_str())
        .unwrap_or_else(|e| panic!("Failed to deserialise response: {:?}", e))
}