use std::thread;
use std::sync::mpsc::Receiver;
use crate::aircraft::Aircraft;
use std::time::Duration;
use piston_window::*;
use piston_window::math::Vec2d;

#[path = "./colour.rs"] mod colour;
#[path = "./coords.rs"] mod coords;

const AIRCRAFT_RENDER_SIZE: f64 = 2.0;

pub fn render<E>(window: &mut PistonWindow, e: &E, r: &RenderArgs, data: &Vec<Aircraft>)
    where E: GenericEvent
{
    window.draw_2d(e, |context, mut g, device| {
        let window_size = context.get_view_size();
        clear([0.0; 4], g);

        let rect = [20.0, 20.0, 2.0, 2.0];

        ellipse(colour::RED, rect, context.transform, g);
        data.iter().for_each(|x| render_aircraft(x, &context, &mut g, &window_size));
    });
}

fn render_aircraft(aircraft: &Aircraft, context: &Context, g: &mut G2d, view_size: &Vec2d)
{
    if let (Some(lon), Some(lat)) = (aircraft.longitude, aircraft.latitude) {
        let (x_norm, y_norm) = coords::origin_based_normalised_screen_coords(lon, lat);
        assert!(x_norm >= 0.0 && y_norm >= 0.0 && x_norm < 1.0 && y_norm < 1.0);
        let (x, y) = (x_norm * view_size[0], y_norm * view_size[1]);

        let rect = [x, y, AIRCRAFT_RENDER_SIZE, AIRCRAFT_RENDER_SIZE];
        ellipse(colour::RED, rect, context.transform, g);
    }
}