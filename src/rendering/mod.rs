#![allow(dead_code)] pub mod colour;

use ::image;
use crate::data::aircraft::{Aircraft};
use crate::geo::coords;

pub type BackBuffer = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;

pub fn render_aircraft(aircraft: &Aircraft, buffer: &mut BackBuffer, view_size: &[u32; 2]) -> bool {
    if let (Some(lon), Some(lat)) = (aircraft.longitude, aircraft.latitude) {
        let (x_norm, y_norm) = coords::normalised_equirectangular_coords(lon, lat);
        let (x, y) = ((x_norm * view_size[0] as f64) as u32, (y_norm * view_size[1] as f64) as u32);

        if x <= view_size[0] && y <= view_size[1] {
            buffer.put_pixel(x, y, colour::GREEN);
            return true
        }
    }
    false
}

pub fn clear_backbuffer(canvas: &mut BackBuffer) {
    canvas.pixels_mut().for_each(|mut p| p.0 = [0, 0, 0, 255]);
}
