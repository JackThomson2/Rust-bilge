use crate::board::GameState;
use atomic_counter::AtomicCounter;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::sync::Arc;

#[derive(Debug)]
pub struct Info {
    pub turn: usize,
    pub score: f32,
}

#[inline]
fn dani_search(
    board: &GameState,
    depth: u8,
    move_number: usize,
    moves: i8,
    min_move: u8,
    cntr: &atomic_counter::RelaxedCounter,
) -> Info {
    let mut copy = *board;
    cntr.inc();
    let score = copy.swap(move_number);

    if score < 0.0 {
        return Info {
            turn: move_number,
            score,
        };
    }

    if depth == 1 {
        return Info {
            turn: move_number,
            score,
        };
    }

    if !copy.something_cleared {
        if moves >= 6 {
            return Info {
                turn: move_number,
                score,
            };
        };

        if moves >= 0 && min_move >= move_number as u8 {
            return Info {
                turn: move_number,
                score,
            };
        }
    }

    let (moves, min_move) = if !copy.something_cleared {
        (moves + 1, std::cmp::max(min_move, move_number as u8))
    } else {
        (0, std::u8::MIN)
    };

    let possible_moves = copy.get_moves();

    let max_score = if depth > 2 {
        possible_moves
            .par_iter()
            .filter(|x| **x >= 6 && **x < 66)
            .map(|i| dani_search(&copy, depth - 1, *i, moves, min_move, &cntr).score)
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(Ordering::Equal))
            .unwrap()
    } else {
        possible_moves
            .iter()
            .filter(|x| **x >= 6 && **x < 66)
            .map(|i| dani_search(&copy, depth - 1, *i, moves, min_move, &cntr).score)
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(Ordering::Equal))
            .unwrap()
    };

    Info {
        turn: move_number,
        score: (score as f32 + (max_score as f32) * 0.9),
    }
}

pub fn find_best_move(board: &GameState, depth: u8) -> Info {
    // println!("Finding best move");
    //let cntr = Arc::new(atomic_counter::RelaxedCounter::new(0));

    let possible_moves = board.get_moves();
    let cntr = Arc::new(atomic_counter::RelaxedCounter::new(0));

    let best_move = possible_moves
        .par_iter()
        .map(|testing| dani_search(&board, depth, *testing, 0, std::u8::MIN, &cntr))
        .max_by(|x, y| x.score.partial_cmp(&y.score).unwrap_or(Ordering::Equal))
        .unwrap();

    println!("Searched {} positions", cntr.get());

    best_move
}
