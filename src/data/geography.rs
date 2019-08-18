use crate::data::parsing;

const COASTLINE_DATA_PATH: &str = "resources/ne_110m_coastline.csv";

pub struct CoastlineDataEntry {
    pub id: i32,
    pub vertices: Vec<[f64; 2]>,
    pub scale_rank: i32,    // TBC
    pub min_zoom: f32       // TBC
}

pub struct GeoData {
    pub coast: Vec<CoastlineDataEntry>
}

impl GeoData {
    pub fn parse(coast_data: &String) -> Self {
        Self {
            coast: coast_data.split("\n")
                .map(|line| CoastlineDataEntry::parse(&line.to_string()))
                .collect::<Vec<CoastlineDataEntry>>()
        }
    }
}

impl CoastlineDataEntry {
    pub fn parse(data: &String) -> Self {
        let entries = parsing::parse_geo_shp(data).collect::<Vec<&str>>();

        Self {
            id: entries[0].parse::<i32>().expect(format!("Failed to parse ID ({})", entries[0]).as_str()),
            vertices: parsing::parse_simple_multiline_string(entries[1])
                .iter()
                .map(|verts| verts.iter()
                    .map(|&v| v.parse::<f64>().unwrap_or_else(|_| panic!("Failed to parse vertex ({})", v)))
                    .collect::<Vec<f64>>()
                )
                .map(|fv| [fv[0], fv[1]])
                .collect::<Vec<[f64; 2]>>(),
            scale_rank: entries[2].parse::<i32>().expect(format!("Failed to parse scale rank ({})", entries[2]).as_str()),
            min_zoom: entries[4].parse::<f32>().expect(format!("Failed to parse min zoom ({})", entries[4]).as_str())
        }
    }
}

pub fn load_coastline_data() -> GeoData {
    GeoData::parse(&std::fs::read_to_string(COASTLINE_DATA_PATH)
        .expect("Failed to load coastline data"))
}