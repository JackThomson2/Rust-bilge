use image::GenericImageView;

struct Location {
    found: bool,
    x: u32,
    y: u32
}

fn LoadNeedle() -> Image::GenericImageView {
    image::open("tests/images/jpg/progressive/cat.jpg").unwrap()
}

fn FindNeedle(haystack: &dyn GenericImageView, needle: &dyn GenericImageView) -> Location {
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


}
