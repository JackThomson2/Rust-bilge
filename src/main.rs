mod board;
use dashmap::DashMap;

use num_cpus;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() {
    let now = Instant::now();

    let mut game = board::generate_rand_board();

    game.clear_board();

    //println!("Best move is {}", board::search_board(game));

    println!("{:?}", now.elapsed());
}
