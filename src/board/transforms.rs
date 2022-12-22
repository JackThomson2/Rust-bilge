use std::intrinsics::{likely, unlikely};

use crate::board::*;

use defs::*;
use helpers::can_move;
use recolored::*;

use std::arch::x86_64 as x86;
use crate::macros::SafeGetters;

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
        let masks = SET_BIT_MASKS.get_safely(new_value);

        self.to_clear_l |= masks.0;
        self.to_clear_r |= masks.1;
    }

    #[inline(always)]
    pub fn apply_pair_to_self(&mut self, pair: (u64, u16)) {
        self.to_clear_l |= pair.0;
        self.to_clear_r |= pair.1;
    }

    #[inline(always)]
    pub fn set_to_inside(&self, a: &mut u64, b: &mut u16, new_value: usize) {
        let masks = SET_BIT_MASKS.get_safely(new_value);

        *a |= masks.0;
        *b |= masks.1;
    }

    #[inline(always)]
    pub fn set_start(&mut self, new_value: usize) {
        if unlikely(new_value == 63) {
            self.set_to_clear(new_value);
            self.set_to_clear(new_value + 1);

            return;
        }

        if unlikely(new_value >= 64) {
            const START: u16 = 0b_0011;
            self.to_clear_r |= START << (new_value - 64);
            return;
        }

        const START_BIG: u64 = 0b0011;
        self.to_clear_l |= START_BIG << new_value;
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

        new_pos as usize + 64
    }

    #[inline]
    pub fn remove_clears(&mut self) {
        while self.to_clear_l != 0 {
            let loc = self.get_position();
            *self.board.get_mut_safely(loc) = CLEARED;
        }

        while self.to_clear_r != 0 {
            let loc = self.get_position();
            *self.board.get_mut_safely(loc) = CLEARED;
        }
    }

    #[inline]
    pub fn jelly_512(&mut self, clearing: u8) {
        unsafe {
            let clear_mask = x86::_mm512_set1_epi8(clearing as i8);
            let ptr = self.board.as_ptr();
            let x = x86::_mm512_loadu_si512(ptr.cast());
            self.to_clear_l |= x86::_mm512_cmpeq_epi8_mask(x, clear_mask);

            for i in 64..72 {
                let checking = self.board[i];

                self.to_clear_r |= ((checking == clearing) as u16) << (i - 64);
            }
        }
    }

    #[inline]
    pub fn jelly(&mut self, clearing: u8) {
        unsafe {
            let clear_mask = x86::_mm256_set1_epi8(clearing as i8);
            let ptr = self.board.as_ptr();

            let x = x86::_mm256_loadu_si256(ptr.cast());
            let res = x86::_mm256_cmpeq_epi8(x, clear_mask);
            self.to_clear_l |= x86::_mm256_movemask_epi8(res) as u64;

            let x = x86::_mm256_loadu_si256(ptr.add(32).cast());
            let res = x86::_mm256_cmpeq_epi8(x, clear_mask);
            self.to_clear_l |= (x86::_mm256_movemask_epi8(res) as u64) << 32;

            for i in 64..72 {
                let checking = self.board[i];

                self.to_clear_r |= ((checking == clearing) as u16) << (i - 64);
            }
        }
    }

    #[inline]
    fn puff(&mut self, pos: usize) {
        let pair = *PUFFER.get_safely(pos);
        self.apply_pair_to_self(pair)
    }

    #[inline]
    pub fn swap(&mut self, pos: usize) -> f32 {
        self.reset_clears();
        let something_cleared;

        let one = *self.board.get_safely(pos);
        let two = *self.board.get_safely(pos + 1);

        let mut return_score: f32;

        if unlikely(one == CLEARED || two == CLEARED) {
            return -20001.0;
        } else if unlikely(one == two) {
            return -30001.0;
        } else if unlikely(one == CRAB || two == CRAB) {
            return -9001.0;
        } else if unlikely(one == PUFFERFISH || two == PUFFERFISH) {
            if one == PUFFERFISH {
                self.puff(pos);
            } else {
                self.puff(pos + 1);
            }

            return_score = self.clear_count() as f32;
            self.remove_clears();
            self.shift_everything();
            something_cleared = true
        } else if unlikely(one == JELLYFISH || two == JELLYFISH) {
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
            *self.board.get_mut_safely(pos) = two;
            *self.board.get_mut_safely(pos + 1) = one;

            let mut score = self.get_combo(pos) as f32;

            if score > 0.0 {
                score += self.clean_board_beta(pos);
            }
            return score;
        }

        if unlikely(something_cleared) {
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

                let right = *self.board.get_safely(pos + 1);
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
            let y = y_pos_fast(pos);

            let board_size = 72;

            if unlikely(y > self.water_level as usize && piece == CRAB) {
                self.set_to_inside(&mut outer_a, &mut outer_b, pos);
                returning = true;
                bonus_score += (self.water_level * 2) as f32;

                continue;
            }

            if unlikely(!can_move(piece)) {
                continue;
            }
            
            let x = x_pos_fast(pos);

            if x < 4
                && piece == *self.board.get_safely(pos + 1)
                && piece == *self.board.get_safely(pos + 2)
            {
                self.set_to_inside(&mut outer_a, &mut outer_b, pos);
                self.set_to_inside(&mut outer_a, &mut outer_b, pos + 1);
                self.set_to_inside(&mut outer_a, &mut outer_b, pos + 2);

                returning = true;
            }

            if pos < 60
                && piece == *self.board.get_safely(pos + 6)
                && piece == *self.board.get_safely(pos + 12)
            {
                self.set_to_inside(&mut outer_a, &mut outer_b, pos);
                self.set_to_inside(&mut outer_a, &mut outer_b, pos + 6);
                self.set_to_inside(&mut outer_a, &mut outer_b, pos + 12);

                returning = true;
            }
        }

        self.to_clear_l |= outer_a;
        self.to_clear_r |= outer_b;

        (returning, bonus_score)
    }

    #[inline]
    pub fn shift_everything(&mut self) {
        for x in 0..6 {
            let mut pos = 11;

            for i in (0..12).rev() {
                let writing = (pos * 6) + x;
                let checking = *self.board.get_mut_safely(((i * 6) + x) as usize);

                *self.board.get_mut_safely(writing) = checking;

                let offset = *LUT.get_safely(checking as usize) as usize;
                pos -= offset;
            }
            match pos {
                0 => update_all(&mut self.board, x, 0),
                1 => update_all(&mut self.board, x, 1),
                2 => update_all(&mut self.board, x, 2),
                3 => update_all(&mut self.board, x, 3),
                4 => update_all(&mut self.board, x, 4),
                5 => update_all(&mut self.board, x, 5),
                6 => update_all(&mut self.board, x, 6),
                7 => update_all(&mut self.board, x, 7),
                8 => update_all(&mut self.board, x, 8),
                9 => update_all(&mut self.board, x, 9),
                10 => update_all(&mut self.board, x, 10),
                _ => {}
            }
        }
    }

    #[inline]
    fn get_combo(&self, pos: usize) -> i32 {
        let x = x_pos_fast(pos);

        let left_piece = self.board.get_safely(pos);
        let right_piece = self.board.get_safely(pos + 1);

        let mut mult_ct = 0;

        let mut left = 0; //left 3 pieces
        let mut l_col = 1; //left column of 5 pieces
        let mut right = 0; //right 3 pieces
        let mut r_col = 1; //left column of 5 pieces

        if x >= 2
            && self.board.get_safely(pos - 1) == left_piece
            && self.board.get_safely(pos - 2) == left_piece
        {
            left = 3;
            mult_ct = 1;
        }

        if x < 3
            && self.board.get_safely(pos + 2) == right_piece
            && self.board.get_safely(pos + 3) == right_piece
        {
            right = 3;
            mult_ct += 1;
        }

        if pos > 5 && self.board.get_safely(pos - 6) == left_piece {
            l_col += 1;
            if pos > 11 && self.board.get_safely(pos - 12) == left_piece {
                l_col += 1;
            }
        }

        if pos < 66 && self.board.get_safely(pos + 6) == left_piece {
            l_col += 1;
            if pos < 60 && self.board.get_safely(pos + 12) == left_piece {
                l_col += 1;
            }
        }

        if pos > 4 && self.board.get_safely(pos - 5) == right_piece {
            r_col += 1;
            if pos > 10 && self.board.get_safely(pos - 11) == right_piece {
                r_col += 1;
            }
        }

        if pos < 65 && self.board.get_safely(pos + 7) == right_piece {
            r_col += 1;
            if pos < 59 && self.board.get_safely(pos + 13) == right_piece {
                r_col += 1;
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
