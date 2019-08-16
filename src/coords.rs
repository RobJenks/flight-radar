const MAX_LONGITUDE: f64 = 180.0;
const MAX_LATITUDE: f64 = 90.0;

pub fn normalised_screen_coords(lon: f64, lat: f64) -> (f64, f64) {
    (lon / MAX_LONGITUDE, lat / MAX_LATITUDE)
}

pub fn origin_based_normalised_screen_coords(lon: f64, lat: f64) -> (f64, f64) {
    ((lon / MAX_LONGITUDE) * 0.5 + 0.5, (lat / MAX_LATITUDE) * 0.5 + 0.5)
}