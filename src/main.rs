mod core;
mod data;
mod geo;
mod rendering;
mod simulation;
mod sources;

use crate::core::flight_radar;
use shader_version::OpenGL;

fn main() {
    let mut flight_radar = flight_radar::FlightRadar::create(
        flight_radar::BuildOptions {
            gl_version: OpenGL::V4_5,
            use_cache: false
        }
    );

    flight_radar.execute();
}
