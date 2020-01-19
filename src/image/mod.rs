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
    for ox in 0..haystack.width() {
        'outer: for oy in 0..haystack.height() {
            for ix in 0..needle.width() {
                for iy in 0..needle.height() {
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
