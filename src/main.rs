#![feature(async_await)]

extern crate piston_window;

use piston_window::*;
use std::thread;
use std::sync::mpsc;
use ::image;
use crate::aircraft::{Aircraft, AircraftData};
use image::Rgba;
use std::time::Duration;

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

    window.set_lazy(false);

    let mut data: AircraftData = AircraftData::empty();

    // Channel (Event loop -> trigger simulation)
    let (tx_simulate, rx_simulate) = mpsc::channel();
    let tx_periodic_simulation = tx_simulate.clone();

    // Channel (Simulation -> Event loop })
    let (tx_data, rx_data) = mpsc::channel();

    // Simulation thread and periodic trigger
    thread::spawn(|| simulation::simulate(rx_simulate, tx_data));
    thread::spawn(|| simulation::periodic_trigger(tx_periodic_simulation, 5));

    let mut draw_size: [u32; 2] = [window.draw_size().width as u32, window.draw_size().height as u32];
    let mut canvas: rendering::BackBuffer = image::ImageBuffer::new(draw_size[0], draw_size[1]);
    let mut texture_context = TextureContext { factory: window.factory.clone(), encoder: window.factory.create_command_buffer().into() };
    let mut texture: G2dTexture = Texture::from_image(&mut texture_context,&canvas, &TextureSettings::new()).unwrap();

    while let Some(e) = window.next() {
        match e {
            Event::Input(event, timestamp) => match event {
                Input::Resize(args) => {
                    draw_size = args.draw_size;
                    canvas = image::ImageBuffer::new(draw_size[0], draw_size[1]);
                    texture_context = TextureContext { factory: window.factory.clone(), encoder: window.factory.create_command_buffer().into() };
                    texture = Texture::from_image(&mut texture_context,&canvas, &TextureSettings::new()).unwrap();

                    tx_simulate.send(());
                },
                //Input::Move(mv) => println!("{:?}", mv),
                _ => ()
            }
            Event::Loop(event) => match event {
                Loop::Render(r) => {
                    texture.update(&mut texture_context, &canvas).unwrap();
                    window.draw_2d(&e, |context, mut g, device| {
                        texture_context.encoder.flush(device);
                        clear([0.0; 4], g);
                        image(&texture, context.transform, g);
                    });
                },
                Loop::AfterRender(ar) => {
                    rx_data.try_recv()
                        .and_then(|d| {
                            data = d;

                            rendering::clear_backbuffer(&mut canvas);

                            Ok(data.data.iter()
                                .map(|x| rendering::render_aircraft(x, &mut canvas, &draw_size))
                                .filter(|&x| x)
                                .count())

                    }).and_then(|x| { println!("Rendered count: {}", x); Ok(())});
                },
                _ => ()
            },
            _ => ()
        }
    }
}

//fn create_buffer(window: &mut PistonWindow, size: [u32; 2]) -> (BackBuffer, G2dTexture, TextureContext) {
//    let canvas = image::ImageBuffer::new(size[0], size[1]);
//    let mut texture_context = TextureContext { factory: window.factory.clone(), encoder: window.factory.create_command_buffer().into() };
//    let mut texture: G2dTexture = Texture::from_image(&mut texture_context,&canvas, &TextureSettings::new())
//        .unwrap_or_else(|_| panic!("Failed to create buffer texture"));
//
//    (canvas, texture, texture_context)
//
//}