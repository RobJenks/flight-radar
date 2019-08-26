use serde::*;
use serde_tuple::*;

#[derive(Clone, Debug, Serialize, Deserialize_tuple)]
pub struct Aircraft {
    pub icao24: String,
    pub callsign: Option<String>,           // Can be null if not received
    pub origin_country: String,
    pub time_position: Option<isize>,       // Time of last position update, as unix timestamp.  Can be null
    pub last_contact: isize,                // Time of last update received, as unix timestamp
    pub longitude: Option<f64>,             // Can be null if not received
    pub latitude: Option<f64>,              // Can be null if not received
    pub baro_altitude: Option<f32>,         // Barometric altitude, meters.  Can be null
    pub on_ground: bool,
    pub velocity: Option<f32>,              // Ground speed, m/s.  Can be null if not received
    pub true_track: Option<f32>,            // Decimal degrees clockwise from N.  Can be null
    pub vertical_rate: Option<f32>,         // m/s, positive means climbing
    pub sensors: Option<Vec<i32>>,          // Source sensor; will not contain useful data in these queries
    pub geo_altitude: Option<f32>,          // Geometric altitude, meters.  Can be null
    pub squawk: Option<String>,             // Transponder code.  Can be null
    pub spi: bool,                          // Special purpose indicator
    pub position_source: i32                // 0=ADS-B, 1=ASTERIX, 2=MLAT
}

#[derive(Debug, Clone, Deserialize)]
pub struct AircraftData {
    pub time: isize,                        // Time of data receipt

    #[serde(rename = "states")]
    pub data: Vec<Aircraft>                 // All state vectors
}

impl AircraftData {
    pub fn empty() -> Self {
        Self { time: 0, data: vec![] }
    }
}

impl Aircraft {
    pub fn _has_position_data(&self) -> bool {
        self.longitude.is_some() && self.latitude.is_some()
    }
}