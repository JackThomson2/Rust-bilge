use crate::board::GameState;
use atomic_counter;
use atomic_counter::AtomicCounter;
use std::sync::Arc;
use std::thread::spawn;

#[derive(Debug)]
pub struct Info {
    pub turn: usize,
    score: i32,
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
    let mut copy = board.clone();
    cntr.inc();

    let score = copy.swap(move_number);

    if depth == 1 {
        let scorz = 10 * copy.get_best_combo();
        return Info {
            turn: move_number,
            score: score + scorz,
        };
    }
    let mut moves = moves;
    let mut min_move = min_move;

    if score < 0 {
        return Info {
            turn: move_number,
            score,
        };
    }

    if !copy.something_cleared {
        if moves >= 3 {
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
        moves += 1;
        min_move = std::cmp::min(min_move, move_number as u8);
    } else {
        moves = 0;
        min_move = std::u8::MAX;
    }

    let possible_moves = copy.get_moves();
    let max_score = possible_moves
        .iter()
        .filter(|x| **x >= 10 && **x < 47)
        .map(|i| dani_search(&copy, depth - 1, 72 - i, moves, min_move, cntr).score)
        .max()
        .unwrap();

    Info {
        turn: move_number,
        score: (score as f32 + (max_score as f32) * 0.9) as i32,
    }
}

pub fn find_best_move(board: &GameState) -> Info {
    // println!("Finding best move");
    let depth = 3;

    let possible_moves = board.get_moves();
    let cntr = Arc::new(atomic_counter::RelaxedCounter::new(0));

    let mut best_scoring = Info {
        score: std::i32::MIN,
        turn: 0,
    };

    let mut children = Vec::with_capacity(possible_moves.len());

    for testing in possible_moves {
        let test_board = board.clone();
        let cnt = cntr.clone();

        children.push(spawn(move || {
            dani_search(&test_board.clone(), depth, testing, -1, std::u8::MAX, &cnt)
        }));
    }

    for child in children {
        let res = child.join().unwrap();

        if res.score > best_scoring.score {
            best_scoring = res;
        }
    }

    /*println!(
        "Best move at depth {} found {:#?} num of calcs {}",
        depth,
        best_scoring,
        cntr.get()
    );*/

    best_scoring
}
