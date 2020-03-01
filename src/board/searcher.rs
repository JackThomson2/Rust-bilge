use crate::board::{int_to_move, int_to_mover, move_to_int, GameState, Move};
use std::collections::HashSet;

use atomic_counter;
use atomic_counter::AtomicCounter;
use rand::prelude::*;
use std::sync::{Arc, Mutex};
use std::{cmp, thread::spawn};

#[derive(Debug)]
pub struct Info {
    turn: Move,
    score: i32,
}

#[inline]
fn dani_search(
    board: &GameState,
    depth: u8,
    move_numer: usize,
    moves: &mut HashSet<usize>,
    rng: &mut ThreadRng,
    cntr: &atomic_counter::RelaxedCounter,
) -> Info {
    let move_num = int_to_mover(move_numer);
    let mut max_score = 0i32;
    let mut copy = board.clone();
    cntr.inc();

    let score = copy.swap(&move_num);

    if score < 0 {
        return Info {
            turn: move_num.clone(),
            score,
        };
    }

    if !copy.something_cleared {
        if moves.len() >= 5 {
            return Info {
                turn: move_num.clone(),
                score,
            };
        };

        if moves.len() > 0 {
            for i in moves.iter() {
                if i >= &move_numer {
                    return Info {
                        turn: move_num.clone(),
                        score,
                    };
                }
            }
        }

        moves.insert(move_numer);
    } else {
        moves.clear();
    }

    if depth == 1 {
        let scorz = 10 * copy.get_best_combo();
        return Info {
            turn: move_num.clone(),
            score: score + scorz,
        };
    }

    let offset = rng.gen_range(0, 15);

    for i in 1 + offset..=40 + offset {
        let score = dani_search(&copy, depth - 1, i, moves, rng, cntr).score;
        max_score = cmp::max(score, max_score);
    }

    Info {
        turn: move_num.clone(),
        score: (score as f32 + (max_score as f32) * 0.9) as i32,
    }
}

pub fn find_best_move(board: &GameState) -> Info {
    println!("Finding best move");
    let depth = 11;

    let possible_moves = board.get_moves();
    let cntr = Arc::new(atomic_counter::RelaxedCounter::new(0));

    println!("there are {} moves", possible_moves.len());

    let mut best_scoring = Info {
        score: std::i32::MIN,
        turn: int_to_move(0),
    };

    let mut children = vec![];

    for testing in possible_moves {
        let test_board = board.clone();
        let test = move_to_int(&testing);
        let cnt = cntr.clone();

        children.push(spawn(move || {
            dani_search(
                &test_board,
                depth,
                test,
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
