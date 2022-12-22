use crate::board::GameState;

use super::redundant_move_filter::*;

use crate::board::Board;
use crate::macros::SafeGetters;
use dashmap::DashMap;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::intrinsics::likely;
use std::sync::Arc;

use super::helpers::{x_pos_fast, y_pos_fast};
use ahash::RandomState;

use super::defs::{CLEARED, CRAB, NULL};

const DROP_PER_TURN: f32 = 0.95;

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

pub type HashTable = DashMap<Board, HashEntry, RandomState>;

pub const NULL_MOVE: Info = Info {
    turn: 0,
    score: 0.0,
};

#[inline]
fn search(
    mut copy: GameState,
    depth: u8,
    total_depth: u8,
    move_number: usize,
    hasher: &HashTable,
    prev_move_map: u64
) -> f32 {
    //cntr.inc();
    debug_assert!(y_pos_fast(move_number) == y_pos_fast(move_number + 1));

    let mut score = copy.swap(move_number);
    let hash_table_range = depth > 2;

    if likely(hash_table_range) {
        let found = hasher.get(&copy.board);

        if let Some(entry) = found {
            if entry.depth >= depth {
                return entry.score;
            }
        }
    }

    if score < 0.0 || depth == 1 {
        return score;
    }

    let new_mask = record_move(total_depth, depth, prev_move_map, move_number, score > 0.0); 

    let greater_than_three = depth > 2;
    
    let (base, end) = if greater_than_three {
        (6, 66)
    } else {
        (12usize, 56usize)
    };

    let range = base..end;

    let filter = |pos: usize| {
        let x_p = x_pos_fast(pos);

        let valid_col = if greater_than_three {
            x_p < 5
        } else {
            x_p < 4 && x_p > 1
        };

        if !valid_col {
            return None;
        }

        let left = *copy.board.get_safely(pos);
        if left == CLEARED || left == NULL || left == CRAB {
            return None;
        }

        let right = *copy.board.get_safely(pos + 1);
        if right == CLEARED || right == NULL || right == CRAB || right == left {
            return None;
        }

        // Prevent making the same move again if nothing broke
        if score == 0.0 && pos == move_number {
            return None;
        }

        if check_if_previously_run(pos, prev_move_map) {
            return None;
        }

        debug_assert!(y_pos_fast(pos) == y_pos_fast(pos + 1));

        Some(pos)
    };

    let max_score = if depth > 3 {
        range.into_par_iter().filter_map(filter)
            .map(|i| search(copy, depth - 1, depth, i, hasher, new_mask))
            .max_by(|x, y| unsafe { x.partial_cmp(y).unwrap_unchecked() })
            .unwrap_or(0.0)
    } else {
        range
            .filter_map(filter)
            .map(|i| search(copy, depth - 1, depth, i, hasher, new_mask))
            .max_by(|x, y| unsafe { x.partial_cmp(y).unwrap_unchecked() } )
            .unwrap_or(0.0)
    };

    score += max_score as f32 * DROP_PER_TURN;

    if likely(hash_table_range) {
        hasher.insert(copy.board, HashEntry { score, depth });
    }

    score
}

#[derive(Copy, Clone)]
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

    let mut best_move: Vec<Info> = possible_moves
        .par_iter()
        .map(|testing| Info {
            turn: *testing,
            score: search(*board, depth, depth, *testing, hash_table, 0),
        })
        .collect();

    best_move.sort_unstable_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));


    TurnList {
        turns: best_move,
        info_str: format!(
            "Done"),
    }
}
