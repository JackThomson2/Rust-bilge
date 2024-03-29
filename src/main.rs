#![feature(thread_local)]
#![feature(core_intrinsics)]
#![feature(stdsimd)]

#[macro_use]
mod macros;

pub mod board;

use bilge::config::TEST_BOARD;
use board::helpers::move_to_dani_move;

use std::env;
use std::time::Instant;

use ahash::RandomState;

use board::searcher::HashTable;

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;
//static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let args: Vec<String> = env::args().collect();
    let random = RandomState::new();

    let mut hash_table: HashTable = dashmap::DashMap::with_capacity_and_hasher(40_000_000, random);

    let arg_count = args.len();

    if arg_count == 4 {
        if args[1].len() != 72 {
            println!("We need a string of 72 length, this was {}", args[1].len());
            return;
        }

        let water_level = u8::from_str_radix(&args[3], 10).unwrap();
        let depth = u8::from_str_radix(&args[2], 10).unwrap();

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
    } else if arg_count == 2 || arg_count == 1 {
        // Benchmarking mode
        if arg_count == 1 || args[1] == "bench" {
            bench(&mut hash_table);
            return;
        }

        let mut input = String::with_capacity(200);

        while let Ok(_read) = std::io::stdin().read_line(&mut input) {
            let len = input.trim_end_matches(&['\r', '\n'][..]).len();
            input.truncate(len);

            let commands: Vec<&str> = input.split(' ').collect();

            if commands.len() < 3 {
                println!("Not enough parameters");
                continue;
            }

            let water_level = u8::from_str_radix(commands[2], 10).unwrap();
            let depth = u8::from_str_radix(commands[1], 10).unwrap();

            let now = Instant::now();
            let game = board::board_from_str(&commands[0], water_level);
            let best_moves = board::searcher::find_best_move_list(&game, depth, false, &hash_table);
            let best_move = best_moves.turns.get(0);

            if best_move.is_none() {
                println!("Couldnt find any moves");
                continue;
            }

            let best_move = best_move.unwrap();

            let dani_move = move_to_dani_move(best_move.turn);
            println!(
                "{} {} ran at depth {}, {} took {:?}",
                dani_move,
                best_move.score,
                depth,
                best_moves.info_str,
                now.elapsed()
            );

            input = String::with_capacity(200);

            let random = RandomState::new();
            hash_table = dashmap::DashMap::with_capacity_and_hasher(40_000_000, random);
        }
    }
}

fn bench(map: &mut HashTable) {
    let game = board::board_from_str(TEST_BOARD, 3);

    game.draw_highlight(39);

    let run_count = 10;
    let mut average = 0;
    
    for i in 0..run_count {
        let now = Instant::now();
        let _best_moves = board::searcher::find_best_move_list(&game, 7, false, map);
        let time_taken = now.elapsed();

        println!(
            "Run {} took {:?} best move {:?}, hashtable size {}",
            i + 1,
            time_taken,
            _best_moves.turns.get(0).unwrap(),
            map.len()
        );

        let random = RandomState::new();
        *map = dashmap::DashMap::with_capacity_and_hasher(40_000_000, random);

        average += time_taken.as_millis();
    }

    println!("Took an average of {}ms", average / 10);
}
