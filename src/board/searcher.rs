use crate::board::GameState;
use atomic_counter::AtomicCounter;
use dashmap::DashMap;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::sync::Arc;

const drop_per_turn: f32 = 0.8;

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

pub type HashTable = DashMap<u64, HashEntry>;

pub const NULL_MOVE: Info = Info {
    turn: 0,
    score: 0.0,
};

#[inline]
fn search(
    board: &GameState,
    max_depth: u8,
    depth: u8,
    move_number: usize,
    cntr: &atomic_counter::RelaxedCounter,
    hasher: &HashTable,
    hash_hits: &atomic_counter::RelaxedCounter,
) -> Info {
    let mut copy = *board;
    cntr.inc();
    let score = copy.swap(move_number);

    let hash_table_range = depth > 1;

    let mut board_hash = None;
    if hash_table_range {
        let hash = copy.hash_board();
        board_hash = Some(hash);
        if let Some(found) = hasher.get(&hash) {
            if found.depth == depth {
                hash_hits.inc();
                return Info {
                    turn: move_number,
                    score: found.score,
                };
            }
        }
    }

    if score < 0.0 || depth == 1 {
        return Info {
            turn: move_number,
            score,
        };
    }

    let possible_moves = copy.get_moves();

    let mv_filter = |x: &&usize| -> bool {
        let x_p = **x % 6;
        let valid_col = if depth <= 4 {
            x_p < 4 && x_p > 1
        } else {
            x_p < 5 && x_p > 0
        };

        if !valid_col {
            return false;
        }

        if depth <= 3 {
            **x >= 12 && **x < 48
        } else {
            **x >= 6 && **x < 60
        }
    };

    let max_score = if depth > 2 {
        possible_moves
            .par_iter()
            .filter(mv_filter)
            .map(|i| search(&copy, max_depth, depth - 1, *i, &cntr, &hasher, &hash_hits).score)
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(Ordering::Equal))
            .unwrap_or(0.0)
    } else {
        possible_moves
            .iter()
            .filter(mv_filter)
            .map(|i| search(&copy, max_depth, depth - 1, *i, &cntr, &hasher, &hash_hits).score)
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(Ordering::Equal))
            .unwrap_or(0.0)
    };

    if let Some(key) = board_hash {
        hasher.insert(
            key,
            HashEntry {
                score: score + (max_score * drop_per_turn),
                depth,
            },
        );
    }

    Info {
        turn: move_number,
        score: (score) + (max_score * drop_per_turn),
    }
}

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
        .map(|testing| {
            search(
                &board,
                depth,
                depth,
                *testing,
                &cntr,
                &hash_table,
                &hash_hits,
            )
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
