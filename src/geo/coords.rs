const MAX_LONGITUDE: f64 = 180.0;
const MAX_LATITUDE: f64 = 90.0;

pub fn normalised_screen_coords(lon: f64, lat: f64) -> (f64, f64) {
    (lon / MAX_LONGITUDE, lat / MAX_LATITUDE)
}

pub fn origin_based_normalised_screen_coords(lon: f64, lat: f64) -> (f64, f64) {
    (
        (lon / MAX_LONGITUDE) * 0.5 + 0.5,
        1.0 - ((lat / MAX_LATITUDE) * 0.5 + 0.5)
    )
}

pub fn normalised_mercator_coords(lon: f64, lat: f64) -> (f64, f64) {
    let x = (lon + 180.0) * (1.0 / 360.0);

    let lat_radians = lat * std::f64::consts::PI / 180.0;

    let merc_n = (std::f64::consts::FRAC_PI_4 + (lat_radians * 0.5)).tan().ln();
    let y = 0.5 - (0.5 * merc_n / (2.0 * std::f64::consts::PI));

    (x, y)
}

pub fn normalised_equirectangular_coords(lon: f64, lat: f64) -> (f64, f64) {
    (
        (lon + 180.0) * (1.0 / 360.0),
        ((lat * -1.0) + 90.0) * (1.0 / 180.0)
    )
}

pub fn transform_normalised_to_screen(coord: (f64, f64)) -> (f64, f64) {
    (coord.0, 1.0 - coord.1)
}

pub fn window_to_map(x: f64, y: f64, window_size: &[f64; 2], view_origin: &[f64; 2], zoom_level: f64) -> (f64, f64) {
    screen_coords_to_map((x / window_size[0], y / window_size[1]), view_origin, zoom_level)
}

pub fn lon_lat_to_map(lon: f64, lat: f64, view_origin: &[f64; 2], zoom_level: f64) -> (f64, f64) {

    screen_coords_to_map(
        normalised_equirectangular_coords(lon, lat),
        view_origin, zoom_level)
}

fn screen_coords_to_map(coord: (f64, f64), view_origin: &[f64; 2], zoom_level: f64) -> (f64, f64) {
    (
        (coord.0 - view_origin[0]) * zoom_level,
        (coord.1 - view_origin[1]) * zoom_level
    )
}

