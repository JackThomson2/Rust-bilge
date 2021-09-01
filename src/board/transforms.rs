use std::intrinsics::{likely, unlikely};

use crate::board::*;

use defs::*;
use helpers::can_move;
use recolored::*;

use unroll::unroll_for_loops;

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
                let piece = self.board[pos];

                if pos == position || pos == position + 1 {
                    print!("{}", defs::draw_piece(piece).bright_green())
                } else if x_pos_fast(pos) == 5 {
                    print!("{}", defs::draw_piece(piece).red())
                } else if y <= self.water_level as usize {
                    print!("{}", defs::draw_piece(piece).blue())
                } else {
                    print!("{}", defs::draw_piece(piece));
                }
            }
            println!(" : {}", y);
        }
        println!();
    }

    #[inline(always)]
    pub fn clear_count(&self) -> usize {
        self.to_clear_l.count_ones() as usize + self.to_clear_r.count_ones() as usize
    }

    #[inline(always)]
    pub fn reset_clears(&mut self) {
        self.to_clear_l = 0;
        self.to_clear_r = 0;
    }

    #[inline(always)]
    pub fn set_to_clear(&mut self, new_value: usize) {
        if unlikely(new_value > 64) {
            let mask = 1 << (new_value - 64);
            self.to_clear_r |= mask;
            return;
        }

        let mask = 1 << new_value;

        self.to_clear_l |= mask;
    }

    #[inline(always)]
    pub fn set_to_inside(&self, a: &mut u64, b: &mut u16, new_value: usize) {
        if unlikely(new_value > 64) {
            let mask = 1 << (new_value - 64);
            *b |= mask;
            return;
        }

        let mask = 1 << new_value;

        *a |= mask;
    }

    #[inline(always)]
    pub fn get_position(&mut self) -> usize {
        if likely(self.to_clear_l != 0) {
            let new_pos = self.to_clear_l.trailing_zeros();
            let pos = 1 << new_pos;

            self.to_clear_l ^= pos;

            return new_pos as usize;
        }

        let new_pos = self.to_clear_r.trailing_zeros();
        let pos = 1 << new_pos;

        self.to_clear_r ^= pos;

        return new_pos as usize + 64;
    }

    #[inline]
    pub fn remove_clears(&mut self) {
        if self.clear_count() == 0 {
            return;
        }

        loop {
            unsafe {
                let loc = self.get_position();
                *self.board.get_unchecked_mut(loc) = CLEARED;
            }

            if self.clear_count() == 0 {
                break;
            }
        }

        self.reset_clears();
    }

    #[inline]
    fn jelly(&mut self, clearing: defs::Pieces) {
        let mut outer_a = 0;
        let mut outer_b = 0;

        for (loc, _pce) in self
            .board
            .iter()
            .enumerate()
            .filter(|(_loc, pce)| *pce == &clearing)
        {
            self.set_to_inside(&mut outer_a, &mut outer_b, loc);
        }

        self.to_clear_l |= outer_a;
        self.to_clear_r |= outer_b;
    }

    #[inline]
    fn puff(&mut self, pos: usize) {
        let x = x_pos_fast(pos);

        let up = pos >= 6;
        let down = pos < 66;
        let right = x < 5;
        let left = x > 0;

        self.set_to_clear(pos);

        if up {
            self.set_to_clear(pos - 6);
        }
        if down {
            self.set_to_clear(pos + 6);
        }
        if left {
            self.set_to_clear(pos - 1);
        }
        if right {
            self.set_to_clear(pos + 1);
        }

        if up && right {
            self.set_to_clear(pos - 5);
        }
        if up && left {
            self.set_to_clear(pos - 7);
        }
        if down && right {
            self.set_to_clear(pos + 7);
        }
        if down && left {
            self.set_to_clear(pos + 5);
        }
    }

    #[inline]
    pub fn swap(&mut self, pos: usize) -> f32 {
        self.reset_clears();
        let something_cleared;

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

            return_score = self.clear_count() as f32;
            self.remove_clears();
            self.shift_everything();
            something_cleared = true
        } else if one == JELLYFISH || two == JELLYFISH {
            if one == JELLYFISH {
                self.jelly(two);
            } else {
                self.jelly(one);
            }
            return_score = self.clear_count() as f32;

            self.remove_clears();
            self.shift_everything();
            something_cleared = true
        } else {
            unsafe { *self.board.get_unchecked_mut(pos) = two };
            unsafe { *self.board.get_unchecked_mut(pos + 1) = one };

            let mut score = self.get_combo(pos) as f32;

            if score > 0.0 {
                score += self.clean_board_beta();
            }
            return score;
        }

        if something_cleared {
            return_score += self.clean_board();
        }

        return_score
    }

    #[inline]
    pub fn get_moves(&self) -> ArrayVec<usize, 60> {
        self.board
            .iter()
            .enumerate()
            .filter_map(|(pos, pieces)| {
                if x_pos_fast(pos) == 5 {
                    return None;
                }

                let left = *pieces;
                if left == CLEARED || left == NULL || left == CRAB {
                    return None;
                }

                let right = unsafe { *self.board.get_unchecked(pos + 1) };
                if right == CLEARED || right == NULL || right == CRAB || right == left {
                    return None;
                }

                Some(pos)
            })
            .collect()
    }

    #[inline]
    pub fn clean_board(&mut self) -> f32 {
        let mut extra_broken = 0.0;
        let mut clear_res = self.mark_clears();

        while clear_res.0 {
            extra_broken += self.clear_count() as f32;
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

        let mut outer_a = 0;
        let mut outer_b = 0;

        for (pos, piece) in self.board.iter().enumerate() {
            let piece = *piece;
            let x = x_pos_fast(pos);
            let y = y_pos!(pos);

            let board_size = 72;

            if unlikely(y > self.water_level as usize && piece == CRAB) {
                self.set_to_inside(&mut outer_a, &mut outer_b, pos);
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
                    self.set_to_inside(&mut outer_a, &mut outer_b, pos);
                    self.set_to_inside(&mut outer_a, &mut outer_b, pos + 1);
                    self.set_to_inside(&mut outer_a, &mut outer_b, pos + 2);

                    returning = true;
                }

                if pos < 60
                    && piece == *self.board.get_unchecked(pos + 6)
                    && piece == *self.board.get_unchecked(pos + 12)
                {
                    self.set_to_inside(&mut outer_a, &mut outer_b, pos);
                    self.set_to_inside(&mut outer_a, &mut outer_b, pos + 6);
                    self.set_to_inside(&mut outer_a, &mut outer_b, pos + 12);

                    returning = true;
                }
            }
        }

        self.to_clear_l |= outer_a;
        self.to_clear_r |= outer_b;

        (returning, bonus_score)
    }

    #[unroll_for_loops]
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
        let x = x_pos_fast(pos);

        let left_piece = unsafe { self.board.get_unchecked(pos) };
        let right_piece = unsafe { self.board.get_unchecked(pos + 1) };

        let mut mult_ct = 0;

        let mut left = 0; //left 3 pieces
        let mut l_col = 1; //left column of 5 pieces
        let mut right = 0; //right 3 pieces
        let mut r_col = 1; //left column of 5 pieces

        unsafe {
            if x >= 2
                && self.board.get_unchecked(pos - 1) == left_piece
                && self.board.get_unchecked(pos - 2) == left_piece
            {
                left = 3;
                mult_ct = 1;
            }

            if x < 3
                && self.board.get_unchecked(pos + 2) == right_piece
                && self.board.get_unchecked(pos + 3) == right_piece
            {
                right = 3;
                mult_ct += 1;
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
