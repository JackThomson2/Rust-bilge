pub mod alt_search;
pub mod defs;
pub mod searcher;

use seahash;

#[macro_use]
pub mod helpers;

use colored::*;
use defs::Pieces::{self, *};
use helpers::can_move;

use arrayvec::ArrayVec;
use rand::Rng;

#[derive(Copy, Clone, Debug)]
pub struct Move {
    x: usize,
    y: usize,
}

#[derive(Clone, Copy)]
pub struct GameState {
    board: [defs::Pieces; 6 * 12],
    water_level: usize,
    to_clear: [usize; 72],
    clear_count: usize,
    pub something_cleared: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct SearchResult {
    move_id: usize,
    score: i16,
}

#[inline]
pub fn int_to_move(move_num: usize) -> Move {
    Move {
        y: (move_num - 1) / 5,
        x: (move_num - 1) % 5,
    }
}

#[inline]
pub fn int_to_mover(move_num: usize) -> Move {
    Move {
        y: (move_num) / 6,
        x: (move_num) % 6,
    }
}

#[inline]
pub fn move_to_int(move_num: &Move) -> usize {
    move_num.x + (move_num.y * defs::WIDTH as usize)
}

impl GameState {
    pub fn as_dani_string(&self) -> String {
        self.board
            .iter()
            .map(|pce| (*pce as u8) - 1)
            .map(|st| st.to_string())
            .collect()
    }

    pub fn draw(&self) {
        println!();

        for y in (0..12).rev() {
            for x in 0..6 {
                let pos = (y * 6) + x;
                let piece = &self.board[pos];

                if y <= self.water_level {
                    print!("{}", defs::draw_piece(piece).blue())
                } else {
                    print!("{}", defs::draw_piece(piece));
                }
            }
            println!();
        }
        println!();
    }

    #[inline]
    fn remove_clears(&mut self) {
        if self.clear_count == 0 {
            return;
        }
        for count in 0..self.clear_count - 1 {
            unsafe {
                let loc = self.to_clear.get_unchecked(count);
                self.board[*loc] = CLEARED;
            }
        }
        self.clear_count = 0;
    }

    #[inline]
    fn jelly(&mut self, clearing: defs::Pieces) {
        for (loc, _pce) in self
            .board
            .iter()
            .enumerate()
            .filter(|(_loc, pce)| *pce == &clearing)
        {
            unsafe {
                *self.to_clear.get_unchecked_mut(self.clear_count) = loc;
            }
            self.clear_count += 1;
        }
    }

    #[inline]
    fn push_to_clear(&mut self, loc: usize) {
        unsafe {
            *self.to_clear.get_unchecked_mut(self.clear_count) = loc;
        }
        self.clear_count += 1;
    }

    #[inline]
    fn puff(&mut self, pos: usize) {
        let x = x_pos!(pos);

        let up = pos > 6;
        let down = pos < 66;
        let right = x < 5;
        let left = x > 0;

        self.push_to_clear(pos);

        if up {
            self.push_to_clear(pos - 6);
        }
        if down {
            self.push_to_clear(pos + 6);
        }
        if left {
            self.push_to_clear(pos - 1);
        }
        if right {
            self.push_to_clear(pos + 1);
        }

        if up && right {
            self.push_to_clear(pos - 5);
        }
        if up && left {
            self.push_to_clear(pos - 7);
        }
        if down && right {
            self.push_to_clear(pos + 7);
        }
        if down && left {
            self.push_to_clear(pos + 5);
        }
    }

    #[inline]
    pub fn hash_board(&self) -> u64 {
        let checking: &[u8; 72] = unsafe { std::mem::transmute(&self.board) };
        seahash::hash(checking)
    }

    #[inline]
    pub fn swap(&mut self, pos: usize) -> f32 {
        self.something_cleared = false;
        if pos >= 71 || x_pos!(pos) == 5 {
            return -9001.0;
        }

        let one = unsafe { *self.board.get_unchecked(pos) };
        let two = unsafe { *self.board.get_unchecked(pos + 1) };

        if one == CLEARED || two == CLEARED {
            return -20001.0;
        } else if one == two {
            return -30001.0;
        } else if one == CRAB {
            return -9001.0;
        } else if one == PUFFERFISH || two == PUFFERFISH {
            if one == PUFFERFISH {
                self.puff(pos);
            } else {
                self.puff(pos + 1);
            }

            self.remove_clears();
            self.shift_everything();
            self.something_cleared = true
        } else if two == CRAB {
            return -90001.0;
        } else if one == JELLYFISH || two == JELLYFISH {
            if one == JELLYFISH {
                self.jelly(one);
            } else {
                self.jelly(two);
            }

            self.remove_clears();
            self.shift_everything();
            self.something_cleared = true
        } else {
            self.board[pos] = two;
            self.board[pos + 1] = one;

            let score = self.get_combo(pos) as f32;
            if score > 0.0 {
                self.clean_board()
            }
            return score;
        }

        if self.something_cleared {
            self.clean_board()
        }

        0.0
    }

    #[inline]
    pub fn get_moves(&self) -> ArrayVec<[usize; 64]> {
        let mut move_vec: ArrayVec<[usize; 64]> = ArrayVec::new();

        for (pos, pieces) in self.board.iter().enumerate() {
            if x_pos!(pos) == 5 {
                continue;
            }

            let left = *pieces;
            if left == CLEARED || left == NULL {
                continue;
            }

            let right = unsafe { *self.board.get_unchecked(pos + 1) };
            if right == CLEARED || right == NULL || right == left {
                continue;
            }
            unsafe { move_vec.push_unchecked(pos) };
        }

        move_vec
    }

    #[inline]
    pub fn clean_board(&mut self) {
        while self.mark_clears() {
            self.remove_clears();
            self.shift_everything();
        }
    }

    #[inline]
    fn mark_clears(&mut self) -> bool {
        let mut returning = false;

        for (pos, piece) in self.board.iter().enumerate() {
            let piece = *piece;
            let x = x_pos!(pos);
            let y = y_pos!(pos);

            let board_size = self.board.len();

            if y > self.water_level && piece == CRAB {
                unsafe {
                    *self.to_clear.get_unchecked_mut(self.clear_count) = pos;
                }
                self.clear_count += 1;
                returning = true;
                continue;
            }

            if !can_move(piece) {
                continue;
            }

            unsafe {
                if x < 4
                    && pos < board_size - 2
                    && piece == *self.board.get_unchecked(pos + 1)
                    && piece == *self.board.get_unchecked(pos + 2)
                {
                    *self.to_clear.get_unchecked_mut(self.clear_count) = pos;
                    *self.to_clear.get_unchecked_mut(self.clear_count + 1) = pos;
                    *self.to_clear.get_unchecked_mut(self.clear_count + 2) = pos;
                    self.clear_count += 3;

                    returning = true;
                }

                if pos < board_size - 12
                    && piece == *self.board.get_unchecked(pos + 6)
                    && piece == *self.board.get_unchecked(pos + 12)
                {
                    *self.to_clear.get_unchecked_mut(self.clear_count) = pos;
                    *self.to_clear.get_unchecked_mut(self.clear_count + 6) = pos;
                    *self.to_clear.get_unchecked_mut(self.clear_count + 12) = pos;
                    self.clear_count += 3;

                    returning = true;
                }
            }
        }

        returning
    }

    #[inline]
    fn shift_everything(&mut self) {
        for x in 0..6 {
            let mut last = 99999;
            for y in 0..12 {
                let pos = (y * 6) + x;
                unsafe {
                    let checking = *self.board.get_unchecked(pos);
                    if checking == CLEARED && last == 99999 {
                        last = y;
                    }

                    if last != 99999 && checking != CLEARED {
                        let last_pos = (last * 6) + x;
                        *self.board.get_unchecked_mut(last_pos) = checking;
                        *self.board.get_unchecked_mut(pos) = CLEARED;
                        last += 1;
                    }
                }
            }
        }
    }

    #[inline]
    fn get_combo(&self, pos: usize) -> i32 {
        let x = x_pos!(pos);

        let left_piece = unsafe { self.board.get_unchecked(pos) };
        let right_piece = unsafe { self.board.get_unchecked(pos + 1) };

        let mut left = 1; //left 3 pieces
        let mut l_col = 1; //left column of 5 pieces
        let mut right = 1; //right 3 pieces
        let mut r_col = 1; //left column of 5 pieces

        unsafe {
            if pos > 2
                && self.board.get_unchecked(pos - 1) == left_piece
                && self.board.get_unchecked(pos - 2) == left_piece
            {
                left = 3;
            }
            if x < 3
                && self.board.get_unchecked(pos + 2) == right_piece
                && self.board.get_unchecked(pos + 3) == right_piece
            {
                right = 3;
            }
            if pos > 5 && self.board.get_unchecked(pos - 6) == left_piece {
                l_col += 1;
                if pos > 11 && self.board.get_unchecked(pos - 12) == left_piece {
                    l_col += 1;
                }
            }
            if pos < 66 && self.board.get_unchecked(pos + 6) == left_piece {
                l_col += 1;
                if pos < 60 && self.board.get_unchecked(pos + 12) == left_piece {
                    l_col += 1;
                }
            }
            if pos > 4 && self.board.get_unchecked(pos - 5) == right_piece {
                r_col += 1;
                if pos > 10 && self.board.get_unchecked(pos - 11) == right_piece {
                    r_col += 1;
                }
            }
            if pos < 65 && self.board.get_unchecked(pos + 7) == right_piece {
                r_col += 1;
                if pos < 59 && self.board.get_unchecked(pos + 13) == right_piece {
                    r_col += 1;
                }
            }
        }

        if r_col < 3 {
            r_col = 0;
        }
        if l_col < 3 {
            l_col = 0;
        }
        if left < 3 {
            left = 0;
        }
        if right < 3 {
            right = 0;
        }

        let mut mult_ct = 0;
        if left == 3 {
            mult_ct += 1;
        }
        if right == 3 {
            mult_ct += 1;
        }

        if l_col >= 3 {
            mult_ct += 1;
        }
        if r_col >= 3 {
            mult_ct += 1;
        }

        // Nothing broke
        if mult_ct == 0 {
            return 0;
        }

        (row_score!(left) + row_score!(right) + row_score!(l_col) + row_score!(r_col)) * mult_ct
    }
}

pub fn generate_rand_board() -> GameState {
    let mut board = [defs::Pieces::CLEARED; 6 * 12];
    let mut rng = rand::thread_rng();

    let mut last: Option<Pieces> = None;
    for x in board.iter_mut() {
        if let Some(pce) = last {
            let mut to_use = defs::piece_from_num(rng.gen_range(1, 7));
            while to_use == pce {
                to_use = defs::piece_from_num(rng.gen_range(1, 7));
            }
            last = Some(to_use);
            *x = to_use;
            continue;
        }
        *x = defs::piece_from_num(rng.gen_range(1, 7));
        last = Some(*x);
    }

    GameState {
        water_level: 3,
        board,
        to_clear: [0; 72],
        clear_count: 0,
        something_cleared: false,
    }
}

pub fn copy_board(copying: &GameState) -> GameState {
    *copying
}

pub fn board_from_array(board: [defs::Pieces; 6 * 12]) -> GameState {
    GameState {
        water_level: 3,
        board,
        to_clear: [0; 72],
        clear_count: 0,
        something_cleared: false,
    }
}

pub fn board_from_str(in_str: &str, water_level: usize) -> GameState {
    let mut board = [defs::Pieces::NULL; 72];
    let brd = defs::str_to_enum(in_str);
    board.copy_from_slice(&brd[..]);

    GameState {
        water_level,
        board,
        to_clear: [0; 72],
        clear_count: 0,
        something_cleared: false,
    }
}

pub fn generate_game() -> GameState {
    GameState {
        water_level: 3,
        board: [defs::Pieces::CLEARED; 6 * 12],
        to_clear: [0; 72],
        clear_count: 0,
        something_cleared: false,
    }
}
