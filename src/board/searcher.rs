use crate::board::make_hash;
use crate::board::GameState;

use atomic_counter::AtomicCounter;
use dashmap::DashMap;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::sync::Arc;

use ahash::RandomState;

use super::defs::{CLEARED, CRAB, NULL};

const DROP_PER_TURN: f32 = 0.9;

#[derive(Debug, Copy, Clone)]
pub struct Info {
    pub turn: usize,
    pub score: f32,
}

#[derive(Debug)]
pub struct TurnInfo {
    pub turn: usize,
    pub score: f32,
    pub info_str: String,
}

pub struct TurnList {
    pub turns: Vec<Info>,
    pub info_str: String,
}

pub type HashTable = DashMap<u64, f32, RandomState>;

pub const NULL_MOVE: Info = Info {
    turn: 0,
    score: 0.0,
};

#[inline]
fn search(
    mut copy: GameState,
    max_depth: u8,
    depth: u8,
    move_number: usize,
    cntr: &atomic_counter::RelaxedCounter,
    hasher: &HashTable,
    hash_hits: &atomic_counter::RelaxedCounter,
) -> f32 {
    //cntr.inc();
    let score = copy.swap(move_number);
    let actual_depth = (max_depth - depth) + 1;

    let hash_table_range = depth > 1;
    let mut hashed = 0;

    if hash_table_range {
        hashed = make_hash(&copy.board, depth);
        if let Some(found) = hasher.get(&hashed) {
            // hash_hits.inc();
            return *found;
        }
    }

    if score < 0.0 || depth == 1 {
        return score;
    }

    let (base, end) = if actual_depth > 3 {
        (12usize, 48usize)
    } else {
        (6, 60)
    };

    let range = base..end;

    let filtered: arrayvec::ArrayVec<[usize; 54]> = range
        .filter_map(|pos| {
            let x_p = x_pos!(pos);
            if x_p == 5 {
                return None;
            }

            let left = unsafe { *copy.board.get_unchecked(pos) };
            if left == CLEARED || left == NULL || left == CRAB {
                return None;
            }

            let right = unsafe { *copy.board.get_unchecked(pos + 1) };
            if right == CLEARED || right == NULL || right == CRAB || right == left {
                return None;
            }

            // Prevent making the same move again if nothing broke
            if score == 0.0 && pos == move_number {
                return None;
            }

            let valid_col = if actual_depth > 3 {
                x_p < 4 && x_p > 1
            } else {
                x_p < 5 && x_p > 0
            };

            if valid_col {
                return Some(pos);
            } else {
                return None;
            }
        })
        .collect();

    let max_score = if depth > 2 {
        filtered
            .par_iter()
            .map(|i| search(copy, max_depth, depth - 1, *i, &cntr, &hasher, &hash_hits))
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(Ordering::Equal))
            .unwrap_or(0.0)
    } else {
        filtered
            .iter()
            .map(|i| search(copy, max_depth, depth - 1, *i, &cntr, &hasher, &hash_hits))
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(Ordering::Equal))
            .unwrap_or(0.0)
    };

    let score = score + (max_score * DROP_PER_TURN);

    if hash_table_range {
        hasher.insert(hashed, score);
    }

    score
}

#[derive(Debug, Copy, Clone)]
pub struct HashEntry {
    score: f32,
    depth: u8,
}

#[inline]
pub fn find_best_move(
    board: &GameState,
    depth: u8,
    verbose: bool,
    hash_table: &HashTable,
) -> TurnInfo {
    let move_list = find_best_move_list(board, depth, verbose, hash_table);
    let best_move = move_list.turns.get(0).unwrap();

    let info_str = format!(
        "{}, best move {} with score {}",
        move_list.info_str, best_move.turn, best_move.score
    );

    TurnInfo {
        turn: best_move.turn,
        score: best_move.score,
        info_str,
    }
}

#[inline]
pub fn find_best_move_list(
    board: &GameState,
    depth: u8,
    verbose: bool,
    hash_table: &HashTable,
) -> TurnList {
    let possible_moves = board.get_moves();
    let cntr = Arc::new(atomic_counter::RelaxedCounter::new(0));
    let hash_hits = Arc::new(atomic_counter::RelaxedCounter::new(0));

    let mut best_move: Vec<Info> = possible_moves
        .par_iter()
        .map(|testing| Info {
            turn: *testing,
            score: search(
                *board,
                depth,
                depth,
                *testing,
                &cntr,
                &hash_table,
                &hash_hits,
            ),
        })
        .collect();

    best_move.sort_unstable_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));

    if verbose {
        println!(
            "Searched {} positions, {} hash hits",
            cntr.get(),
            hash_hits.get(),
        );
    }

    TurnList {
        turns: best_move,
        info_str: format!(
            "Searched {} positions, {} hash hits.",
            cntr.get(),
            hash_hits.get()
        ),
    }
}
