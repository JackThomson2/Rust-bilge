pub mod alt_search;
pub mod defs;
pub mod searcher;

use std::collections::HashSet;

#[macro_use]
pub mod helpers;

use colored::*;
use defs::Pieces::{self, *};
use helpers::can_move;

type col_set = HashSet<usize>;

use arrayvec::ArrayVec;
use rand::Rng;

type PieceArray = ArrayVec<[usize; 72]>;

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
    #[inline]
    pub fn as_dani_string(&self) -> String {
        self.board
            .iter()
            .map(|pce| (*pce as u8) - 1)
            .map(|st| st.to_string())
            .collect()
    }

    pub fn draw(&self) {
        self.draw_highlight(99);
    }

    pub fn draw_highlight(&self, position: usize) {
        println!();

        for y in (0..12).rev() {
            for x in 0..6 {
                let pos = (y * 6) + x;
                let piece = &self.board[pos];

                if pos == position || pos == position + 1 {
                    print!("{}", defs::draw_piece(piece).bright_green())
                } else if x_pos!(pos) == 5 {
                    print!("{}", defs::draw_piece(piece).red())
                } else if y <= self.water_level {
                    print!("{}", defs::draw_piece(piece).blue())
                } else {
                    print!("{}", defs::draw_piece(piece));
                }
            }
            println!(" : {}", y);
        }
        println!();
    }

    #[inline]
    fn remove_clears(&mut self) {
        if self.clear_count == 0 {
            return;
        }

        for count in 0..self.clear_count {
            unsafe {
                let loc = self.to_clear.get_unchecked(count);
                *self.board.get_unchecked_mut(*loc) = CLEARED;
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

        let up = pos >= 6;
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
        let mut y_height = 0;
        if pos >= 71 || x_pos!(pos) == 5 {
            return -9001.0;
        }

        let one = unsafe { *self.board.get_unchecked(pos) };
        let two = unsafe { *self.board.get_unchecked(pos + 1) };

        let mut return_score = 0.0;

        if one == CLEARED || two == CLEARED {
            return -20001.0;
        } else if one == two {
            return -30001.0;
        } else if one == CRAB || two == CRAB {
            return -9001.0;
        } else if one == PUFFERFISH || two == PUFFERFISH {
            if one == PUFFERFISH {
                self.puff(pos);
            } else {
                self.puff(pos + 1);
            }

            return_score = self.clear_count as f32;
            self.remove_clears();
            self.shift_everything();
            self.something_cleared = true;
            y_height = std::cmp::min(11, y_pos!(pos) + 1);
        } else if one == JELLYFISH || two == JELLYFISH {
            if one == JELLYFISH {
                self.jelly(two);
            } else {
                self.jelly(one);
            }
            return_score = self.clear_count as f32;

            self.remove_clears();
            self.shift_everything();
            self.something_cleared = true;
            y_height = 11;
        } else {
            unsafe {
                *self.board.get_unchecked_mut(pos) = two;
                *self.board.get_unchecked_mut(pos + 1) = one;
            }

            let mut score = self.get_combo(pos, &two, &one) as f32;
            if score > 0.0 {

                let mut testing: PieceArray = ArrayVec::new();
                unsafe {
                    testing.push_unchecked(pos);
                    testing.push_unchecked(pos + 1);
                }

                score += self.clean_board_vecaro(testing);
            }
            return score;
        }

        if self.something_cleared {
            return_score += self.clean_board(y_height);
        }

        return_score
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
    pub fn clean_board(&mut self, max_y: usize) -> f32 {
        let mut extra_broken = 0.0;
        let mut clear_res = self.mark_clears(max_y);

        while clear_res.0 {
            extra_broken += self.clear_count as f32;
            extra_broken += clear_res.1;
            self.remove_clears();
            self.shift_everything();
            clear_res = self.mark_clears(clear_res.2);
        }

        extra_broken
    }

    #[inline]
    pub fn clean_board_vecaro(&mut self, clearing: ArrayVec<[usize; 72]>) -> f32 {
        let mut extra_broken = 0.0;
        let mut clearing = clearing;
        let mut clear_res = self.clear_vec(&clearing);

        while clear_res.0 {
            extra_broken += self.clear_count as f32;
            extra_broken += clear_res.1;
            self.remove_clears();
            clearing = self.shift_everything_vecaro(clear_res.2);
            clear_res = self.clear_vec(&clearing);
        }

        extra_broken
    }

    #[inline]
    fn clear_vec(&mut self, to_clear: &ArrayVec<[usize; 72]>) -> (bool, f32, col_set) {
        let mut returning = false;
        let mut bonus_score = 0f32;
        let mut cols = HashSet::with_capacity(6);

        for pce in to_clear {
            let x = x_pos!(pce);
            let y = y_pos!(pce);
            let piece = unsafe { *self.board.get_unchecked(*pce) };

            if y > self.water_level && piece == CRAB {
                unsafe {
                    *self.to_clear.get_unchecked_mut(self.clear_count) = *pce;
                }
                self.clear_count += 1;
                returning = true;
                bonus_score += (self.water_level * 2) as f32;

                cols.insert(x);
                continue;
            }

            if !can_move(piece) {
                continue;
            }

            let mut left = 0;
            let mut right = 0;
            let mut up = 0;
            let mut down = 0;

            unsafe {
                if x > 0 && piece ==  *self.board.get_unchecked(pce - 1) {
                    left += 1;
                    if x > 1 && piece ==  *self.board.get_unchecked(pce - 2) {
                        left += 1;
                    }
                }

                if x < 5 && piece ==  *self.board.get_unchecked(pce + 1) {
                    right += 1;
                    if x < 4 && piece ==  *self.board.get_unchecked(pce + 2) {
                        right += 1;
                    }
                }

                if y > 0 && piece ==  *self.board.get_unchecked(pce - 6) {
                    down += 1;
                    if y > 1 && piece ==  *self.board.get_unchecked(pce - 12) {
                        down += 1;
                    }
                }

                if y < 11 && piece ==  *self.board.get_unchecked(pce + 6) {
                    down += 1;
                    if x < 10 && piece ==  *self.board.get_unchecked(pce + 12) {
                        down += 1;
                    }
                }

                if left + right >= 2 {
                    cols.insert(x);
                    returning = true;
                    if left > 0 {
                        for left_cnt in 1..left+1 {
                            *self.to_clear.get_unchecked_mut(self.clear_count) = pce - left_cnt;
                            self.clear_count += 1;

                            cols.insert(pce - left_cnt);
                        }
                    }

                    if right > 0 {
                        for right_cnt in 1..right+1 {
                            *self.to_clear.get_unchecked_mut(self.clear_count) = pce + right_cnt;
                            self.clear_count += 1;
                            cols.insert(pce + right_cnt);
                        }
                    }
                }
    
                if up + down >= 2 {
                    cols.insert(x);
                    returning = true;
                    if down > 0 {
                        for down_cnt in 1..down+1 {
                            *self.to_clear.get_unchecked_mut(self.clear_count) = pce - (down_cnt * 6);
                            self.clear_count += 1;
                        }
                    }

                    if up > 0 {
                        for up_cnt in 1..up+1 {
                            *self.to_clear.get_unchecked_mut(self.clear_count) = pce + (up_cnt * 6);
                            self.clear_count += 1;
                        }
                    }
                }
            }
        }

        (returning, bonus_score, cols)
    }

    #[inline]
    fn mark_clears(&mut self, max_y: usize) -> (bool, f32, usize) {
        let mut returning = false;
        let mut bonus_score = 0.0;
        let mut highest = 0;
        let mut past_water = false;

        for y in 0..max_y + 1 {
            if !past_water && y > self.water_level {
                past_water = true;
            }

            for x in 0..6 {
                let pos = x + (y * 6);
                let piece = unsafe {*self.board.get_unchecked(pos)};

                if past_water && piece == CRAB {
                    unsafe {
                        *self.to_clear.get_unchecked_mut(self.clear_count) = pos;
                    }
                    self.clear_count += 1;
                    returning = true;
                    bonus_score += (self.water_level * 2) as f32;

                    highest = y;
                    continue;
                }
    
                if !can_move(piece) {
                    continue;
                }
    
                unsafe {
                    if x < 4
                        && piece == *self.board.get_unchecked(pos + 1)
                        && piece == *self.board.get_unchecked(pos + 2)
                    {
                        *self.to_clear.get_unchecked_mut(self.clear_count) = pos;
                        *self.to_clear.get_unchecked_mut(self.clear_count + 1) = pos + 1;
                        *self.to_clear.get_unchecked_mut(self.clear_count + 2) = pos + 2;
                        self.clear_count += 3;

                        highest = y;
    
                        returning = true;
                    }
    
                    if pos < 60
                        && piece == *self.board.get_unchecked(pos + 6)
                        && piece == *self.board.get_unchecked(pos + 12)
                    {
                        *self.to_clear.get_unchecked_mut(self.clear_count) = pos;
                        *self.to_clear.get_unchecked_mut(self.clear_count + 1) = pos + 6;
                        *self.to_clear.get_unchecked_mut(self.clear_count + 2) = pos + 12;
                        self.clear_count += 3;

                        highest = y + 3;
    
                        returning = true;
                    }
                }
            }
        }

        (returning, bonus_score, highest)
    }

    #[inline]
    fn shift_everything(&mut self) {
        for x in 0..6 {
            let mut last = 99999;
            for y in (0..12).rev() {
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
                        last -= 1;
                    }
                }
            }
        }
    }

    #[inline]
    fn shift_everything_vecaro(&mut self, columns: col_set) -> ArrayVec<[usize; 72]> {
        let mut to_test = ArrayVec::new();

        for x in columns.iter() {
            let mut last = 99999;
            for y in (0..12).rev() {
                let pos = (y * 6) + x;
                unsafe {
                    let checking = *self.board.get_unchecked(pos);
                    if checking == CLEARED && last == 99999 {
                        last = y;
                    }

                    if last != 99999 && checking != CLEARED {
                        let last_pos = (last * 6) + x;
                        *self.board.get_unchecked_mut(last_pos) = checking;
                        to_test.push_unchecked(last_pos);

                        *self.board.get_unchecked_mut(pos) = CLEARED;
                        last -= 1;
                    }
                }
            }
        }

        to_test
    }

    #[inline]
    fn get_combo(&self, pos: usize, left_piece: &Pieces, right_piece: &Pieces) -> i32 {
        let x = x_pos!(pos);

        let mut left = 1; //left 3 pieces
        let mut l_col = 1; //left column of 5 pieces
        let mut right = 1; //right 3 pieces
        let mut r_col = 1; //left column of 5 pieces

        unsafe {
            if x >= 2
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

        if mult_ct == 4 && (r_col == 5 && l_col == 5) {
            return 9999999; 
        }

        if mult_ct == 4 && (r_col == 5 || l_col == 5) {
            return 999999;
        }

        (row_score!(left) + row_score!(right) + row_score!(l_col) + row_score!(r_col))
            * promote_scorers!(mult_ct)
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
