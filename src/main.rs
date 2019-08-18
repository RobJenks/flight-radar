extern crate piston_window;
extern crate chrono;

use piston_window::*;
use chrono::Utc;
use std::thread;
use std::sync::mpsc;
use ::image;
use image::ImageFormat;
use crate::data::aircraft::AircraftData;
use crate::sources::sources::SourceProvider;

mod data;
mod geo;
mod sources;
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

    // Source provider
    let cred = get_creds();
    let source_provider = SourceProvider::new(cred, false);
    println!("Connected to {} sources", if source_provider.is_authenticated() { "authenticated" } else { "unauthenticated" });

    let mut data: AircraftData;

    // Channel (Event loop -> trigger simulation)
    let (tx_simulate, rx_simulate) = mpsc::channel();
    let tx_periodic_simulation = tx_simulate.clone();

    // Channel (Simulation -> Event loop })
    let (tx_data, rx_data) = mpsc::channel();

    // Simulation thread and periodic trigger
    let periodic_source = source_provider.source_state_vectors();
    thread::spawn(move || simulation::simulate(rx_simulate, tx_data));
    thread::spawn(move || simulation::periodic_trigger(tx_periodic_simulation, periodic_source, 2));

    let mut draw_size: [u32; 2] = [window.draw_size().width as u32, window.draw_size().height as u32];
    let mut canvas: rendering::BackBuffer = image::ImageBuffer::new(draw_size[0], draw_size[1]);
    let mut texture_context = TextureContext { factory: window.factory.clone(), encoder: window.factory.create_command_buffer().into() };
    let mut texture: G2dTexture = Texture::from_image(&mut texture_context,&canvas, &TextureSettings::new()).unwrap();

    while let Some(e) = window.next() {
        match e {
            Event::Input(event, _timestamp) => match event {
                Input::Resize(args) => {
                    draw_size = args.draw_size;
                    canvas = image::ImageBuffer::new(draw_size[0], draw_size[1]);
                    texture_context = TextureContext { factory: window.factory.clone(), encoder: window.factory.create_command_buffer().into() };
                    texture = Texture::from_image(&mut texture_context,&canvas, &TextureSettings::new()).unwrap();

                    let source = source_provider.source_state_vectors();
                    tx_simulate
                        .send(source)
                        .expect("Failed to trigger simulation cycle");
                },
                Input::Button(args) => {
                    match args.button {
                        Button::Keyboard(Key::F12) if args.state == ButtonState::Release => screenshot(&canvas),
                        _ => ()
                    }
                }
                _ => ()
            }
            Event::Loop(event) => match event {
                Loop::Render(r) => {
                    texture.update(&mut texture_context, &canvas).unwrap();
                    window.draw_2d(&e, |_context: Context, g, device| {
                        // Global transform to a [0.0 1.0] coordinate space, in each axis
                        let (size_x, size_y) = (r.draw_size[0] as f64, r.draw_size[1] as f64);
                        let context = piston_window::Context::new_abs(size_x, size_y).scale(size_x, size_y);

                        clear([0.0; 4], g);

                        texture_context.encoder.flush(device);
                        image(&texture, context.scale(1.0 / texture.get_width() as f64, 1.0 / texture.get_height() as f64).transform, g);
                    });
                },
                Loop::AfterRender(_ar) => {
                    if let Ok(d) = rx_data.try_recv() {
                        data = d;

                        rendering::clear_backbuffer(&mut canvas);
                        let rendered = data.data.iter()
                            .map(|x| rendering::render_aircraft(x, &mut canvas, &draw_size))
                            .filter(|&x| x)
                            .count();

                        println!("Processed: {}, Rendered: {}", data.data.len(), rendered);
                    }
                },
                _ => ()
            },
            _ => ()
        }
    }
}

fn get_creds() -> Option<String> {
    match std::fs::read_to_string("cred") {
        Ok(x) => Some(x),
        _ => None
    }
}

fn screenshot(buffer: &rendering::BackBuffer) {
    let filename = format!("image-{}.png", Utc::now().format("%Y%m%d-%H%M%S"));
    buffer
        .save_with_format(filename.as_str(), ImageFormat::PNG)
        .and_then(|_| {
            println!("Screenshot saved to \"{}\"", filename);
            Ok(())
        })
        .expect("Failed to save screenshot");
}