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