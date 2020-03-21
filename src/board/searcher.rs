use crate::board::GameState;
use std::collections::HashSet;

use atomic_counter;
use atomic_counter::AtomicCounter;
use rand::prelude::*;
use std::sync::{Arc};
use std::{cmp, thread::spawn};

#[derive(Debug)]
pub struct Info {
    turn: usize,
    score: i32,
}

#[inline]
fn dani_search(
    board: &GameState,
    depth: u8,
    move_number: usize,
    moves: &mut HashSet<usize>,
    rng: &mut ThreadRng,
    cntr: &atomic_counter::RelaxedCounter,
) -> Info {
    let mut max_score = 0i32;
    let mut copy = board.clone();
    cntr.inc();

    let score = copy.swap(move_number);

    if score < 0 {
        return Info {
            turn: move_number,
            score,
        };
    }

    if !copy.something_cleared {
        if moves.len() >= 5 {
            return Info {
                turn: move_number,
                score,
            };
        };

        if moves.is_empty() {
            for i in moves.iter() {
                if i >= &move_number {
                    return Info {
                        turn: move_number,
                        score,
                    };
                }
            }
        }

        moves.insert(move_number);
    } else {
        moves.clear();
    }

    if depth == 1 {
        let scorz = 10 * copy.get_best_combo();
        return Info {
            turn: move_number,
            score: score + scorz,
        };
    }

    let offset = rng.gen_range(0, 15);

    for i in 1 + offset..=40 + offset {
        let score = dani_search(&copy, depth - 1, i, moves, rng, cntr).score;
        max_score = cmp::max(score, max_score);
    }

    Info {
        turn: move_number,
        score: (score as f32 + (max_score as f32) * 0.9) as i32,
    }
}

pub fn find_best_move(board: &GameState) -> Info {
    println!("Finding best move");
    let depth = 9;

    let possible_moves = board.get_moves();
    let cntr = Arc::new(atomic_counter::RelaxedCounter::new(0));

    println!("there are {} moves", possible_moves.len());

    let mut best_scoring = Info {
        score: std::i32::MIN,
        turn: 0,
    };

    let mut children = Vec::with_capacity(possible_moves.len());

    for testing in possible_moves {
        let test_board = board.clone();
        let cnt = cntr.clone();

        children.push(spawn(move || {
            dani_search(
                &test_board.clone(),
                depth,
                testing,
                &mut HashSet::new(),
                &mut rand::thread_rng(),
                &cnt,
            )
        }));
    }

    for child in children {
        let res = child.join().unwrap();

        if res.score > best_scoring.score {
            best_scoring = res;
        }
    }

    println!(
        "Best move at depth {} found {:#?} num of calcs {}",
        depth,
        best_scoring,
        cntr.get()
    );

    best_scoring
}
