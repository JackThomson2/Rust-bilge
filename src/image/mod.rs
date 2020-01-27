mod colours;

#[path = "../board/mod.rs"]
mod board;

use board::defs::Pieces;
use board::defs::Pieces::*;

use scrap::{Capturer, Display};

use std::fs::File;
use std::io::ErrorKind::WouldBlock;

use repng;

use png;

use colours::*;

struct Info {
    buffer: Vec<u8>,
    height: u32,
    width: u32,
}

fn load_needle() -> Info {
    let decoder = png::Decoder::new(File::open("test/TopLeft.png").unwrap());
    let (info, mut reader) = decoder.read_info().unwrap();
    // Allocate the output buffer.
    let mut buffer = vec![0; info.buffer_size()];
    // Read the next frame. Currently this function should only called once.
    // The default options
    reader.next_frame(&mut buffer).unwrap();

    println!("info {:?}", info.color_type);

    Info {
        buffer,
        height: info.height,
        width: info.width,
    }
}

struct Location {
    found: bool,
    x: u32,
    y: u32,
}

pub struct ImageCapture {
    screen: Capturer,
    needle: Info,
    screen_width: usize,
    screen_height: usize,
}

impl ImageCapture {
    pub fn new() -> ImageCapture {
        let display = Display::primary().expect("Couldn't find primary display.");
        let capture = Capturer::new(display).expect("Couldn't begin capture.");
        let screen_width = capture.width();
        let screen_height = capture.height();

        ImageCapture {
            screen: capture,
            needle: load_needle(),
            screen_width,
            screen_height,
        }
    }

    fn find_needle(&self, haystack: &Vec<u8>, needle: &Info) -> Location {
        let needle_width = needle.width as usize;
        let needle_height = needle.height as usize;

        println!(
            "Haystack w/h {},{}  screen w/h {},{}",
            needle_width, needle_height, self.screen_width, self.screen_height
        );

        for ox in 0..self.screen_width - needle_width {
            'outer: for oy in needle_height..self.screen_height {
                for iy in 0..needle_height {
                    let pos = ((ox) + ((oy + iy) * self.screen_width)) * 4;
                    let needle_pos = (iy * needle_width) * 4;

                    for cntr in 0..needle_width * 4 {
                        if haystack[pos + cntr] != needle.buffer[needle_pos + cntr] {
                            continue 'outer;
                        }
                    }

                    //println!("Matching r fou");
                    //println!("{:?}, {:?}", screen_slice, needle_slice);
                }

                // Image matches
                return Location {
                    found: true,
                    x: ox as u32,
                    y: oy as u32,
                };
            }
        }

        Location {
            found: false,
            x: 0,
            y: 0,
        }
    }

    pub fn load_test_image(&mut self) {
        let haystack = self.take_screenshot();

        let image_loc = self.find_needle(&haystack, &self.needle);

        if !image_loc.found {
            println!("Couldn't find image");
            return;
        }

        return; /*
                let read_x = image_loc.x + 29;
                let read_y = image_loc.y + 29;

                let mut board = [NULL; 72];

                for y in 0..12 {
                    for x in 0..6 {
                        let pixel = img.get_pixel(read_x + (x * 45), read_y + ((11 - y) * 45));
                        board[(x + (y * 6)) as usize] = get_piece_from_pixel(pixel);
                    }
                }

                board::board_from_array(board); */
    }

    pub fn take_screenshot(&mut self) -> Vec<u8> {
        let mut bitflipped: Vec<u8> =
            Vec::with_capacity(self.screen_width * self.screen_height * 4);
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
                    let i = (stride * y) + (4 * x);
                    bitflipped.extend_from_slice(&[buffer[i + 2], buffer[i + 1], buffer[i], 255]);
                }
            }

            return bitflipped;
        }
    }
}

fn get_piece_from_pixel(pixel: Vec<u8>) -> Pieces {
    let pixel_array: [u8; 4] = [
        pixel.get(0).unwrap().clone(),
        pixel.get(1).unwrap().clone(),
        pixel.get(2).unwrap().clone(),
        pixel.get(3).unwrap().clone(),
    ];

    match pixel_array {
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

        _ => NULL,
    }
}
