use serde::*;

#[derive(Debug, Clone, Deserialize)]
pub struct Flight {
    #[serde(rename = "icao24")]                             pub icao24: String,
    #[serde(rename = "firstSeen")]                          pub first_seen: i64,
    #[serde(rename = "estDepartureAirport")]                pub est_departure_airport: String,
    #[serde(rename = "lastSeen")]                           pub last_seen: i64,
    #[serde(rename = "estArrivalAirport")]                  pub est_arrival_airport: String,
    #[serde(rename = "callsign")]                           pub callsign: String,
    #[serde(rename = "estDepartureAirportHorizDistance")]   pub est_departure_airport_horiz_distance: i64,
    #[serde(rename = "estDepartureAirportVertDistance")]    pub est_departure_airport_vert_distance: i64,
    #[serde(rename = "estArrivalAirportHorizDistance")]     pub est_arrival_airport_horiz_distance: i64,
    #[serde(rename = "estArrivalAirportVertDistance")]      pub est_arrival_airport_vert_distance: i64,
    #[serde(rename = "departureAirportCandidatesCount")]    pub departure_airport_candidates_count: i64,
    #[serde(rename = "arrivalAirportCandidatesCount")]      pub arrival_airport_candidates_count: i64
}

// Collection type returned in queries
pub type FlightData = Vec<Flight>;
