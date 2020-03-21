pub mod defs;
pub mod searcher;

mod helpers;

use helpers::can_move;
use colored::*;
use defs::Pieces::{self, *};

use fasthash::MetroHasher;
use std::hash::{Hash, Hasher};

use rand::Rng;
use arrayvec::{ArrayVec, Array};

pub struct HashEntry {
    search_res: SearchResult,
    depth: i8,
}

#[derive(Copy, Clone, Debug)]
pub struct Move {
    x: usize,
    y: usize,
}

#[derive(Clone)]
pub struct GameState {
    board: [[defs::Pieces; 6]; 12],
    water_level: usize,
    to_clear: ArrayVec<[Move; 72]>,
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
    pub fn draw(&self) {
        let mut cntr = 0;

        for y in (0..12).rev() {
            for x in 0..6 {
                //print!("{}|", x + (y * 6));
                let piece = self.board[y][x];

                if y <= self.water_level {
                    print!("{}", defs::draw_piece(&piece).blue())
                } else {
                    print!("{}", defs::draw_piece(&piece));
                }
            }
            println!();
        }
    }

    #[inline]
    fn remove_clears(&mut self) {
        for moving in self.to_clear.iter() {
            self.board[moving.y][moving.x] = CLEARED;
        }
        self.to_clear.clear();
    }

    #[inline]
    fn jelly(&mut self, clearing: defs::Pieces) {
        for y in 0..12 {
            for x in 0..6 {
                if self.board[y][x] == clearing {
                    self.to_clear.push(Move{x, y});
                }
            }
        }
    }

    #[inline]
    fn puff(&mut self, x: usize, y: usize) {
        let up = y > 0;
        let down = y < 11;
        let right = x < 5;
        let left = x > 0;


        self.to_clear.push(Move{x, y});

        if up {
            self.to_clear.push(Move{x, y: y - 1});
        }
        if down {
            self.to_clear.push(Move{x, y: y + 1});
        }
        if left {
            self.to_clear.push(Move{x: x - 1, y});
        }
        if right {
            self.to_clear.push(Move{x: x + 1, y});
        }

        if up && right {
            self.to_clear.push(Move{x: x + 1, y: y - 1});
        }
        if up && left {
            self.to_clear.push(Move{x: x - 1, y: y - 1});
        }
        if down && right {
            self.to_clear.push(Move{x: x + 1, y: y + 1});
        }
        if down && left {
            self.to_clear.push(Move{x: x - 1, y: y + 1});
        }
    }

    #[inline]
    pub fn swap(&mut self, pos: &Move) -> i32 {
        if pos.x == 5 {
            return -9001;
        }

        let mut score = 0;

        let one = self.board[pos.y][pos.x];
        let two = self.board[pos.y][pos.x + 1];

        self.something_cleared = false;

        if one == CLEARED || two == CLEARED {
            return -20001;
        } else if one == two {
            return -30001;
        } else if one == CRAB {
            return -9001;
        } else if one == PUFFERFISH || two == PUFFERFISH {
            if one == PUFFERFISH {
                self.puff(pos.y, pos.x);
            } else {
                self.puff(pos.y, pos.x + 1);
            }

            self.remove_clears();
            self.shift_everything();
            self.something_cleared = true
        } else if two == CRAB {
            return -90001;
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
            self.board[pos.y][pos.x] = two;
            self.board[pos.y][pos.x + 1] = one;

            score = 10 * self.get_combo(pos.y, pos.x);
            if score > 0 {
                self.something_cleared = true
            }
        }

        if self.something_cleared {
            self.clean_board()
        }

        score
    }

    #[inline]
    pub fn get_moves(&self) -> Vec<Move> {
        let mut move_vec: Vec<Move> = Vec::with_capacity(60);

        for y in 0..12 {
            for x in 0..5 {
                if self.board[y][x] == CLEARED || self.board[y][x] == NULL {
                    continue;
                }

                if self.board[y][x + 1] == CLEARED || self.board[y][x + 1] == NULL {
                    continue;
                }

                if self.board[y][x] != self.board[y][x + 1] {
                    move_vec.push(Move { x, y })
                }
            }
        }

        return move_vec;
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

        for y in 0..12 {
            for x in 0..6 {
                let piece = self.board[y][x];

                if y > self.water_level && piece == CRAB {
                    self.to_clear.push(Move{x, y});
                    returning = true;
                    continue;
                }

                if !can_move(piece) {
                    continue;
                }

                if x < 4 && piece == self.board[y][x + 1] && piece == self.board[y][x + 2] {
                    self.to_clear.push(Move{x, y});
                    self.to_clear.push(Move{x: x + 1, y});
                    self.to_clear.push(Move{x: x + 2, y});

                    returning = true;
                }

                if y < 10 && piece == self.board[y + 1][x] && piece == self.board[y + 2][x] {
                    self.to_clear.push(Move{x, y});
                    self.to_clear.push(Move{x, y: y + 1});
                    self.to_clear.push(Move{x, y: y + 2});

                    returning = true;
                }
            }
        }

        returning
    }

    #[inline]
    pub fn hash_me(&self) -> u64 {
        let mut s = MetroHasher::default();
        self.board.hash(&mut s);
        s.finish()
    }

    #[inline]
    fn shift_everything(&mut self) {
        for x in 0..6 {
            let mut last = 99999;
            for y in 0..12 {
                if self.board[y][x] == CLEARED && last == 99999 {
                    last = y;
                }
                if last != 99999 && self.board[y][x] != CLEARED {
                    self.board[last][x] = self.board[y][x];
                    self.board[y][x] = CLEARED;
                    last += 1;
                }
            }
        }
    }

    #[inline]
    pub fn get_best_combo(&self) -> i32 {
        let mut max = 0;

        for y in 0..(60 / 5) - 1 {
            for x in 0..5 {
                let left_piece = self.board[y][x];
                let right_piece = self.board[y][x + 1];

                if left_piece != right_piece && can_move(left_piece) && can_move(right_piece)
                {
                    let combo = self.get_combo(y, x);
                    if combo > max {
                        max = combo
                    }
                }
            }
        }

        max
    }

    #[inline]
    fn get_combo(&self, y: usize, x: usize) -> i32 {
        let left_piece = self.board[y][x];
        let right_piece = self.board[y][x + 1];

        let mut left = 1; //left 3 pieces
        let mut l_col = 1; //left column of 5 pieces
        let mut right = 1; //right 3 pieces
        let mut r_col = 1; //left column of 5 pieces

        if x > 1 && self.board[y][x - 1] == left_piece && self.board[y][x - 2] == left_piece {
            left = 3;
        }
        if x < 3 && self.board[y][x + 2] == right_piece && self.board[y][x + 3] == right_piece {
            right = 3;
        }
        if y > 0 && self.board[y - 1][x] == left_piece {
            l_col += 1;
            if y > 1 && self.board[y - 2][x] == left_piece {
                l_col += 1;
            }
        }
        if y < 11 && self.board[y + 1][x] == left_piece {
            l_col += 1;
            if y < 10 && self.board[y + 2][x] == left_piece {
                l_col += 1;
            }
        }
        if y > 0 && self.board[y - 1][x + 1] == right_piece {
            r_col += 1;
            if y > 1 && self.board[y - 2][x + 1] == right_piece {
                r_col += 1;
            }
        }
        if y < 11 && self.board[y + 1][x + 1] == right_piece {
            r_col += 1;
            if y < 10 && self.board[y + 2][x + 1] == right_piece {
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
    let mut board = [[defs::Pieces::CLEARED; 6]; 12];
    let mut rng = rand::thread_rng();

    for y in &mut board {
        for x in y.iter_mut()  {
            *x = defs::piece_from_num(&rng.gen_range(1, 7));
        }
    }

    GameState {
        water_level: 3,
        board,
        to_clear: ArrayVec::new(),
        something_cleared: false,
    }
}

pub fn copy_board(copying: &GameState) -> GameState {
    copying.clone()
}

pub fn board_from_array(board: [[defs::Pieces; 6]; 12]) -> GameState {
    GameState {
        water_level: 3,
        board,
        to_clear: ArrayVec::new(),
        something_cleared: false,
    }
}

pub fn generate_game() -> GameState {
    GameState {
        water_level: 3,
        board: [[defs::Pieces::CLEARED; 6]; 12],
        to_clear: ArrayVec::new(),
        something_cleared: false,
    }
}
