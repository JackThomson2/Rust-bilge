use crate::board::defs::*;
use std::hash::{Hash, Hasher};
use ahash::{CallHasher, AHasher, RandomState};

pub type Board = [Pieces; 6 * 12];

#[thread_local]
pub static mut TO_CLEAR: [usize; 72] = [0; 72];

#[thread_local]
pub static mut CLEAR_COUNT: usize = 0;

#[derive(Copy, Clone, Debug)]
pub struct Move {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Copy)]
pub struct GameState {
    pub board: Board,
    pub water_level: usize,
    pub something_cleared: bool,
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
pub fn make_hash(brd: &[u8], depth: u8) -> u64 {
    let mut hashing = AHasher::default();

    hashing.write(brd);
    hashing.write_u8(depth);
    
    hashing.finish()
}
impl PartialEq for GameState {
    fn eq(&self, other: &Self) -> bool {
        self.board == other.board
    }
}
impl Eq for GameState {}

impl Hash for GameState {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.board.get_hash(hasher);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SearchResult {
    pub move_id: usize,
    pub score: i16,
}
