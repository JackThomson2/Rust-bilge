mod colours;

#[path = "../board/mod.rs"]
mod board;

use board::defs::Pieces;
use board::defs::Pieces::*;

use scrap::{Capturer, Display};

use std::io::ErrorKind::WouldBlock;
use std::time::{Duration, Instant};
use std::borrow::Borrow;
use std::thread;

use image::{GenericImageView, DynamicImage, Rgba};
use colours::*;

fn load_needle() -> DynamicImage {
    image::open("test/TopLeft.png").unwrap()
}

struct Location {
    found: bool,
    x: u32,
    y: u32
}

fn find_needle(haystack: &DynamicImage, needle: &DynamicImage) -> Location {
    let needle_width = needle.width();
    let needle_height = needle.height();

    for ox in 0..haystack.width() - needle_width {
        'outer: for oy in needle_height..haystack.height() {
            for ix in 0..needle_width {
                for iy in 0..needle_height {
                    if haystack.get_pixel(ox + ix, oy + iy) !=  needle.get_pixel(ix, iy) {
                        continue 'outer;
                    }
                }
            }

            // Image matches
            return Location { found: true, x: ox, y: oy}
        }
    }

    Location { found: false, x: 0,  y:0}
}

pub struct ImageCapture {
    screen: Capturer,
    needle: DynamicImage,
    screen_width: usize,
    screen_height: usize
}

impl ImageCapture {
    pub fn new() -> ImageCapture {
        let display = Display::primary().expect("Couldn't find primary display.");
        let mut capture = Capturer::new(display).expect("Couldn't begin capture.");
        let screen_width = capture.width();
        let screen_height = capture.height();

        ImageCapture {
            screen: capture,
            needle: load_needle(),
            screen_width,
            screen_height
        }
    }

    pub fn load_test_image(&mut self) {
        let img = image::open("test/testimage.png").unwrap();

        let now = Instant::now();

        self.take_screenshot();

        println!("Screeny took {:?}", now.elapsed());

        let image_loc = find_needle(&img, &self.needle);

        if !image_loc.found {
            println!("Couldn't find image");
            return;
        }

        let read_x = image_loc.x + 29;
        let read_y = image_loc.y + 29;

        let mut board = [NULL; 72];

        for y in 0..12 {
            for x in 0..6 {
                let pixel = img.get_pixel(read_x + (x * 45), read_y + ((11 - y) * 45));
                board[(x + (y * 6)) as usize] = get_piece_from_pixel(pixel);
            }
        }

        let game = board::board_from_array(board);

        println!("{:?}", now.elapsed());

        game.draw();
    }

    pub fn take_screenshot(&mut self) -> Vec<Rgba<u8>> {
        let mut bitflipped:Vec<Rgba<u8>> = Vec::with_capacity(self.screen_width * self.screen_height * 4);

        loop {
            let buffer = match self.screen.frame() {
                Ok(buffer) => buffer,
                Err(error) => {
                    if error.kind() == WouldBlock {
                        continue;
                    } else {
                        panic!("Error: {}", error);
                    }
                }
            };

            let stride = buffer.len() / self.screen_height;

            for y in 0..self.screen_height {
                for x in 0..self.screen_width {
                    let i = stride * y + 4 * x;
                    bitflipped.push(Rgba([
                        buffer[i],
                        buffer[i + 1],
                        buffer[i + 2],
                        255,
                    ]));
                }
            }
            return bitflipped;
        }
    }
}


fn get_piece_from_pixel(pixel: Rgba<u8>) -> Pieces {
    match pixel {
        WAVY_COLOUR => WavySquare,
        WAVY_COLOUR_UW => WavySquare,

        BREEN_COLOUR => BreenOctagon,
        BREEN_COLOUR_UW => BreenOctagon,

        BLUE_COLOUR => BlueCircle,
        BLUE_COLOUR_UW => BlueCircle,

        PALE_COLOUR => PaleCircle,
        PALE_COLOUR_UW => PaleCircle,

        DB_COLOUR => DarkBlueSquare,
        DB_COLOUR_UW => DarkBlueSquare,

        GREEN_COLOUR => GreenSquare,
        GREEN_COLOUR_UW => GreenSquare,

        PENT_COLOUR => BluePentagon,
        PENT_COLOUR_UW => BluePentagon,

        PUFFER_COLOUR => PUFFERFISH,
        PUFFER_COLOUR_UW => PUFFERFISH,

        CRAB_COLOUR => CRAB,

        _ => NULL
    }
}
