mod defs;

use dashmap::DashMap;

use num_cpus;

use std::sync::{Arc, Mutex};

use rand::Rng;

use std::hash::{Hash, Hasher};
use std::thread;

use fasthash::MetroHasher;

pub type PeerMap = Arc<dashmap::DashMap<u64, HashEntry>>;

pub struct HashEntry {
    search_res: SearchResult,
    depth: i8,
}

#[derive(Clone)]
pub struct GameState {
    board: [defs::Pieces; 72],
    water_level: i8,
    to_clear: [bool; 72],
}

#[derive(Debug, Copy, Clone)]
pub struct SearchResult {
    move_id: usize,
    score: i16,
}

impl GameState {
    pub fn draw(&self) {
        let mut cntr = 0;
        for i in self.board.iter() {
            print!("{}", defs::draw_piece(i));
            cntr += 1;
            if defs::WIDTH == cntr {
                println!();
                cntr = 0;
            }
        }
    }

    pub fn jelly(&mut self, clearing: defs::Pieces) {
        for i in 0..self.board.len() - 1 {
            if (self.board[i] == clearing) {
                self.to_clear[i] = true
            }
        }
    }

    pub fn puff(&mut self, pos: usize) {
        let x = pos % 6;
        let y = pos / 6 as usize;

        let up = y > 0;
        let down = y < 11;
        let right = x < 5;
        let left = x > 0;

        self.to_clear[pos] = true;

        if up {
            self.to_clear[pos - 6] = true;
        }
        if down {
            self.to_clear[pos + 6] = true;
        }
        if left {
            self.to_clear[pos - 1] = true;
        }
        if right {
            self.to_clear[pos + 1] = true;
        }

        if up && right {
            self.to_clear[pos - 5] = true;
        }
        if up && left {
            self.to_clear[pos - 7] = true;
        }
        if down && right {
            self.to_clear[pos + 7] = true;
        }
        if down && left {
            self.to_clear[pos + 5] = true;
        }
    }

    pub fn swap(&mut self, pos: usize) {
        if pos % 5 == 0 {
            return;
        }

        self.board.swap(pos, pos + 1);
    }

    pub fn get_moves(&self) -> Vec<usize> {
        let mut moves = Vec::new();

        for i in 0..self.board.len() - 1 {
            if i % 5 == 0 {
                continue;
            }

            if self.board[i] == defs::Pieces::CLEARED || self.board[i] == defs::Pieces::NULL {
                continue;
            }

            if self.board[i + 1] == defs::Pieces::CLEARED || self.board[i + 1] == defs::Pieces::NULL
            {
                continue;
            }

            if self.board[i] != self.board[i + 1] {
                moves.push(i)
            }
        }

        return moves;
    }

    fn hash_me(&self) -> u64 {
        let mut s = MetroHasher::default();
        self.board.hash(&mut s);
        s.finish()
    }

    fn shift_everything(&mut self) {
        for x in 0..6 {
            let mut last = 9999;
            for y in 0..12 {
                let piece = self.board[x + (y * 6)];

                if piece == defs::Pieces::CLEARED && last == 9999 {
                    last = y;
                }

                if last != 9999 && piece != defs::Pieces::CLEARED && piece != defs::Pieces::NULL {
                    self.board[x + (last * 6)] = piece;
                    self.board[x + (y * 6)] = defs::Pieces::CLEARED;
                    last += 1;
                }
            }
        }
    }

    pub fn clear_board(&mut self) -> i16 {
        let mut clears = 0;
        let mut new_board = self.board.clone();
        let mut broken = false;

        for i in 0..self.board.len() {
            let piece = self.board[i];
            if piece == defs::Pieces::CLEARED || piece == defs::Pieces::NULL {
                continue;
            }

            let x = i % 6;
            let y = i / 6 as usize;

            if x > 1 && piece == self.board[i - 1] && piece == self.board[i - 2] {
                clears += 1;
                new_board[i] = defs::Pieces::CLEARED;
                new_board[i - 1] = defs::Pieces::CLEARED;
                new_board[i - 2] = defs::Pieces::CLEARED;

                broken = true;
            }

            if x < 4 && piece == self.board[i + 1] && piece == self.board[i + 2] {
                clears += 1;
                new_board[i] = defs::Pieces::CLEARED;
                new_board[i + 1] = defs::Pieces::CLEARED;
                new_board[i + 2] = defs::Pieces::CLEARED;

                broken = true;
            }

            if y > 1 && piece == self.board[i - 6] && piece == self.board[i - 12] {
                clears += 1;
                new_board[i] = defs::Pieces::CLEARED;
                new_board[i - 6] = defs::Pieces::CLEARED;
                new_board[i - 12] = defs::Pieces::CLEARED;

                broken = true;
            }

            if y < 4 && piece == self.board[i + 6] && piece == self.board[i + 12] {
                clears += 1;
                new_board[i] = defs::Pieces::CLEARED;
                new_board[i + 6] = defs::Pieces::CLEARED;
                new_board[i + 12] = defs::Pieces::CLEARED;

                broken = true;
            }
        }

        self.board = new_board;

        if broken {
            self.shift_everything();
        }

        clears
    }
}

pub fn copy_board(copying: &GameState) -> GameState {
    copying.clone()
}

pub fn generate_game() -> GameState {
    GameState {
        water_level: 3,
        board: [defs::Pieces::CLEARED; 72],
        to_clear: [false; 72],
    }
}
