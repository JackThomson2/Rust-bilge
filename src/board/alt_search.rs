use crate::board::GameState;
use atomic_counter::AtomicCounter;
use dashmap::DashMap;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::sync::Arc;

#[derive(Debug)]
pub struct Info {
    pub turn: usize,
    pub score: f32,
}

type HashTable = DashMap<u64, HashEntry>;

#[inline]
fn search(
    board: &GameState,
    max_depth: u8,
    depth: u8,
    move_number: usize,
    moves: i8,
    min_move: u8,
    cntr: &atomic_counter::RelaxedCounter,
    hasher: &HashTable,
    hash_hits: &atomic_counter::RelaxedCounter,
) -> Info {
    let moves_made = ((max_depth - depth) + 1) as f32;
    let mut copy = *board;
    cntr.inc();
    let score = copy.swap(move_number);

    let mut board_hash = None;
    if max_depth - depth <= 1 {
        let hash = copy.hash_board();
        board_hash = Some(hash);
        if let Some(found) = hasher.get(&hash) {
            if found.depth >= depth {
                hash_hits.inc();
                return Info {
                    turn: move_number,
                    score: found.score / moves_made,
                };
            }
        }
    }

    if score < 0.0 || depth == 1 {
        return Info {
            turn: move_number,
            score: score / moves_made,
        };
    }

    if !copy.something_cleared {
        if moves >= 6 || (moves >= 0 && min_move >= move_number as u8) {
            return Info {
                turn: move_number,
                score: score / moves_made,
            };
        }
    }

    let (moves, min_move) = if !copy.something_cleared {
        (moves + 1, std::cmp::max(min_move, move_number as u8))
    } else {
        (0, std::u8::MIN)
    };

    let possible_moves = copy.get_moves();

    let vert_range = 30;

    let upper_range = std::cmp::min(71, move_number + vert_range);
    let lower_range = if (move_number as isize - vert_range as isize) < 0 {
        0
    } else {
        move_number - vert_range
    };

    let max_score = if depth > 2 {
        possible_moves
            .par_iter()
            .filter(|x| **x >= lower_range && **x < upper_range)
            .map(|i| {
                search(
                    &copy,
                    max_depth,
                    depth - 1,
                    *i,
                    moves,
                    min_move,
                    &cntr,
                    &hasher,
                    &hash_hits,
                )
                .score
            })
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(Ordering::Equal))
            .unwrap()
    } else {
        possible_moves
            .iter()
            .filter(|x| **x >= lower_range && **x < upper_range)
            .map(|i| {
                search(
                    &copy,
                    max_depth,
                    depth - 1,
                    *i,
                    moves,
                    min_move,
                    &cntr,
                    &hasher,
                    &hash_hits,
                )
                .score
            })
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(Ordering::Equal))
            .unwrap()
    };

    if let Some(key) = board_hash {
        if let Some(found) = hasher.get(&key) {
            if found.depth >= depth {
                hash_hits.inc();
                return Info {
                    turn: move_number,
                    score: found.score / moves_made,
                };
            }
        }

        hasher.insert(
            key,
            HashEntry {
                score: score + max_score,
                depth,
            },
        );
    }

    Info {
        turn: move_number,
        score: (score / moves_made) + max_score,
    }
}

struct HashEntry {
    score: f32,
    depth: u8,
}

pub fn find_best_move(board: &GameState, depth: u8) -> Info {
    // println!("Finding best move");

    let hash_table: HashTable = DashMap::new();

    let possible_moves = board.get_moves();
    let cntr = Arc::new(atomic_counter::RelaxedCounter::new(0));
    let hash_hits = Arc::new(atomic_counter::RelaxedCounter::new(0));

    let best_move = possible_moves
        .par_iter()
        .map(|testing| {
            search(
                &board,
                depth,
                depth,
                *testing,
                0,
                std::u8::MIN,
                &cntr,
                &hash_table,
                &hash_hits,
            )
        })
        .max_by(|x, y| x.score.partial_cmp(&y.score).unwrap_or(Ordering::Equal))
        .unwrap();

    println!(
        "Searched {} positions, {} hash hits",
        cntr.get(),
        hash_hits.get()
    );

    best_move
}
