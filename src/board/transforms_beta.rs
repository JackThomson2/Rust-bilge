use crate::board::*;
use bit_array::BitArray;
use defs::Pieces::*;

impl GameState {
    #[inline]
    pub fn clean_board_beta(&mut self, moves: &mut PositionTracker) -> f32 {
        let mut extra_broken = 0.0;
        let mut clear_res = self.mark_clears_targetted(moves);

        while clear_res.0 {
            extra_broken += clear_res.1 + self.clear_count as f32;

            self.remove_clears();
            self.shift_tracked(moves);
            clear_res = self.mark_clears_targetted(moves)
        }

        extra_broken
    }

    #[inline]
    fn shift_tracked(&mut self, found: &mut PositionTracker) {
        found.clear();

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

                        found.set_visible(last_pos);
                        found.set_invisible(pos);
                        last -= 1;
                    }
                }
            }
        }
    }

    #[inline]
    pub fn remove_clears_tracker(&mut self) -> Option<ShifterTracked> {
        if self.clear_count == 0 {
            return None;
        }

        let mut max_y = 0;
        let mut rows: RowArr = BitArray::from_elem(false);

        for count in 0..self.clear_count {
            unsafe {
                let loc = self.to_clear.get_unchecked(count);
                *self.board.get_unchecked_mut(*loc) = CLEARED;

                max_y = std::cmp::max(max_y, y_pos!(loc));
                rows.set(x_pos!(loc), true);
            }
        }

        self.clear_count = 0;

        Some((max_y, rows))
    }

    /// Alternative to mark clears which will check around a point
    #[inline]
    fn mark_clears_targetted(&mut self, checking: &PositionTracker) -> (bool, f32) {
        let mut returning = false;
        let mut bonus_score = 0.0;

        let iter = checking
            .get_inner()
            .iter()
            .enumerate()
            .filter(|(_pos, val)| *val);

        for (pos, _val) in iter {
            let piece = unsafe { *self.board.get_unchecked(pos) };

            let x = x_pos!(pos);
            let y = y_pos!(pos);

            let mut x_left_range = 0;
            let mut x_right_range = 0;

            let mut y_up_range = 0;
            let mut y_down_range = 0;

            if piece == CRAB && y > self.water_level {
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
                    *self.to_clear.get_unchecked_mut(self.clear_count) = pos;
                    self.clear_count += 1;

                    if x_right_range > 0 {
                        for x_range in 1..x_right_range + 1 {
                            *self.to_clear.get_unchecked_mut(self.clear_count) = pos + (x_range);
                            self.clear_count += 1;
                        }
                    }

                    if x_left_range > 0 {
                        for x_range in 1..x_left_range + 1 {
                            *self.to_clear.get_unchecked_mut(self.clear_count) = pos - (x_range);
                            self.clear_count += 1;
                        }
                    }
                }

                // Move than 3
                if y_up_range + y_down_range > 1 {
                    returning = true;
                    *self.to_clear.get_unchecked_mut(self.clear_count) = pos;
                    self.clear_count += 1;

                    if y_up_range > 0 {
                        for y_range in 1..y_up_range + 1 {
                            *self.to_clear.get_unchecked_mut(self.clear_count) =
                                pos + (y_range * 6);
                            self.clear_count += 1;
                        }
                    }

                    if y_down_range > 0 {
                        for y_range in 1..y_down_range + 1 {
                            *self.to_clear.get_unchecked_mut(self.clear_count) =
                                pos - (y_range * 6);
                            self.clear_count += 1;
                        }
                    }
                }
            }
        }

        (returning, bonus_score)
    }
}
