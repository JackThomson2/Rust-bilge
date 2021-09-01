use crate::board::defs::*;
use crate::board::Move;

#[inline(always)]
pub fn can_move(piece: Pieces) -> bool {
    // Special pieces are all 7 or above (saves multiple checks)
    (piece as u8) < 7
}

macro_rules! x_pos {
    ($x:expr) => {
        $x % 6
    };
}

macro_rules! y_pos {
    ($x:expr) => {
        $x / 6
    };
}

const fn build_x_arr() -> [usize; 72] {
    let mut end = [0; 72];

    let mut cntr = 0;

    loop {
        end[cntr] = cntr % 6;

        cntr += 1;
        if cntr >= 72 {
            break;
        }
    }

    end
}

pub const X_ARR: [usize; 72] = build_x_arr();

#[inline]
pub fn x_pos_fast(x: usize) -> usize {
    unsafe { *X_ARR.get_unchecked(x) }
}

#[inline]
pub fn move_to_dani_move(movement: usize) -> usize {
    let x = x_pos_fast(movement);
    let y = 12 - y_pos!(movement);

    (y * 5) + x
}

#[inline]
pub fn int_to_move(move_num: usize) -> Move {
    Move {
        y: (move_num - 1) / 5,
        x: (move_num - 1) % 5,
    }
}

#[inline(always)]
pub fn int_to_mover(move_num: usize) -> Move {
    Move {
        y: (move_num) / 6,
        x: (move_num) % 6,
    }
}

#[inline(always)]
pub fn move_to_int(move_num: &Move) -> usize {
    move_num.x + (move_num.y * WIDTH as usize)
}

macro_rules! promote_scorers {
    ($x:expr) => {
        match $x {
            2 => 2,
            3 => 100,
            4 => 10000,
            _ => $x,
        }
    };
}

macro_rules! row_score {
    ($x:expr) => {
        match $x {
            3 => 3,
            4 => 5,
            5 => 7,
            _ => 0,
        }
    };
}
