mod colours;

use image::{GenericImageView, DynamicImage};
use colours::*;

struct Location {
    found: bool,
    x: u32,
    y: u32
}



fn LoadNeedle() -> DynamicImage {
    image::open("tests/images/jpg/progressive/cat.jpg").unwrap()
}

fn FindNeedle(haystack: &DynamicImage, needle: &DynamicImage) -> Location {
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

pub fn LoadTestImage() {
    let needle = LoadNeedle();
    let img = image::open("tests/images/jpg/progressive/cat.jpg").unwrap();

    let imageLoc = FindNeedle(&img, &needle);

    if !imageLoc.found {
        println!("Couldn't find image");
        return;
    }

    let read_x = imageLoc.x + 29;
    let read_y = imageLoc.y + 29;

    for y in 0..12 {
        for x in 0..6 {
            let pixel = img.get_pixel(read_x + (x * 45), read_y + (y * 45));


        }
    }

}
