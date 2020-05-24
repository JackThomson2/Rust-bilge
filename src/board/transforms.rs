use crate::board::*;

use seahash;

use colored::*;
use defs::Pieces::*;
use helpers::can_move;

use arrayvec::ArrayVec;

impl GameState {
    pub fn as_dani_string(&self) -> String {
        self.board
            .iter()
            .map(|pce| (*pce as u8) - 1)
            .map(|st| st.to_string())
            .collect()
    }

    #[inline]
    pub fn draw(&self) {
        self.draw_highlight(99);
    }

    #[inline]
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
    pub fn remove_clears(&mut self) {
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

        let one = unsafe { *self.board.get_unchecked(pos) };
        let two = unsafe { *self.board.get_unchecked(pos + 1) };

        let mut return_score: f32;

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
            self.something_cleared = true
        } else if one == JELLYFISH || two == JELLYFISH {
            if one == JELLYFISH {
                self.jelly(two);
            } else {
                self.jelly(one);
            }
            return_score = self.clear_count as f32;

            self.remove_clears();
            self.shift_everything();
            self.something_cleared = true
        } else {
            unsafe { *self.board.get_unchecked_mut(pos) = two };
            unsafe { *self.board.get_unchecked_mut(pos + 1) = one };

            let mut score = self.get_combo(pos) as f32;

            let mut moves = new_tracker(pos);

            if score > 0.0 {
                score += self.clean_board_beta(&mut moves);
            }
            return score;
        }

        if self.something_cleared {
            return_score += self.clean_board();
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
            if left == CLEARED || left == NULL || left == CRAB {
                continue;
            }

            let right = unsafe { *self.board.get_unchecked(pos + 1) };
            if right == CLEARED || right == NULL || right == CRAB || right == left {
                continue;
            }
            unsafe { move_vec.push_unchecked(pos) };
        }

        move_vec
    }

    #[inline]
    pub fn clean_board(&mut self) -> f32 {
        let mut extra_broken = 0.0;
        let mut clear_res = self.mark_clears();

        while clear_res.0 {
            extra_broken += self.clear_count as f32;
            extra_broken += clear_res.1;
            self.remove_clears();
            self.shift_everything();
            clear_res = self.mark_clears();
        }

        extra_broken
    }

    #[inline]
    fn mark_clears(&mut self) -> (bool, f32) {
        let mut returning = false;
        let mut bonus_score = 0.0;

        for (pos, piece) in self.board.iter().enumerate() {
            let piece = *piece;
            let x = x_pos!(pos);
            let y = y_pos!(pos);

            let board_size = 72;

            if y > self.water_level && piece == CRAB {
                unsafe {
                    *self.to_clear.get_unchecked_mut(self.clear_count) = pos;
                }
                self.clear_count += 1;
                returning = true;
                bonus_score += (self.water_level * 2) as f32;

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
                    *self.to_clear.get_unchecked_mut(self.clear_count + 1) = pos + 1;
                    *self.to_clear.get_unchecked_mut(self.clear_count + 2) = pos + 2;
                    self.clear_count += 3;

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

                    returning = true;
                }
            }
        }

        (returning, bonus_score)
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
    fn get_combo(&self, pos: usize) -> i32 {
        let x = x_pos!(pos);

        let left_piece = unsafe { self.board.get_unchecked(pos) };
        let right_piece = unsafe { self.board.get_unchecked(pos + 1) };

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
            return 9999999; // Minus to give space for other points
        }

        if mult_ct == 4 && (r_col == 5 || l_col == 5) {
            return 999999; // Minus to give space for other points
        }

        (row_score!(left) + row_score!(right) + row_score!(l_col) + row_score!(r_col))
            * promote_scorers!(mult_ct)
    }
}