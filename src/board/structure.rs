use crate::board::defs::*;
use std::hash::Hasher;

pub type Board = [Pieces; 6 * 12];

#[derive(Clone, Copy, Hash)]
pub struct GameState {
    pub board: Board,
    pub water_level: u8,
    pub to_clear_l: u64,
    pub to_clear_r: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct Move {
    pub x: usize,
    pub y: usize,
}

#[inline(always)]
pub fn make_hash(brd: &[u8; 72], depth: u8) -> u64 {
    let mut hashing = ahash::AHasher::default();

    hashing.write(brd);
    hashing.write_u8(depth);

    hashing.finish()
}

impl PartialEq for GameState {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.board == other.board
    }
}
impl Eq for GameState {}

#[derive(Debug, Copy, Clone)]
pub struct SearchResult {
    pub move_id: usize,
    pub score: i16,
}
