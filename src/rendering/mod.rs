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

pub fn prepare_backbuffer(buffer: &mut BackBuffer, draw_size: &[u32; 2], zoom_level: f64, view_origin: [f64; 2], aircraft: &AircraftData) {
    clear_backbuffer(buffer);

    // Render aircraft
    let aircraft_rendered = aircraft.data.iter()
        .map(|x| render_aircraft(x, buffer, draw_size, zoom_level, &view_origin))
        .filter(|&x| x)
        .count();

    println!("Processed: {}, Rendered: {}", aircraft.data.len(), aircraft_rendered);
}

pub fn perform_rendering(g: &mut G2d, context: &Context, render_size: (f64, f64), zoom_level: f64, view_origin: [f64; 2], geo_data: &GeoData) {
    piston_window::clear([0.0, 0.0, 0.0, 1.0], g);

    // Render geography
    let geo_segments_rendered = geo_data.coast
        .iter()
        .map(|x| render_coastline(x, g, context, render_size, zoom_level, &view_origin))
        .sum::<usize>();

    println!("Rendered {} geo segments", geo_segments_rendered);
}

fn clear_backbuffer(canvas: &mut BackBuffer) {
    canvas.pixels_mut().for_each(|mut p| p.0 = [0, 0, 0, 0]);
}

fn render_aircraft(aircraft: &Aircraft, buffer: &mut BackBuffer, view_size: &[u32; 2], zoom_level: f64, view_origin: &[f64; 2]) -> bool {
    if let (Some(lon), Some(lat)) = (aircraft.longitude, aircraft.latitude) {
        let (x_norm, y_norm) = coords::normalised_equirectangular_coords(lon, lat);
        let (x_norm_scaled, y_norm_scaled) = (
            (x_norm * zoom_level) - (view_origin[0] - 0.0),
            (y_norm * zoom_level) - (view_origin[1] - 0.0)
        );

        if x_norm_scaled >= 0.0 && y_norm_scaled >= 0.0 && x_norm_scaled < 1.0 && y_norm_scaled < 1.0 {
            let (x, y) = ((x_norm_scaled * view_size[0] as f64) as u32, (y_norm_scaled * view_size[1] as f64) as u32);

            buffer.put_pixel(x, y, colour::GREEN);
            return true
        }
    }
    false
}

fn render_coastline(data: &CoastlineDataEntry, g: &mut piston_window::G2d, context: &Context,
                    _render_size: (f64, f64), zoom_level: f64, view_origin: &[f64; 2]) -> usize {
    let transformed = data.vertices.iter()
        .map(|v| coords::normalised_equirectangular_coords(v[0], v[1]))
        .map(|v| [v.0 * zoom_level - (view_origin[0] - 0.0), v.1 * zoom_level - (view_origin[1] - 0.0)])
        .collect::<Vec<[f64; 2]>>();

    (0..(transformed.len() - 1))
        .map(|ix| coastline_segment(g, context, transformed[ix], transformed[ix + 1]))
        .filter(|x| *x)
        .count()
}

fn coastline_segment(g: &mut piston_window::G2d, context: &Context, v0: [f64; 2], v1: [f64; 2]) -> bool {
    if segment_in_bounds(v0, v1) {
        line_from_to(COLOUR_COASTLINE, COASTLINE_WIDTH, v0, v1, context.transform, g);
        true
    }
    else { false }
}

fn segment_in_bounds(v0: [f64; 2], v1: [f64; 2]) -> bool {
    !(                                                                      // OUT of bounds if...
        (v0[0] < 0.0 && v1[0] < 0.0) || (v0[0] > 1.0 && v1[0] > 1.0) ||     // BOTH points are off the left, or off the right,
        (v0[1] < 0.0 && v1[1] < 0.0) || (v0[1] > 1.0 && v1[1] > 1.0)        // or off the top, or off the bottom, of the window bounds
    )
}