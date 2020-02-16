mod board;
mod image;

use std::time::Instant;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let mut searcher = image::ImageCapture::new();
    let now = Instant::now();

    searcher.load_test_image();

    //println!("Best move is {}", board::search_board(game));

    println!("{:?}", now.elapsed());
}
