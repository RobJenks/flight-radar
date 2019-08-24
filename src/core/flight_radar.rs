extern crate piston_window;
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Sender};
use ::image;
use piston_window::*;
use sources::sources::{Source, SourceProvider};
use crate::data;
use crate::sources;
use crate::simulation;
use crate::rendering;
use crate::data::geography;
use crate::data::aircraft::AircraftData;
use crate::rendering::BackBuffer;
use std::cell::{RefCell, Ref, RefMut};

const SCROLL_SCALING_FACTOR: f64 = 0.1;


pub struct FlightRadar {
    window: RefCell<PistonWindow>,
    source_provider: SourceProvider,

    data: AircraftData,
    geo_data: geography::GeoData,

    draw_size: [u32; 2],
    draw_sizef: [f64; 2],
    canvas: BackBuffer,

    zoom_level: f64,
    view_origin: [f64; 2],
    cursor_pos: [f64; 2]
}

impl FlightRadar {
    pub fn execute(&mut self) {
        // Channel (Event loop -> trigger simulation)
        let (tx_data, rx_data) = mpsc::channel();   // Channel (Simulation -> Event loop)
        let (tx_simulate, rx_simulate) = mpsc::channel();   // (Event loop -> trigger simulation)
        let tx_periodic_simulation = tx_simulate.clone();

        // Simulation thread and periodic trigger
        let periodic_source = self.source_provider.source_state_vectors();
        thread::spawn(move || simulation::simulate(rx_simulate, tx_data));
        thread::spawn(move || simulation::periodic_trigger(tx_periodic_simulation, periodic_source, 2));

        let factory: GfxFactory = self.window().factory.clone();
        let mut texture_context = TextureContext { factory, encoder: self.window_mut().factory.create_command_buffer().into() };
        let mut texture: G2dTexture = Texture::from_image(&mut texture_context,&self.canvas, &TextureSettings::new()).unwrap();

        //while let Some(e) = self.window_mut().next() {
        loop {
            let e_next = self.window_mut().next();
            if e_next == None { break; }
            let e = e_next.unwrap();

            match e {
                Event::Input(event, _timestamp) => match event {
                    Input::Resize(args) => {
                        self.update_size(&args.draw_size);

                        let factory: GfxFactory = self.window().factory.clone();
                        texture_context = TextureContext { factory, encoder: self.window_mut().factory.create_command_buffer().into() };
                        texture = Texture::from_image(&mut texture_context,&self.canvas, &TextureSettings::new()).unwrap();

                        self.trigger_simulation(&tx_simulate);
                    },
                    Input::Button(args) => {
                        match args.button {
                            Button::Keyboard(Key::F12) if args.state == ButtonState::Release => rendering::screenshot::display_screenshot(),
                            _ => ()
                        }
                    },
                    Input::Move(args) => {
                        match args {
                            Motion::MouseCursor(cursor) => self.cursor_pos = cursor,
                            Motion::MouseScroll(scroll) => {
                                self.perform_zoom(scroll);
                                self.update_backbuffer();
                            },
                            _ => ()
                        }
                    }
                    _ => ()
                }
                Event::Loop(event) => match event {
                    Loop::Render(_) => {
                        texture.update(&mut texture_context, &self.canvas).unwrap();
                        let zoom_level = self.zoom_level;
                        let view_origin = self.view_origin;
                        let render_size = self.draw_sizef;
                        let scaled_size = (self.draw_sizef[0] / self.zoom_level, self.draw_sizef[1] / self.zoom_level);

                        self.window.borrow_mut().draw_2d(&e, |_context: Context, g, device| {
                            // Global transform to a [0.0 1.0] coordinate space, in each axis
                            let context = piston_window::Context::new_abs(render_size[0], render_size[1])
                                .scale(render_size[0], render_size[1]);

                            // Render all window content
                            rendering::perform_rendering(g, &context, scaled_size, zoom_level, view_origin, &self.geo_data);

                            // Apply pre-rendered backbuffer target
                            texture_context.encoder.flush(device);
                            image(&texture, context.scale(1.0 / texture.get_width() as f64, 1.0 / texture.get_height() as f64).transform, g);
                        });
                    },
                    Loop::AfterRender(_ar) => {
                        if let Ok(d) = rx_data.try_recv() {
                            self.data = d;
                            self.update_backbuffer();
                        }
                    },
                    _ => ()
                },
                _ => ()
            }
        }
    }

    fn window(& self) -> Ref<PistonWindow> {
        self.window.borrow()
    }

    fn window_mut(&self) -> RefMut<PistonWindow> {
        self.window.borrow_mut()
    }

    fn update_size(&mut self, size: &[u32; 2]) {
        self.draw_size = *size;
        self.draw_sizef = [self.draw_size[0] as f64, self.draw_size[1] as f64];

        self.canvas = image::ImageBuffer::new(self.draw_size[0], self.draw_size[1]);
    }

    fn trigger_simulation(&self, tx: &Sender<Source>) {
        let source = self.source_provider.source_state_vectors();
        tx.send(source)
          .expect("Failed to trigger simulation cycle");
    }

    fn update_backbuffer(&mut self) {
        rendering::prepare_backbuffer(&mut self.canvas, &self.draw_size, self.zoom_level, self.view_origin,&self.data);
    }

    #[allow(unused_parens)]
    fn perform_zoom(&mut self, scroll: [f64; 2]) {
        let (_h_scroll, v_scroll) = (scroll[0], scroll[1]);

        // Set new zoom level
        let original_zoom_level = self.zoom_level;
        self.zoom_level += (v_scroll * SCROLL_SCALING_FACTOR);

        // Limit to acceptable bounds
        self.zoom_level = self.zoom_level.max(0.1);

        // Determine pan required to maintain consistent zoom target
        let size: Size = self.window().draw_size();
        let scale_change = (1.0 / self.zoom_level - 1.0 / original_zoom_level);
        let zoom_point = [self.cursor_pos[0] / size.width, self.cursor_pos[1] / size.height];

        let offset = (-(zoom_point[0] * scale_change), -(zoom_point[1] * scale_change));
        self.view_origin[0] += offset.0;
        self.view_origin[1] += offset.1;
    }


    pub fn create(options: BuildOptions) -> Self {
        let window = FlightRadar::init_window(&options);

        let creds = FlightRadar::init_creds();
        let source_provider = SourceProvider::new(&creds, options.use_cache);
        println!("Connecting to {} sources", if source_provider.is_authenticated() { "authenticated" } else { "unauthenticated" });

        let data = AircraftData::empty();
        let geo_data = data::geography::load_coastline_data();

        let draw_size: [u32; 2] = [window.draw_size().width as u32, window.draw_size().height as u32];
        let draw_sizef: [f64; 2] = [draw_size[0] as f64, draw_size[1] as f64];
        let canvas: BackBuffer = image::ImageBuffer::new(draw_size[0], draw_size[1]);


        Self {
            window: RefCell::new(window),
            source_provider,

            data,
            geo_data,

            draw_size,
            draw_sizef,
            canvas,

            zoom_level: 1.0,
            view_origin: [0.0, 0.0],
            cursor_pos: [0.0, 0.0]
        }
    }

    fn init_window(options: &BuildOptions) -> PistonWindow {
        let mut window: PistonWindow = WindowSettings::new("flight-radar", [512; 2])
            .graphics_api(options.gl_version)
            .exit_on_esc(true)
            .build()
            .unwrap_or_else(|_| panic!("Cannot initialise window"));

        window.set_lazy(false);
        window
    }

    fn init_creds() -> Option<String> {
        match std::fs::read_to_string("cred") {
            Ok(x) => Some(x),
            _ => None
        }
    }


}


pub struct BuildOptions {
    pub gl_version: OpenGL,
    pub use_cache: bool
}