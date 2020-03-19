use serde::*;
use serde_tuple::*;
use std::time::SystemTime;
use crate::util::temporal;

#[derive(Clone, Debug, Serialize, Deserialize_tuple)]
pub struct Aircraft {
    pub icao24: String,
    pub callsign: Option<String>,           // Can be null if not received
    pub origin_country: String,
    pub time_position: Option<i64>,         // Time of last position update, as unix timestamp.  Can be null
    pub last_contact: i64,                  // Time of last update received, as unix timestamp
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

    pub fn linear_search<P>(&self, pred: P) -> Option<&Aircraft>
        where P: FnMut(&&Aircraft) -> bool {
        self.data.iter()
            .filter(pred)
            .next()
    }
}

impl Aircraft {
    pub fn _has_position_data(&self) -> bool {
        self.longitude.is_some() && self.latitude.is_some()
    }

    pub fn basic_status(&self) -> String {
        // e.g. "AAA1234 (United States, ICAO: ab42de, Last contact: 2 seconds ago)"
        let last_contact = temporal::get_duration(
            temporal::systemtime_from_datetime(
                temporal::utc_datetime_from_timestamp(self.last_contact as i64)),
            SystemTime::now());

        format!("{} ({}, ICAO: {}, Last contact: {})",
            self.callsign.as_ref().unwrap_or(&"[Unknown callsign]".to_string()),
            self.origin_country,
            self.icao24,
            last_contact.map(|x| format!("{} seconds ago", x.as_secs())).unwrap_or("[Unknown]".to_string())
        )
    }

}