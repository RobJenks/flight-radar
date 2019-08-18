use crate::data::parsing;

pub struct CoastlineDataEntry {
    pub id: i32,
    pub vertices: Vec<(f64, f64)>,
    pub scale_rank: i32,    // TBC
    pub min_zoom: f32       // TBC
}

pub struct CoastlineData ( Vec<CoastlineDataEntry> );

impl CoastlineData {
    pub fn parse(data: &String) -> Self {
        Self(data.split("\n")
            .map(|line| CoastlineDataEntry::parse(&line.to_string()))
            .collect::<Vec<CoastlineDataEntry>>())
    }
}

impl CoastlineDataEntry {
    pub fn parse(data: &String) -> Self {
        let entries = parsing::parse_geo_shp(data).collect::<Vec<&str>>();
        Self {
            id: entries[0].parse::<i32>().expect("Failed to parse ID"),
            vertices: parsing::parse_simple_multiline_string(entries[1])
                .iter()
                .map(|verts| verts.iter()
                    .map(|&v| v.parse::<f64>().unwrap_or_else(|_| panic!("Failed to parse vertex")))
                    .collect::<Vec<f64>>()
                )
                .map(|fv| (fv[0], fv[1]))
                .collect::<Vec<(f64, f64)>>(),
            scale_rank: entries[2].parse::<i32>().expect("Failed to parse scale rank"),
            min_zoom: entries[3].parse::<f32>().expect("Failed to parse min zoom")
        }
    }
}