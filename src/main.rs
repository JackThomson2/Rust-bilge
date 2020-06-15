pub mod auth;
pub mod board;
pub mod config;

use auth::get_serial_number;
use board::helpers::move_to_dani_move;
use config::{MAX_DEPTH, TEST_BOARD};

use std::env;
use std::time::Instant;

use board::searcher::HashTable;

#[cfg(feature = "custom-alloc")]
#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut hash_table: HashTable = dashmap::DashMap::with_capacity(20_000_000);

    let authorised = match get_serial_number() {
        Ok(true) => true,
        _ => false,
    };

    //auth::get_serial_number().unwrap();

    if args.len() == 1 {
        if authorised {
            println!("You are authenticated!");
        } else {
            println!("You are not authorised!");
        }

        println!("Press any key to close...");
        let mut input = String::new();
        let _s = std::io::stdin().read_line(&mut input);
    } else if args.len() == 4 {
        if !authorised {
            return;
        }

        if args[1].len() != 72 {
            println!("We need a string of 72 length, this was {}", args[1].len());
            return;
        }

        let water_level = usize::from_str_radix(&args[3], 10).unwrap();
        let depth = u8::from_str_radix(&args[2], 10).unwrap();
        let depth = std::cmp::min(depth, MAX_DEPTH);

        let now = Instant::now();
        let game = board::board_from_str(&args[1], water_level);
        let best_move = board::searcher::find_best_move(&game, depth, false, &hash_table);
        let dani_move = move_to_dani_move(best_move.turn);

        println!(
            "{} {} ran at depth {}, {}, it took {:?}",
            dani_move,
            best_move.score,
            depth,
            best_move.info_str,
            now.elapsed()
        )
    } else if args.len() == 2 {
        // Benchmarking mode
        if args[1] == "bench" {
            bench(&hash_table);
            return;
        }

        if !authorised {
            return;
        }

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
            let depth = std::cmp::min(depth, MAX_DEPTH);

            let now = Instant::now();
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
                "{} {} ran at depth {}, {}. Double move stopped {} took {:?}",
                dani_move,
                best_move.score,
                depth,
                best_moves.info_str,
                saved_double_move,
                now.elapsed()
            );

            last_board = game.hash_board();
            input = String::new();
            hash_table = dashmap::DashMap::with_capacity(20_000_000);
        }
    }
}

fn bench(map: &HashTable) {
    let game = board::board_from_str(TEST_BOARD, 3);

    let run_count = 10;
    let mut average = 0;

    for i in 0..run_count {
        let now = Instant::now();
        let _best_moves = board::searcher::find_best_move_list(&game, 6, false, map);
        let time_taken = now.elapsed();

        println!("Run {} took {:?}", i + 1, time_taken);
        map.clear();

        average += time_taken.subsec_millis();
    }

    println!("Took an average of {}ms", average / 10);
}
