use image::Rgba;

pub const BLACK: Rgba<u8> = Rgba([0, 0, 0, 255]);
pub const RED: Rgba<u8> = Rgba([255, 0, 0, 255]);
pub const GREEN: Rgba<u8> = Rgba([0, 255, 0, 255]);
pub const BLUE: Rgba<u8> = Rgba([0, 0, 255, 255]);

pub const COLOUR_AIRCRAFT: Rgba<u8> = GREEN;
pub const COLOUR_SELECTION: [f32; 4] = [152.0/255.0, 250.0/255.0, 161.0/255.0, 0.5];
