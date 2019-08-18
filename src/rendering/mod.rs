#![allow(dead_code)] pub mod colour;
pub mod screenshot;

use ::image;
use crate::data::aircraft::{Aircraft, AircraftData};
use crate::data::geography::{GeoData, CoastlineDataEntry};
use crate::geo::coords;
use piston_window::*;
use image::Rgba;

pub type BackBuffer = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;

const COLOUR_COASTLINE: [f32; 4] = [140.0/255.0, 184.0/255.0, 151.0/255.0, 0.5];
const COLOUR_AIRCRAFT: Rgba<u8> = colour::GREEN;

const COASTLINE_WIDTH: f64 = 0.0005;

pub fn prepare_backbuffer(buffer: &mut BackBuffer, draw_size: &[u32; 2], aircraft: &AircraftData) {
    clear_backbuffer(buffer);

    // Render aircraft
    let aircraft_rendered = aircraft.data.iter()
        .map(|x| render_aircraft(x, buffer, draw_size))
        .filter(|&x| x)
        .count();

    println!("Processed: {}, Rendered: {}", aircraft.data.len(), aircraft_rendered);
}

pub fn perform_rendering(g: &mut G2d, context: &Context, render_size: (f64, f64), geo_data: &GeoData) {
    piston_window::clear([0.0, 0.0, 0.0, 1.0], g);

    // Render geography
    geo_data.coast.iter()
        .for_each(|x| render_coastline(x, g, context, render_size));

}

fn clear_backbuffer(canvas: &mut BackBuffer) {
    canvas.pixels_mut().for_each(|mut p| p.0 = [0, 0, 0, 0]);
}

fn render_aircraft(aircraft: &Aircraft, buffer: &mut BackBuffer, view_size: &[u32; 2]) -> bool {
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

fn render_coastline(data: &CoastlineDataEntry, g: &mut piston_window::G2d, context: &Context, render_size: (f64, f64)) {
    let transformed = data.vertices.iter()
        .map(|v| coords::normalised_equirectangular_coords(v[0], v[1]))
        .map(|v| [v.0, v.1])
        .collect::<Vec<[f64; 2]>>();

    for ix in 0..(transformed.len() - 1) {
        coastline_segment(g, context, transformed[ix], transformed[ix + 1]);
    }
}

fn coastline_segment(g: &mut piston_window::G2d, context: &Context, v0: [f64; 2], v1: [f64; 2]) {
    line_from_to(COLOUR_COASTLINE, COASTLINE_WIDTH, v0, v1, context.transform, g);
}