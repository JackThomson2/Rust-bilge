pub mod board;

use std::env;
use std::time::Instant;

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        let game = board::generate_rand_board();
        game.draw();

        for i in 0..10 {
            let now = Instant::now();
            board::alt_search::find_best_move(&game, 5);

            println!("Finding best move took {:?}", now.elapsed());
        }
        game.draw();
    } else if args.len() == 4 {
        if args[1].len() != 72 {
            println!("We need a string of 72 length, this was {}", args[1].len());
            return;
        }

        let water_level = usize::from_str_radix(&args[2], 10).unwrap();
        let depth = u8::from_str_radix(&args[2], 10).unwrap();

        let game = board::board_from_str(&args[1], water_level);
        game.draw();
        let best_move = board::alt_search::find_best_move(&game, depth);
        println!("{} {}", best_move.turn, best_move.score)
    }
}
