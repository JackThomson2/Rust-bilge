mod colours;

#[path = "../board/mod.rs"]
mod board;

use board::defs::Pieces;
use board::defs::Pieces::*;

use scrap::{Capturer, Display};

use std::io::ErrorKind::WouldBlock;

use colored::*;
use colours::*;

type PixelArray = Vec<Vec<Pixel>>;

struct Info {
    buffer: PixelArray,
    height: usize,
    width: usize,
}

#[inline]
fn load_needle() -> Info {
    let image = lodepng::decode32_file("test/TestBilg.png").unwrap();
    // Allocate the output buffer.
    let buffer = image.buffer;

    let height = image.height as usize;
    let width = image.width as usize;
    let mut pixels: Vec<Vec<Pixel>> = Vec::with_capacity(height);

    let mut row: Vec<Pixel> = Vec::with_capacity(width);

    for y in 0..height {
        row.clear();
        for x in 0..width {
            let i = (width * y) + (x);
            let pix = (buffer[i].r / 3) + (buffer[i].g / 3) + (buffer[i].b / 3);

            row.push(pix);

            print!("{}", " ".on_true_color(pix, pix, pix),);
        }
        pixels.push(row.clone());
        println!();
    }

    Info {
        buffer: pixels,
        height,
        width,
    }
}

pub type Pixel = u8;

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
    #[inline]
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

    #[inline]
    fn find_needle(&self, haystack: &PixelArray, needle: &Info) -> Location {
        let needle_width = needle.width as usize;
        let needle_height = needle.height as usize;

        println!(
            "Haystack w/h {},{} size {}  screen w/h {},{} size {}",
            needle_width,
            needle_height,
            needle.buffer.len(),
            self.screen_width,
            self.screen_height,
            haystack.len()
        );

        let mut checked = 0;

        for oy in 0..self.screen_height {
            'outer: for ox in 0..self.screen_width {
                for iy in 0..needle_height {
                    for ix in 0..needle_width {
                        let x_loc = ox + ix;
                        let y_loc = oy + iy;

                        checked += 1;
                        if haystack[y_loc][x_loc] != needle.buffer[iy][ix] {
                            continue 'outer;
                        }
                    }
                }

                println!("Found a match at loc {},{}", oy, ox);
                // Image matches
                return Location {
                    found: true,
                    x: ox as u32,
                    y: oy as u32,
                };
            }
        }

        println!("Checked {} places", checked);

        Location {
            found: false,
            x: 0,
            y: 0,
        }
    }

    #[inline]
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

    #[inline]
    pub fn take_screenshot(&mut self) -> PixelArray {
        let mut bitflipped: PixelArray = Vec::with_capacity(self.screen_height);
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
            let mut row: Vec<Pixel> = Vec::with_capacity(self.screen_width);

            let lowY = 400;
            let highY = 450;

            let lowX = 800;
            let highX = 900;

            for y in 0..self.screen_height {
                row.clear();
                for x in 0..self.screen_width {
                    let i = (stride * y) + (4 * x);
                    let pix: Pixel = (buffer[i + 2] / 3) + (buffer[i + 1] / 3) + (buffer[i] / 3);
                    row.push(pix);

                    if y > lowY && y < highY && x > lowX && x < highX {
                        print!("{}", " ".on_true_color(pix, pix, pix),);
                    }
                }
                if y < highY + 1 && y > lowY - 1 {
                    println!();
                }
                bitflipped.push(row.clone());
            }

            return bitflipped;
        }
    }
}

fn get_piece_from_pixel(pixel: &Vec<u8>) -> Pieces {
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
