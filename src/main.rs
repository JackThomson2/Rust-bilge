pub mod board;
mod image;

use std::time::Instant;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    //let mut searcher = image::ImageCapture::new();

    //searcher.load_test_image();

    let game = board::generate_rand_board();
    game.draw();
    let now = Instant::now();
    board::searcher::find_best_move(&game);

    println!("Finding best move took {:?}", now.elapsed());
    game.draw();
}
