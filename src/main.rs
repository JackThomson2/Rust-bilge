pub mod board;
pub mod image;

use std::time::Instant;

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

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
