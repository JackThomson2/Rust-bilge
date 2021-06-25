use crate::board::defs::*;
use std::hash::Hasher;

pub type Board = [Pieces; 6 * 12];

#[thread_local]
pub static mut TO_CLEAR: [usize; 72] = [0; 72];

#[thread_local]
pub static mut CLEAR_COUNT: usize = 0;

#[derive(Clone, Copy, Hash)]
pub struct GameState {
    pub board: Board,
    pub water_level: u8,
}

#[derive(Copy, Clone, Debug)]
pub struct Move {
    pub x: usize,
    pub y: usize,
}

#[inline(always)]
pub fn set_to_clear(new_value: usize) {
    unsafe {
        *TO_CLEAR.get_unchecked_mut(CLEAR_COUNT) = new_value;
        CLEAR_COUNT += 1;
    }
}

#[inline(always)]
pub fn reset_clears() {
    unsafe {
        CLEAR_COUNT = 0;
    }
}

#[inline(always)]
pub fn clear_count() -> usize {
    unsafe { CLEAR_COUNT }
}

#[inline(always)]
pub fn get_position(index: usize) -> usize {
    unsafe { *TO_CLEAR.get_unchecked(index) }
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
