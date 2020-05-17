pub mod board;
use board::helpers::move_to_dani_move;

use std::env;
use std::time::Instant;

use board::searcher::HashTable;

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

fn main() {
    let args: Vec<String> = env::args().collect();
    let hash_table: HashTable = dashmap::DashMap::new();

    if args.len() == 1 {
        let game = board::generate_rand_board();
        game.draw();

        let now = Instant::now();
        let moving = board::searcher::find_best_move_list(&game, 6, true, &hash_table);

        println!("Turns {:?}", &moving.turns[..]);

        let moving = moving.turns.get(0).unwrap();
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

        let water_level = usize::from_str_radix(&args[3], 10).unwrap();
        let depth = u8::from_str_radix(&args[2], 10).unwrap();

        let game = board::board_from_str(&args[1], water_level);
        let best_move = board::searcher::find_best_move(&game, depth, false, &hash_table);
        let dani_move = move_to_dani_move(best_move.turn);
        println!(
            "{} {} ran at depth {}, {}",
            dani_move, best_move.score, depth, best_move.info_str
        )
    } else if args.len() == 2 {
        let mut input = String::new();
        let mut last_board = 0;

        while let Ok(_read) = std::io::stdin().read_line(&mut input) {
            let len = input.trim_end_matches(&['\r', '\n'][..]).len();
            input.truncate(len);

            let commands: Vec<&str> = input.split(' ').collect();

            if commands.len() < 3 {
                println!("Not enough parameters");
                continue;
            }

            let water_level = usize::from_str_radix(&commands[2], 10).unwrap();
            let depth = u8::from_str_radix(&commands[1], 10).unwrap();

            let game = board::board_from_str(&commands[0], water_level);
            let best_moves = board::searcher::find_best_move_list(&game, depth, false, &hash_table);
            let mut best_move = None;

            let mut saved_double_move = false;

            for mov in best_moves.turns {
                let mut copy = game;
                copy.swap(mov.turn);

                if copy.hash_board() != last_board {
                    best_move = Some(mov);
                    break;
                } else {
                    saved_double_move = true;
                }
            }

            if best_move.is_none() {
                println!("Couldnt find any moves");
                continue;
            }

            let best_move = best_move.unwrap();

            let dani_move = move_to_dani_move(best_move.turn);
            println!(
                "{} {} ran at depth {}, {}. Double move stopped {}",
                dani_move, best_move.score, depth, best_moves.info_str, saved_double_move
            );

            last_board = game.hash_board();
            input = String::new();
            hash_table.clear();
        }
    }
}
