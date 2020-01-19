mod board;
mod image;

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

    let mut searcher = image::ImageCapture::new();
    let now = Instant::now();


    searcher.load_test_image();

    //println!("Best move is {}", board::search_board(game));

    println!("{:?}", now.elapsed());
}
