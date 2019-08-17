use std::sync::mpsc::{Sender, Receiver};
use crate::aircraft::{Aircraft, AircraftData};
use std::thread;
use std::time::Duration;
use std::error::Error;

#[path = "./httpclient.rs"] mod httpclient;
use crate::sources;


pub fn simulate(trigger: Receiver<sources::Source>, out: Sender<AircraftData>) {
    loop {
        trigger.recv().and_then(|source| {
            println!("Simulating...");

            if source.should_use_cache() {
                // @TODO: Implement local cache storage
                eprintln!("Local cached data not yet implemented");
            }
            else {
                match httpclient::get(source.get_path().as_str()) {
                    Err(e) => println!("Retrieval error; {}", e.description()),
                    Ok(data) =>
                        match out.send(parse_data(data)) {
                            Err(e) => println!("Send error: {}", e.description()),
                            _ => ()
                        }
                }
            }
            Ok(())
        });
    }
}

pub fn periodic_trigger(trigger: Sender<sources::Source>, request: sources::Source, interval: u64) {
    loop {
        thread::sleep(Duration::from_secs(interval));
        trigger.send(request.clone());
    };
}

fn parse_data(data: String) -> AircraftData {
    serde_json::from_str(data.as_str())
        .unwrap_or_else(|e| panic!("Failed to deserialise response: {:?}", e))
}