use crate::board::GameState;
use rayon::prelude::*;

#[derive(Debug)]
pub struct Info {
  pub turn: usize,
  score: i32,
}

#[inline]
fn dani_search(board: &GameState, depth: u8, move_number: usize, moves: i8, min_move: u8) -> Info {
  let mut copy = board.clone();
  let score = copy.swap(move_number);

  if score < 0 {
    return Info {
      turn: move_number,
      score,
    };
  }

  if depth == 1 {
    let scorz = 10 * copy.get_best_combo();
    return Info {
      turn: move_number,
      score: (score as f32 + (scorz as f32) * 0.9) as i32,
    };
  }

  if !copy.something_cleared {
    if moves >= 3 {
      return Info {
        turn: move_number,
        score,
      };
    };

    if moves >= 0 && min_move <= move_number as u8 {
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
      .filter(|x| **x >= 8 && **x < 55)
      .map(|i| dani_search(&copy, depth - 1, 72 - i, moves, min_move).score)
      .max()
      .unwrap()
  } else {
    possible_moves
      .iter()
      .filter(|x| **x >= 8 && **x < 55)
      .map(|i| dani_search(&copy, depth - 1, 72 - i, moves, min_move).score)
      .max()
      .unwrap()
  };

  Info {
    turn: move_number,
    score: (score as f32 + (max_score as f32) * 0.9) as i32,
  }
}

pub fn find_best_move(board: &GameState, depth: u8) -> Info {
  // println!("Finding best move");
  //let cntr = Arc::new(atomic_counter::RelaxedCounter::new(0));

  let possible_moves = board.get_moves();

  possible_moves
    .par_iter()
    .map(|testing| dani_search(&board, depth, *testing, -1, std::u8::MIN))
    .max_by_key(|res| res.score)
    .unwrap()
}
