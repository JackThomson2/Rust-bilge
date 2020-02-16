mod board;
mod image;

use std::time::Instant;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let mut searcher = image::ImageCapture::new();
    let now = Instant::now();

    searcher.load_test_image();

    let mut game = board::generate_rand_board();
    game.draw();
    game.clean_board();
    game.draw();

    println!("Best Move {}", game.get_best_combo());

    println!("{:?}", now.elapsed());
}
