use crate::board::defs::*;
use crate::board::GameState;

use rand::Rng;

pub fn generate_rand_board() -> GameState {
    let mut board = [CLEARED; 6 * 12];
    let mut rng = rand::thread_rng();

    let mut last: Option<Pieces> = None;
    for x in board.iter_mut() {
        if let Some(pce) = last {
            let mut to_use = piece_from_num(rng.gen_range(1, 7));
            while to_use == pce {
                to_use = piece_from_num(rng.gen_range(1, 7));
            }
            last = Some(to_use);
            *x = to_use;
            continue;
        }
        *x = piece_from_num(rng.gen_range(1, 7));
        last = Some(*x);
    }

    GameState {
        water_level: 3,
        board,
        something_cleared: false,
    }
}

pub fn copy_board(copying: &GameState) -> GameState {
    *copying
}

pub fn board_from_array(board: [Pieces; 6 * 12]) -> GameState {
    GameState {
        water_level: 3,
        board,
        something_cleared: false,
    }
}

pub fn board_from_str(in_str: &str, water_level: usize) -> GameState {
    let mut board = [NULL; 72];
    let brd = str_to_enum(in_str);
    board.copy_from_slice(&brd[..]);

    GameState {
        water_level,
        board,
        something_cleared: false,
    }
}

pub fn generate_game() -> GameState {
    GameState {
        water_level: 3,
        board: [CLEARED; 6 * 12],
        something_cleared: false,
    }
}
