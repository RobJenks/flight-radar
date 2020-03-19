extern crate piston_window;
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use ::image;
use piston_window::*;
use sources::sources::{Source, SourceProvider};
use crate::data;
use crate::sources;
use crate::simulation;
use crate::rendering;
use crate::data::geography;
use crate::data::aircraft::{Aircraft, AircraftData};
use crate::data::flight::FlightData;
use crate::rendering::BackBuffer;
use crate::text;
use std::cell::{RefCell, Ref, RefMut};
use crate::geo::coords::{lon_lat_to_map, normalise_to_window, normalised_coords};
use crate::rendering::colour::{COLOUR_SELECTED_OBJECT, COLOUR_STATUS_AREA_BACK, COLOUR_STATUS_AREA_OUTLINE, COLOUR_STATUS_AREA_TEXT};
use crate::util::temporal::get_current_timestamp_secs;

const MOUSE_LEFT: usize = 0;
const MOUSE_RIGHT: usize = 1;
const MOUSE_BUTTON_COUNT: usize = 2;

const SCROLL_SCALING_FACTOR: f64 = 0.1;
const PAN_SCALING_FACTOR: f64 = 1.5;

const MAX_OBJECT_SELECT_DISTANCE_SQ: f64 = 2.0 * 2.0;
const SELECTION_CIRCLE_RADIUS: f64 = 5.0;
const STATUS_AREA_SIZE: f64 = 0.1;

pub struct FlightRadar {
    window: RefCell<PistonWindow>,
    source_provider: SourceProvider,
    text_manager: RefCell<text::TextManager>,

    data: AircraftData,
    flight_data: FlightData,
    geo_data: geography::GeoData,

    draw_size: [u32; 2],
    draw_sizef: [f64; 2],
    window_size: [f64; 2],
    canvas: BackBuffer,

    zoom_level: f64,
    view_origin: [f64; 2],
    cursor_pos: [f64; 2],

    mouse_down_point: [Option<[f64; 2]>; MOUSE_BUTTON_COUNT],
    selected_object: Option<Aircraft>,

    tx_flight_data_req: Option<Sender<(String, Source)>>,       // (Request channel for new flight data)
    rx_flight_data_resp: Option<Receiver<FlightData>>,          // (Response channel with new flight data)
}

impl FlightRadar {
    pub fn execute(&mut self) {
        // Inter-thread channels
        let (tx_data, rx_data) = mpsc::channel();   // (Simulation -> Event loop)
        let (tx_simulate, rx_simulate) = mpsc::channel();   // (Event loop -> trigger simulation)
        let tx_periodic_simulation = tx_simulate.clone();

        // Simulation thread and periodic trigger
        let periodic_source = self.source_provider.source_state_vectors();
        thread::spawn(move || simulation::simulate(rx_simulate, tx_data));
        thread::spawn(move || simulation::periodic_trigger(tx_periodic_simulation, periodic_source, 2));

        // Async requests for flight data
        let (tx_flight_data_req, rx_flight_data_req) = mpsc::channel();     // (Request for flight data)
        let (tx_flight_data_resp, rx_flight_data_resp) = mpsc::channel();   // (Response with new flight data)

        self.tx_flight_data_req = Some(tx_flight_data_req);
        self.rx_flight_data_resp = Some(rx_flight_data_resp);
        thread::spawn(move || simulation::retrieve_flight_data(rx_flight_data_req, tx_flight_data_resp));

        let factory: GfxFactory = self.window().factory.clone();
        let mut texture_context = TextureContext { factory, encoder: self.window_mut().factory.create_command_buffer().into() };
        let mut texture: G2dTexture = Texture::from_image(&mut texture_context,&self.canvas, &TextureSettings::new()).unwrap();

        loop {
            let e_next = self.window_mut().next();
            if e_next == None { break; }
            let e = e_next.unwrap();

            match e {
                Event::Input(event, _timestamp) => match event {
                    Input::Resize(args) => {
                        let window_size = self.window().size();
                        self.update_size(&args.draw_size, window_size);

                        let factory: GfxFactory = self.window().factory.clone();
                        texture_context = TextureContext { factory, encoder: self.window_mut().factory.create_command_buffer().into() };
                        texture = Texture::from_image(&mut texture_context,&self.canvas, &TextureSettings::new()).unwrap();

                        self.trigger_simulation(&tx_simulate);
                    },
                    Input::Button(args) => {
                        match args.button {
                            Button::Keyboard(key) if args.state == ButtonState::Press => self.key_down(&key),
                            Button::Keyboard(key) if args.state == ButtonState::Release => self.key_up(&key),

                            Button::Mouse(button) if args.state == ButtonState::Press => self.mouse_down(&button),
                            Button::Mouse(button) if args.state == ButtonState::Release => self.mouse_up(&button),

                            _ => ()
                        }
                    },
                    Input::Move(args) => {
                        match args {
                            Motion::MouseCursor(cursor) => self.mouse_move(&cursor),
                            Motion::MouseRelative(movement) => self.mouse_move_relative(&movement),
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
                        let window_size = self.window_size;
                        let scaled_size = (self.draw_sizef[0] / self.zoom_level, self.draw_sizef[1] / self.zoom_level);
                        let mut text_manager = self.text_manager.borrow_mut();
                        let glyph_cache = text_manager.glyph_cache();

                        self.window.borrow_mut().draw_2d(&e, |_context: Context, g, device| {
                            // Global transform to a [0.0 1.0] coordinate space, in each axis
                            let context = piston_window::Context::new_abs(render_size[0], render_size[1])
                                .scale(render_size[0], render_size[1]);

                            // Render all window content
                            rendering::perform_rendering(g, &context, scaled_size, zoom_level, view_origin, &self.geo_data);

                            // Apply pre-rendered backbuffer target (if not panning the map)
                            if !self.is_mouse_dragging(MOUSE_RIGHT) {
                                texture_context.encoder.flush(device);
                                image(&texture, context.scale(1.0 / texture.get_width() as f64, 1.0 / texture.get_height() as f64).transform, g);
                            }

                            // Draw zoom box if relevant
                            if self.is_mouse_dragging(MOUSE_LEFT) {
                                let rect = self.get_drag_selection(MOUSE_LEFT, &window_size).unwrap_or_else(|| panic!("No drag data"));
                                rectangle(rendering::colour::COLOUR_SELECTION, rect, context.transform, g);
                            }

                            self.render_selected_object_data(glyph_cache, &context, g);

                            glyph_cache.factory.encoder.flush(device);
                        });
                    },
                    Loop::AfterRender(_ar) => {
                        if let Ok(d) = rx_data.try_recv() {
                            self.data = d;
                            self.update_backbuffer();
                        }

                        self.receive_flight_data();
                        self.update_selection()
                    },
                    _ => ()
                },
                _ => ()
            }
        }
    }

    fn key_down(&mut self, _key: &Key) { }

    fn key_up(&mut self, key: &Key) {
        match key {
            Key::Home => self.reset_view(),
            Key::F12 => rendering::screenshot::display_screenshot(),

            _ => ()
        }
    }

    fn mouse_down(&mut self, button: &MouseButton) {
        if let Some(ix) = FlightRadar::mouse_button_index(button) {
            self.mouse_down_point[ix] = Some(self.cursor_pos.clone());
        }
    }

    fn mouse_up(&mut self, button: &MouseButton) {
        if let Some(ix) = FlightRadar::mouse_button_index(button) {
            if self.is_mouse_dragging(ix) {
                self.mouse_drag_up(ix)
            } else {
                self.mouse_click(ix, &self.mouse_down_point[ix].unwrap_or_else(|| panic!("No mouse down location")));
            }

            self.mouse_down_point[ix] = None;
        }
    }

    fn mouse_click(&mut self, button_index: usize, location: &[f64; 2]) {
        match button_index {
            MOUSE_LEFT => self.map_click(location),
            _ => ()
        }
    }

    fn mouse_drag_up(&mut self, button_index: usize) {
        match button_index {
            MOUSE_LEFT => {         // Post-selection drag
                let rect = self.get_drag_selection(MOUSE_LEFT, &self.window_size).unwrap_or_else(|| panic!("No drag data"));
                self.zoom_to(&rect);
                self.update_backbuffer();
            }
            MOUSE_RIGHT => {        // Post-drag
                self.update_backbuffer();
            },
            _ => ()
        }
    }

    fn mouse_move(&mut self, cursor: &[f64; 2]) {
        self.cursor_pos = *cursor;
    }

    fn mouse_move_relative(&mut self, movement: &[f64; 2]) {
        if self.mouse_is_down(MOUSE_RIGHT) {
            self.pan_view(
                self.adjust_pan_for_map_settings([
                    -(movement[0] / self.draw_sizef[0]),
                    -(movement[1] / self.draw_sizef[1])
                ])
            );
        }
    }

    fn mouse_is_down(&self, button: usize) -> bool {
        self.mouse_down_point[button].is_some()
    }

    fn is_mouse_dragging(&self, button: usize) -> bool {
        self.mouse_down_point[button]
            .and_then(|start| Some(
                (start[0] - self.cursor_pos[0]).abs() + (start[1] - self.cursor_pos[1]).abs() > MAX_OBJECT_SELECT_DISTANCE_SQ
            ))
            .unwrap_or(false)
    }

    fn get_drag_selection(&self, button: usize, window_size: &[f64; 2]) -> Option<[f64; 4]> {
        if self.is_mouse_dragging(button) {
            let (nx, ny) = (|x| x / window_size[0], |y| y / window_size[1]);

            let start = self.mouse_down_point[button].unwrap_or_else(|| panic!("No mouse down start location"));
            Some([nx(start[0]), ny(start[1]), nx(self.cursor_pos[0] - start[0]), ny(self.cursor_pos[1] - start[1])])
        }
        else {
            None
        }
    }

    fn get_unzoomed_position(&self, pos: [f64; 2]) -> [f64; 2] {
        [self.view_origin[0] + (pos[0] / self.zoom_level),
         self.view_origin[1] + (pos[1] / self.zoom_level)]
    }

    fn window(& self) -> Ref<PistonWindow> {
        self.window.borrow()
    }

    fn window_mut(&self) -> RefMut<PistonWindow> {
        self.window.borrow_mut()
    }

    fn reset_view(&mut self) {
        self.view_origin = [0.0, 0.0];
        self.zoom_level = 1.0;
    }

    fn map_click(&mut self, location: &[f64; 2]) {
        let loc = normalised_coords(location, &self.window_size);

        // Get the closest object to this click location
        let (origin, zoom) = (self.view_origin, self.zoom_level);
        let closest = self.data.data
            .iter()
            .enumerate()
            .filter(|(_, x)| x.longitude.is_some() && x.latitude.is_some())
            .map(|(i, x)| (i, lon_lat_to_map(x.longitude.unwrap(), x.latitude.unwrap(), &origin, zoom)))
            .map(|(i, pos)| (i, ((pos.0 - loc.0).abs(), (pos.1 - loc.1).abs())))
            .map(|(i, dxy)| (i, dxy.0 * dxy.0 + dxy.1 * dxy.1))  // Squared distance to point
            .filter(|(_, d2)| *d2 <= MAX_OBJECT_SELECT_DISTANCE_SQ)
            .fold(None, |closest: Option<(usize, f64)>, (i, d2)|
                if closest.is_none() || d2 < closest.unwrap().1 {Some((i, d2))} else {closest});

        self.select_object(closest.map(|(index, _)| index));
    }

    fn select_object(&mut self, index: Option<usize>) {
        // Select the new object
        self.selected_object = index
            .map(|i| self.data.data[i].clone());

        // Issue a request for detailed flight data
        let icao24 = self.selected_object.as_ref()
            .map(|x| x.icao24.clone())
            .unwrap_or("[None]".to_string());
        let timestamp = get_current_timestamp_secs();

        if self.tx_flight_data_req.is_some() {
            println!("Issuing request for \"{}\" flight details", icao24);
            self.tx_flight_data_req.as_ref().unwrap().send((
                icao24.clone(),
                self.source_provider.source_flight_data(
                    &icao24,
                    timestamp - (2 * 24 * 60 * 60),   // -5d
                    timestamp
                )
            )).unwrap_or_else(|e| println!("Failed to issue flight details request ({:?})", e));
        }
    }

    fn render_selected_object_data(&self, glyph_cache: &mut Glyphs, context: &Context, g: &mut G2d) {
        if let Some(obj) = &self.selected_object {
            if let (Some(lon), Some(lat)) = (obj.longitude, obj.latitude) {
                let (x, y) = lon_lat_to_map(lon, lat, &self.view_origin, self.zoom_level);

                let adj = normalise_to_window(SELECTION_CIRCLE_RADIUS, SELECTION_CIRCLE_RADIUS, &self.draw_sizef);
                let (select_min, select_max) = (
                    (x - adj.0, y - adj.1),
                    (x + adj.0, y + adj.1)
                );

                // Selection highlight around object
                ellipse_from_to(COLOUR_SELECTED_OBJECT,
                                [select_min.0, select_min.1],
                                [select_max.0, select_max.1], context.transform, g);

                // Status area
                rectangle(COLOUR_STATUS_AREA_BACK, [0.0, 1.0 - STATUS_AREA_SIZE, 1.0, STATUS_AREA_SIZE], context.transform, g);
                line_from_to(COLOUR_STATUS_AREA_OUTLINE, 0.001, [0.0, 1.0 - STATUS_AREA_SIZE], [1.0, 1.0 - STATUS_AREA_SIZE], context.transform, g);

                // Object information
                // {"time":1566137050,"states":[["ac96b8","AAL137  ","United States",1566136785,1566136790,-97.0546,32.9235,228.6,false,72.02,180,-4.88,null,213.36,"0755",false,0]
                self.render_text_lines(vec![
                    obj.basic_status().as_str()
                ],
                &[0.01, 1.0 - STATUS_AREA_SIZE + 0.02], 16.0, COLOUR_STATUS_AREA_TEXT, 14, glyph_cache, context, g
                );
            }
        }
    }

    fn render_text(&self, text: &str, pos: &[f64; 2], colour: [f32; 4], font_size: u32, glyph_cache: &mut Glyphs, context: &Context, g: &mut G2d) {
        piston_window::text::Text::new_color(colour, font_size).draw(
            text,
            glyph_cache,
            &context.draw_state,
            context.transform
                .scale(1.0 / self.draw_sizef[0], 1.0 / self.draw_sizef[1])
                .trans(pos[0] * self.draw_sizef[0], pos[1] * self.draw_sizef[1]),
            g)
            .unwrap_or_else(|e| panic!("Text rendering failed ({:?})", e));
    }

    fn render_text_lines(&self, text: Vec<&str>, pos: &[f64; 2], line_spacing: f64, colour: [f32; 4],
                         font_size: u32, glyph_cache: &mut Glyphs, context: &Context, g: &mut G2d) {
        text.iter()
            .enumerate()
            .for_each(|(i, &line)| self.render_text(line, &[pos[0], pos[1] + line_spacing * i as f64],
                                                    colour, font_size, glyph_cache, context, g));
    }

    fn update_size(&mut self, size: &[u32; 2], window_size: Size) {
        self.draw_size = *size;
        self.draw_sizef = [self.draw_size[0] as f64, self.draw_size[1] as f64];
        self.window_size = [window_size.width, window_size.height];

        self.canvas = image::ImageBuffer::new(self.draw_size[0], self.draw_size[1]);
    }

    fn trigger_simulation(&self, tx: &Sender<Source>) {
        let source = self.source_provider.source_state_vectors();
        tx.send(source)
          .expect("Failed to trigger simulation cycle");
    }

    fn receive_flight_data(&mut self) {
        if self.rx_flight_data_resp.is_some() {
            let rcv = self.rx_flight_data_resp.as_ref().unwrap().try_recv();
            match rcv {
                Ok(x) => {
                    println!("Received {} flight data entries: {:?}", x.len(), x);
                    self.flight_data = x;
                }
                _ => ()     // Allowed, this is a non-blocking try-read
            }
        }
    }

    fn update_selection(&mut self) {
        if self.selected_object.is_some() {
            let new = self.selected_object.as_ref()
                .and_then(|x| self.data
                    .linear_search(|&y| x.icao24 == y.icao24)
                    .map(|x| x.clone()));

            self.selected_object = new;
        }
    }

    fn update_backbuffer(&mut self) {
        rendering::prepare_backbuffer(&mut self.canvas, &self.draw_size, self.zoom_level, self.view_origin, &self.data);
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
        let size = self.window_size;
        let scale_change = (1.0 / self.zoom_level - 1.0 / original_zoom_level);
        let zoom_point = [self.cursor_pos[0] / size[0], self.cursor_pos[1] / size[1]];

        let offset = (-(zoom_point[0] * scale_change), -(zoom_point[1] * scale_change));
        self.view_origin[0] += offset.0;
        self.view_origin[1] += offset.1;
    }

    fn zoom_to(&mut self, rect: &[f64; 4]) {
        let origin = [rect[0], rect[1]];
        let (width, height) = (rect[2] / self.zoom_level, rect[3] / self.zoom_level);

        self.view_origin = self.get_unzoomed_position(origin);
        self.zoom_level = (1.0 / width).min(1.0 / height);
        println!("rect: {:?}, width: {}, height: {}, new origin: {:?}, new zoom level: {}", rect, width, height, self.view_origin, self.zoom_level);
    }

    fn pan_view(&mut self, pan: [f64; 2]) {
        self.view_origin = [
            self.view_origin[0] + pan[0],
            self.view_origin[1] + pan[1]
        ];
    }

    fn adjust_pan_for_map_settings(&self, pan: [f64; 2]) -> [f64; 2] {
        [
            (pan[0] * PAN_SCALING_FACTOR) / self.zoom_level,
            (pan[1] * PAN_SCALING_FACTOR) / self.zoom_level
        ]
    }


    pub fn create(options: BuildOptions) -> Self {
        let mut window = FlightRadar::init_window(&options);
        let text_manager = FlightRadar::init_text_manager(text::DEFAULT_FONT.to_string(), &mut window);

        let creds = FlightRadar::init_creds();
        let source_provider = SourceProvider::new(&creds, options.use_cache);
        println!("Connecting to {} sources", if source_provider.is_authenticated() { "authenticated" } else { "unauthenticated" });

        let data = AircraftData::empty();
        let geo_data = data::geography::load_coastline_data();

        let draw_size: [u32; 2] = [window.draw_size().width as u32, window.draw_size().height as u32];
        let draw_sizef: [f64; 2] = [draw_size[0] as f64, draw_size[1] as f64];
        let window_size = [window.size().width, window.size().height];
        let canvas: BackBuffer = image::ImageBuffer::new(draw_size[0], draw_size[1]);

        Self {
            window: RefCell::new(window),
            source_provider,
            text_manager: RefCell::new(text_manager),

            data,
            flight_data: FlightData::new(),
            geo_data,

            draw_size,
            draw_sizef,
            window_size,
            canvas,

            zoom_level: 1.0,
            view_origin: [0.0, 0.0],
            cursor_pos: [0.0, 0.0],

            mouse_down_point: [None; MOUSE_BUTTON_COUNT],
            selected_object: None,

            tx_flight_data_req: None,
            rx_flight_data_resp: None
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

    fn init_text_manager(font: String, window: &mut PistonWindow) -> text::TextManager {
        let glyph_cache = window.load_font(font.as_str())
            .unwrap_or_else(|e| panic!("Failed to initialise text manager ({:?})", e));

        text::TextManager::create(font, glyph_cache)
    }

    fn init_creds() -> Option<String> {
        match std::fs::read_to_string("cred") {
            Ok(x) => Some(x),
            _ => None
        }
    }

    fn mouse_button_index(button: &MouseButton) -> Option<usize> {
        match button {
            MouseButton::Left => Some(MOUSE_LEFT),
            MouseButton::Right => Some(MOUSE_RIGHT),

            _ => None
        }
    }


}


pub struct BuildOptions {
    pub gl_version: OpenGL,
    pub use_cache: bool
}