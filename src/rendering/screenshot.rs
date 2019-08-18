extern crate repng;
extern crate scrap;

use scrap::{Capturer, Display};
use std::io::ErrorKind::WouldBlock;
use std::fs::File;
use std::thread;
use std::time::Duration;
use chrono::Utc;
use image::ImageFormat;
use super::BackBuffer;


fn screenshot_filename() -> String {
    format!("image-{}.png", Utc::now().format("%Y%m%d-%H%M%S"))
}

pub fn display_screenshot() {
    let one_second = Duration::new(1, 0);
    let one_frame = one_second / 120;

    let display = Display::primary().expect("Couldn't find primary display.");
    let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");
    let (w, h) = (capturer.width(), capturer.height());

    loop {
        println!("Checking");
        // Wait until there's a frame.
        let buffer = match capturer.frame() {
            Ok(buffer) => buffer,
            Err(error) => {
                if error.kind() == WouldBlock {
                    // Keep spinning.
                    thread::sleep(one_frame);
                    continue;
                } else {
                    panic!("Error: {}", error);
                }
            }
        };

        //println!("Captured! Saving...");

        // Flip the ARGB image into a BGRA image.
        let mut bitflipped = Vec::with_capacity(w * h * 4);
        let stride = buffer.len() / h;

        for y in 0..h {
            for x in 0..w {
                let i = stride * y + 4 * x;
                bitflipped.extend_from_slice(&[
                    buffer[i + 2],
                    buffer[i + 1],
                    buffer[i],
                    255,
                ]);
            }
        }

        // Save the image
        let filename = screenshot_filename();
        repng::encode(
            File::create(&filename).unwrap(),
            w as u32,
            h as u32,
            &bitflipped,
        ).unwrap();

        println!("Image saved to \"{}\"", &filename);
        break;
    }
}

pub fn buffer_screenshot(buffer: &BackBuffer) {
    let filename = screenshot_filename();
    buffer
        .save_with_format(filename.as_str(), ImageFormat::PNG)
        .and_then(|_| {
            println!("Screenshot saved to \"{}\"", filename);
            Ok(())
        })
        .expect("Failed to save screenshot");
}