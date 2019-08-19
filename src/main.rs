extern crate piston_window;
extern crate chrono;

use piston_window::*;
use std::thread;
use std::sync::mpsc;
use ::image;
use crate::data::aircraft::AircraftData;
use crate::sources::sources::SourceProvider;
use gfx::Device;
use crate::rendering::BackBuffer;

mod data;
mod geo;
mod sources;
mod simulation;
mod rendering;

const SCROLL_SCALING_FACTOR: f64 = 0.1;


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

    let mut data: AircraftData = AircraftData::empty();
    let geo_data = data::geography::load_coastline_data();

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
    let mut draw_sizef: [f64; 2] = [draw_size[0] as f64, draw_size[1] as f64];
    let mut canvas: rendering::BackBuffer = image::ImageBuffer::new(draw_size[0], draw_size[1]);
    let mut texture_context = TextureContext { factory: window.factory.clone(), encoder: window.factory.create_command_buffer().into() };
    let mut texture: G2dTexture = Texture::from_image(&mut texture_context,&canvas, &TextureSettings::new()).unwrap();

    let mut cursor_pos = [0.0, 0.0];
    let mut zoom_level = 1.0;
    let mut zoom_centre = [0.5, 0.5];

    while let Some(e) = window.next() {
        match e {
            Event::Input(event, _timestamp) => match event {
                Input::Resize(args) => {
                    draw_size = args.draw_size;
                    draw_sizef = [draw_size[0] as f64, draw_size[1] as f64];
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
                        Button::Keyboard(Key::F12) if args.state == ButtonState::Release => rendering::screenshot::display_screenshot(),
                        _ => ()
                    }
                },
                Input::Move(args) => {
                    match args {
                        Motion::MouseCursor(cursor) => cursor_pos = cursor,
                        Motion::MouseScroll(scroll) => {
                            perform_zoom(scroll, cursor_pos, [window.size().width, window.size().height], &mut zoom_level, &mut zoom_centre);
                            update_backbuffer(&mut canvas, draw_size, zoom_level, &data);
                        },
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
                        let size = (r.draw_size[0] as f64, r.draw_size[1] as f64);
                        let scaled_size = (size.0 / zoom_level, size.1 / zoom_level);

                        let context = piston_window::Context::new_abs(size.0, size.1)
                            .scale(size.0, size.1)
                            .trans(0.0, 0.0); //zoom_centre[0] * scaled_size.0, zoom_centre[1] * scaled_size.1);

                        // Render all window content
                        rendering::perform_rendering(g, &context, scaled_size, zoom_level, &geo_data);

                        // Apply pre-rendered backbuffer target
                        texture_context.encoder.flush(device);
                        image(&texture, context.scale(1.0 / texture.get_width() as f64, 1.0 / texture.get_height() as f64).transform, g);
                    });
                },
                Loop::AfterRender(_ar) => {
                    if let Ok(d) = rx_data.try_recv() {
                        data = d;
                        let dims = [canvas.dimensions().0, canvas.dimensions().1];
                        update_backbuffer(&mut canvas, dims, zoom_level, &data);
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

fn update_backbuffer(canvas: &mut BackBuffer, draw_size: [u32; 2], zoom_level: f64, data: &AircraftData) {
    rendering::prepare_backbuffer(canvas, &draw_size, zoom_level,&data);
}

fn perform_zoom(scroll: [f64; 2], cursor_pos: [f64; 2], window_size: [f64; 2], zoom_level: &mut f64, zoom_centre: &mut [f64; 2]) {
    let (h_scroll, v_scroll) = (scroll[0], scroll[1]);

    *zoom_centre = [cursor_pos[0] / window_size[0], cursor_pos[1] / window_size[1]];
    *zoom_level += (v_scroll * SCROLL_SCALING_FACTOR);

    println!("Zoom of {:?}, level: {}, centre: {:?} (cursor: {:?}, window_size: {:?})", scroll, zoom_level, zoom_centre, cursor_pos, window_size);
}