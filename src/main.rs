#![feature(async_await)]

extern crate piston_window;

use piston_window::*;
use std::thread;
use std::sync::mpsc;
use crate::aircraft::Aircraft;

mod aircraft;
mod simulation;
mod rendering;

fn main() {
    let gl = OpenGL::V4_5;
    let mut window: PistonWindow = WindowSettings::new("flight-radar", [512; 2])
        .graphics_api(gl)
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|_| panic!("Cannot initialise window"));

    window.set_lazy(true);

    let mut data: Vec<Aircraft> = vec![];
    let (tx, rx) = mpsc::channel();

    thread::spawn(|| simulation::simulate(tx));

    while let Some(e) = window.next() {
        match e {
            Event::Input(event, timestamp) => (),
            Event::Loop(event) => match event {
                Loop::Render(r) => { println!("Render"); rendering::render(&mut window, &e, &r, &data)},
                Loop::AfterRender(ar) => {
                    let new_data = rx.try_recv();
                    if new_data.is_ok() {println!("New data");
                        data = new_data.unwrap();
                    }
                },
                _ => ()
            },
            _ => ()
        }
    }

}
