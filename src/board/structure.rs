use crate::board::defs::*;

#[derive(Copy, Clone, Debug)]
pub struct Move {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Copy)]
pub struct GameState {
    pub board: [Pieces; 6 * 12],
    pub water_level: usize,
    pub to_clear: [usize; 72],
    pub clear_count: usize,
    pub something_cleared: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct SearchResult {
    pub move_id: usize,
    pub score: i16,
}
