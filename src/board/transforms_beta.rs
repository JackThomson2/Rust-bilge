use std::{intrinsics::unlikely, mem::MaybeUninit, slice};

use crate::board::*;
use defs::*;

impl GameState {
    #[inline]
    pub fn clean_board_beta(&mut self, pos: usize) -> f32 {
        let mut position_tracker: [isize; 6] = [-1; 6];
        let mut removing_tracker: [usize; 72] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut removing_count: usize = 2;

        removing_tracker[0] = pos;
        removing_tracker[1] = pos + 1;

        let mut extra_broken = 0.0;
        let mut clear_res = self.mark_clears_targetted(&mut removing_count, &mut removing_tracker);

        while clear_res.0 {
            extra_broken += clear_res.1 + self.clear_count() as f32;

            self.remove_clears_max(&mut position_tracker);
            self.shift_tracked(
                &mut position_tracker,
                &mut removing_count,
                &mut removing_tracker,
            );
            clear_res = self.mark_clears_targetted(&mut removing_count, &mut removing_tracker);
        }

        extra_broken
    }

    #[inline]
    fn shift_tracked(
        &mut self,
        position_tracker: &mut [isize; 6],
        removing_count: &mut usize,
        removing_tracker: &mut [usize; 72],
    ) {
        unsafe {
            for (x, max_y) in position_tracker.iter().enumerate() {
                let max_y = match max_y {
                    d if *d < 0 => continue,
                    a => *a as usize,
                };

                let mut last = 99999;
                for y in (0..=max_y).rev() {
                    let pos = (y * 6) + x;
                    let checking = *self.board.get_unchecked(pos);
                    if checking == CLEARED && last == 99999 {
                        last = y;
                    }

                    if last != 99999 && checking != CLEARED {
                        let last_pos = (last * 6) + x;
                        *self.board.get_unchecked_mut(last_pos) = checking;
                        *self.board.get_unchecked_mut(pos) = CLEARED;

                        *removing_tracker.get_unchecked_mut(*removing_count) = last_pos;
                        *removing_count += 1;
                        last -= 1;
                    }
                }
            }
        }
    }

    #[inline]
    /// New function which will return the biggest y cleared
    pub fn remove_clears_max(&mut self, position_tracker: &mut [isize; 6]) {
        if self.clear_count() == 0 {
            return;
        }

        position_tracker.iter_mut().for_each(|m| *m = -1);

        while self.clear_count() != 0 {
            unsafe {
                let loc = self.get_position();
                *self.board.get_unchecked_mut(loc) = CLEARED;

                let x_pos = x_pos_fast(loc);

                *position_tracker.get_unchecked_mut(x_pos) = std::cmp::max(
                    *position_tracker.get_unchecked(x_pos),
                    y_pos_fast(loc) as isize,
                );
            }
        }
        self.reset_clears();
    }

    /// Alternative to mark clears which will check around a point
    #[inline]
    fn mark_clears_targetted(
        &mut self,
        removing_count: &mut usize,
        removing_tracker: &mut [usize; 72],
    ) -> (bool, f32) {
        let mut returning = false;
        let mut bonus_score = 0.0;

        unsafe {
            let ptr = removing_tracker.as_mut_ptr();
            let slice = slice::from_raw_parts(ptr, *removing_count);

            for pos in slice.iter() {
                let pos = *pos;
                let piece = *self.board.get_unchecked(pos);

                let y = y_pos_fast(pos);

                if unlikely(piece == CRAB && y > self.water_level as usize) {
                    self.set_to_clear(pos);

                    returning = true;
                    bonus_score += (self.water_level * 2) as f32;

                    continue;
                }

                if unlikely(!can_move(piece)) {
                    continue;
                }

                let mut x_left_range = 0;
                let mut x_right_range = 0;

                let mut y_up_range = 0;
                let mut y_down_range = 0;

                let x = x_pos_fast(pos);

                if x < 5 && piece == *self.board.get_unchecked(pos + 1) {
                    x_right_range += 1;

                    if x < 4 && piece == *self.board.get_unchecked(pos + 2) {
                        x_right_range += 1;
                    }
                }

                if x > 0 && piece == *self.board.get_unchecked(pos - 1) {
                    x_left_range += 1;

                    if x > 1 && piece == *self.board.get_unchecked(pos - 2) {
                        x_left_range += 1;
                    }
                }

                if y < 11 && piece == *self.board.get_unchecked(pos + 6) {
                    y_up_range += 1;

                    if y < 10 && piece == *self.board.get_unchecked(pos + 12) {
                        y_up_range += 1;
                    }
                }

                if y > 0 && piece == *self.board.get_unchecked(pos - 6) {
                    y_down_range += 1;

                    if y > 1 && piece == *self.board.get_unchecked(pos - 12) {
                        y_down_range += 1;
                    }
                }

                // Move than 3
                if x_left_range + x_right_range > 1 {
                    returning = true;

                    self.set_to_clear(pos);
                    if x_right_range > 0 {
                        for x_range in 1..x_right_range + 1 {
                            self.set_to_clear(pos + x_range);
                        }
                    }

                    if x_left_range > 0 {
                        for x_range in 1..x_left_range + 1 {
                            self.set_to_clear(pos - x_range);
                        }
                    }
                }

                // Move than 3
                if y_up_range + y_down_range > 1 {
                    returning = true;
                    self.set_to_clear(pos);

                    if y_up_range > 0 {
                        for y_range in 1..y_up_range + 1 {
                            self.set_to_clear(pos + (y_range * 6));
                        }
                    }

                    if y_down_range > 0 {
                        for y_range in 1..y_down_range + 1 {
                            self.set_to_clear(pos - (y_range * 6));
                        }
                    }
                }
            }

            *removing_count = 0;
        }

        (returning, bonus_score)
    }
}
