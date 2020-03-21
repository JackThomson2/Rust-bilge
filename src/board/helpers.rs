use crate::board::defs::Pieces::{self, *};

#[inline(always)]
pub fn can_move(piece: Pieces) -> bool {
    piece != CRAB && piece != JELLYFISH && piece != PUFFERFISH && piece != CLEARED
} 