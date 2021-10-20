use crate::board::defs::*;
use crate::board::Move;

#[inline(always)]
pub fn can_move(piece: Pieces) -> bool {
    // Special pieces are all 7 or above (saves multiple checks)
    (piece as u8) < 7
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

const fn build_y_arr() -> [usize; 72] {
    let mut end = [0; 72];
    let mut cntr = 0;

    loop {
        end[cntr] = cntr / 6;

        cntr += 1;
        if cntr >= 72 {
            break;
        }
    }

    end
}

pub const X_ARR: [usize; 72] = build_x_arr();
pub const Y_ARR: [usize; 72] = build_y_arr();

#[inline]
pub fn x_pos_fast(x: usize) -> usize {
    unsafe { *X_ARR.get_unchecked(x) }
}

#[inline]
pub fn y_pos_fast(x: usize) -> usize {
    unsafe { *Y_ARR.get_unchecked(x) }
}

pub const PUFFER: [(u64, u16); 72] = build_puffers();

macro_rules! apply_to_pair {
    ($pair:expr, $pos:expr) => {
        if $pos >= 64 {
            $pair.1 |= 1 << ($pos - 64);
        } else {
            $pair.0 |= 1 << $pos;
        }
    };
}

pub const fn build_puffers() -> [(u64, u16); 72] {
    let mut end = [(0, 0); 72];
    let mut pos = 0;

    loop {
        let x = X_ARR[pos];
        let y = Y_ARR[pos];

        let mut pair = (0, 0);

        apply_to_pair!(pair, pos);

        let up = y > 0;
        let down = y < 11;
        let right = x < 5;
        let left = x > 0;

        if up {
            apply_to_pair!(pair, pos - 6);
        }
        if down {
            apply_to_pair!(pair, pos + 6);
        }
        if left {
            apply_to_pair!(pair, pos - 1);
        }
        if right {
            apply_to_pair!(pair, pos + 1);
        }

        if up && right {
            apply_to_pair!(pair, pos - 5);
        }
        if up && left {
            apply_to_pair!(pair, pos - 7);
        }
        if down && right {
            apply_to_pair!(pair, pos + 7);
        }
        if down && left {
            apply_to_pair!(pair, pos + 5);
        }

        end[pos] = pair;

        pos += 1;
        if pos >= 72 {
            break;
        }
    }

    end
}

#[inline]
pub fn move_to_dani_move(movement: usize) -> usize {
    let x = x_pos_fast(movement);
    let y = 12 - y_pos_fast(movement);

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
