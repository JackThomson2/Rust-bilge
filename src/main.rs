pub mod board;
use board::helpers::move_to_dani_move;

use std::env;
use std::time::Instant;

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        let game = board::generate_rand_board();
        game.draw();

        let now = Instant::now();
        let moving = board::alt_search::find_best_move(&game, 5, true);

        let dani_move = move_to_dani_move(moving.turn);

        println!(
            "Finding best move took {:?} as Dani move {}",
            now.elapsed(),
            dani_move
        );

        game.draw_highlight(moving.turn);
    } else if args.len() == 4 {
        if args[1].len() != 72 {
            println!("We need a string of 72 length, this was {}", args[1].len());
            return;
        }

        let water_level = usize::from_str_radix(&args[2], 10).unwrap();
        let depth = u8::from_str_radix(&args[2], 10).unwrap();

        let game = board::board_from_str(&args[1], water_level);
        let best_move = board::alt_search::find_best_move(&game, depth, false);
        let dani_move = move_to_dani_move(best_move.turn);
        println!(
            "{} {} ran at depth {}, {}",
            dani_move, best_move.score, depth, best_move.info_str
        )
    }
}
