use image::Rgba;

pub const BLACK: Rgba<u8> = Rgba([0, 0, 0, 255]);
pub const RED: Rgba<u8> = Rgba([255, 0, 0, 255]);
pub const GREEN: Rgba<u8> = Rgba([0, 255, 0, 255]);
pub const BLUE: Rgba<u8> = Rgba([0, 0, 255, 255]);

pub const COLOUR_AIRCRAFT: Rgba<u8> = GREEN;
pub const COLOUR_SELECTION: [f32; 4] = [152.0/255.0, 250.0/255.0, 161.0/255.0, 0.5];
pub const COLOUR_SELECTED_OBJECT: [f32; 4] = [255.0/255.0, 235.0/255.0, 133.0/255.0, 0.5];
pub const COLOUR_STATUS_AREA_BACK: [f32; 4] = [0.0/255.0, 0.0/255.0, 0.0/255.0, 1.0];
pub const COLOUR_STATUS_AREA_OUTLINE: [f32; 4] = [31.0/255.0, 102.0/255.0, 50.0/255.0, 0.75];
pub const COLOUR_STATUS_AREA_TEXT: [f32; 4] = [126.0/255.0, 214.0/255.0, 135.0/255.0, 1.0];
