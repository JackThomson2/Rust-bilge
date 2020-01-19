pub(crate) mod defs;

use colored::*;
use defs::Pieces::*;

use std::hash::{Hash, Hasher};
use std::sync::Arc;

use colored::*;

use fasthash::MetroHasher;

use rand::Rng;

pub struct HashEntry {
    search_res: SearchResult,
    depth: i8,
}

#[derive(Clone)]
pub struct GameState {
    board: [defs::Pieces; 72],
    water_level: usize,
    to_clear: [bool; 72],
    something_cleared: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct SearchResult {
    move_id: usize,
    score: i16,
}

impl GameState {
    pub fn draw(&self) {
        let mut cntr = 0;

        for y in (0..11).rev() {
            for x in 0..6 {
                let piece = self.board[x + (y * 6)];

                if y <= self.water_level {
                    print!("{}", defs::draw_piece(&piece).blue())
                } else {
                    print!("{}", defs::draw_piece(&piece));
                }
            }
            println!();
        }
    }

    fn remove_clears(&mut self) {
        for i in 0..self.to_clear.len()  {
            if self.to_clear[i] {
                self.board[i] = defs::Pieces::CLEARED;
                self.to_clear[i] = false;
            }
        }
    }

    fn jelly(&mut self, clearing: defs::Pieces) {
        for i in 0..self.board.len() {
            if self.board[i] == clearing {
                self.to_clear[i] = true
            }
        }
    }

    fn puff(&mut self, pos: usize) {
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

    fn swap(&mut self, pos: usize) -> i32 {
        if pos % 5 == 0 {
            return -9001;
        }

        let mut score = 0;

        let one = self.board[pos];
        let two = self.board[pos + 1];

        self.something_cleared = false;

        if one == CLEARED || two == CLEARED {
            return -20001;
        } else if one == two {
            return -30001;
        } else if one == CRAB {
            return -9001;
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
            return -90001;
        } else if one == JELLYFISH || two == JELLYFISH {
            if one == JELLYFISH {
                self.jelly(self.board[pos])
            } else {
                self.jelly(self.board[pos + 1])
            }

            self.remove_clears();
            self.shift_everything();
            self.something_cleared = true
        } else {
            self.board.swap(pos, pos + 1);
            score = 10 * self.get_combo(pos);
            if score > 0 {
                self.something_cleared = true
            }
        }

        if self.something_cleared {
            self.clean_board()
        }

        score
    }

    pub fn get_moves(&self) -> Vec<usize> {
        let mut moves = Vec::new();

        for i in 0..self.board.len() {
            if i % 5 == 0 {
                continue;
            }

            if self.board[i] == CLEARED || self.board[i] == NULL {
                continue;
            }

            if self.board[i + 1] == CLEARED || self.board[i + 1] == NULL {
                continue;
            }

            if self.board[i] != self.board[i + 1] {
                moves.push(i)
            }
        }

        return moves;
    }

    pub fn clean_board(&mut self) {
        while self.mark_clears() {
            self.remove_clears();
            self.shift_everything();
        }
    }

    fn mark_clears(&mut self) -> bool {
        let mut returning = false;

        for i in 0..self.board.len() - 1 {
            let x = i % 6;
            let y = i / 6 as usize;

            let piece = self.board[i];

            if y > self.water_level && piece == CRAB {
                self.to_clear[i] = true;
                returning = true;
                continue;
            }

            if piece == PUFFERFISH || piece == CRAB || piece == JELLYFISH || piece == CLEARED {
                continue;
            }

            if x < 4 && piece == self.board[i + 1] && piece == self.board[i + 2] {
                self.to_clear[i] = true;
                self.to_clear[i + 1] = true;
                self.to_clear[i + 2] = true;


                let x2 = (i + 1) % 6;
                let x3 = (i + 2) % 6;

                println!("We got a row at {},{},{} y: {}", x, x2, x3, y);

                returning = true;
            }

            if y < 10 && piece == self.board[i + 6] && piece == self.board[i + 12] {
                self.to_clear[i] = true;
                self.to_clear[i + 6] = true;
                self.to_clear[i + 12] = true;

                returning = true;
            }
        }

        println!();

        returning
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

                if piece == CLEARED && last == 9999 {
                    last = y;
                }

                if last != 9999 && piece != CLEARED && piece != NULL {
                    self.board[x + (last * 6)] = self.board[x + (y * 6)];
                    self.board[x + (y * 6)] = CLEARED;
                    last += 1;
                }
            }
        }
    }

    pub fn get_best_combo(&self) -> i32 {
        let mut max = 0;

        for y in 0..(60 / 5) - 1 {
            for x in 0..5 {
                let left_piece = self.board[x + (y * 6)];
                let right_piece = self.board[x + (y * 6) + 1];

                if left_piece != CRAB
                    && left_piece != JELLYFISH
                    && left_piece != PUFFERFISH
                    && left_piece != CLEARED
                    && right_piece != CRAB
                    && right_piece != JELLYFISH
                    && right_piece != PUFFERFISH
                    && right_piece != CLEARED
                {
                    let combo = self.get_combo(x + (y * 6));
                    if combo > max {
                        max = combo
                    }
                }
            }
        }

        max
    }

    fn get_combo(&self, pos: usize) -> i32 {
        let x = pos % 6;
        let y = pos / 6 as usize;

        let left_piece = self.board[pos];
        let right_piece = self.board[pos + 1];

        let mut left = 1; //left 3 pieces
        let mut l_col = 1; //left column of 5 pieces
        let mut right = 1; //right 3 pieces
        let mut r_col = 1; //left column of 5 pieces

        if x > 1 && self.board[pos - 1] == left_piece && self.board[pos - 2] == left_piece {
            left = 3;
        }
        if x < 3 && self.board[pos + 2] == right_piece && self.board[pos + 3] == right_piece {
            right = 3;
        }
        if y > 0 && self.board[pos - 6] == left_piece {
            l_col += 1;
            if y > 1 && self.board[pos - 12] == left_piece {
                l_col += 1;
            }
        }
        if y < 11 && self.board[y + 6] == left_piece {
            l_col += 1;
            if y < 10 && self.board[y + 12] == left_piece {
                l_col += 1;
            }
        }
        if y > 0 && self.board[pos - 5] == right_piece {
            r_col += 1;
            if y > 1 && self.board[y + 11] == right_piece {
                r_col += 1;
            }
        }
        if y < 11 && self.board[pos + 7] == right_piece {
            r_col += 1;
            if y < 10 && self.board[pos + 13] == right_piece {
                r_col += 1;
            }
        }
        if r_col < 3 {
            r_col = 1;
        }
        if l_col < 3 {
            l_col = 1;
        }
        if left == right && l_col == left && r_col == left && left == 1 {
            return 0;
        }

        let mut mult_ct = 1;
        let mut ret = 0;
        if left == 3 {
            mult_ct += 1;
        }
        if right == 3 {
            mult_ct += 1;
        }
        if l_col == 3 {
            mult_ct += 1;
        }
        if r_col == 3 {
            mult_ct += 1;
        }
        if mult_ct == 1 {
            return 0;
        }
        if mult_ct == 2 {
            return 2;
        }
        if mult_ct == 3 {
            return 25;
        }
        if mult_ct == 4 {
            return 100;
        }
        if mult_ct == 5 {
            ret = 400;
        }
        if mult_ct == 5 && (r_col == 5 || l_col == 5) {
            return 9001;
        }

        ret
    }
}

pub fn generate_rand_board() -> GameState {
    let mut board = [defs::Pieces::CLEARED; 72];
    let mut rng = rand::thread_rng();

    for i in board.iter_mut() {
        *i = defs::piece_from_num(&rng.gen_range(1, 7))
    }

    GameState {
        water_level: 3,
        board,
        to_clear: [false; 72],
        something_cleared: false,
    }
}

pub fn copy_board(copying: &GameState) -> GameState {
    copying.clone()
}

pub fn board_from_array(board: [defs::Pieces; 72]) -> GameState {
    GameState {
        water_level: 3,
        board,
        to_clear: [false; 72],
        something_cleared: false
    }
}

pub fn generate_game() -> GameState {
    GameState {
        water_level: 3,
        board: [defs::Pieces::CLEARED; 72],
        to_clear: [false; 72],
        something_cleared: false,
    }
}
