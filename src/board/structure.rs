use crate::board::defs::*;
use std::hash::{Hash, Hasher};

pub type Board = [Pieces; 6 * 12];

#[derive(Copy, Clone, Debug)]
pub struct Move {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Copy)]
pub struct GameState {
    pub board: Board,
    pub water_level: usize,
    pub to_clear: [usize; 72],
    pub clear_count: usize,
    pub something_cleared: bool,
}

/*impl PartialEq for GameState {
    fn eq(&self, other: &Self) -> bool {
        println!("I am called?");
        self.board
            .iter()
            .zip(other.board.iter())
            .all(|(a, b)| a == b)
    }
}
impl Eq for GameState {}*/

impl Hash for GameState {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        Hash::hash_slice(&self.board, hasher);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SearchResult {
    pub move_id: usize,
    pub score: i16,
}
