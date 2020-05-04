use crate::board::defs::Pieces::{self, *};

#[inline(always)]
pub fn can_move(piece: Pieces) -> bool {
    piece != CRAB && piece != JELLYFISH && piece != PUFFERFISH && piece != CLEARED
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

#[inline]
pub fn move_to_dani_move(movement: usize) -> usize {
    let x = x_pos!(movement);
    let y = 12 - y_pos!(movement);

    (y * 5) + x
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
