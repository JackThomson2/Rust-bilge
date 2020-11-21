use crate::board::*;
use bit_array::BitArray;
use defs::*;
use structure::set_to_clear;

#[thread_local]
pub static mut POSITION_TRACKER: [isize; 6] = [-1; 6];

impl GameState {
    #[inline]
    pub fn clean_board_beta(&mut self) -> f32 {
        let mut extra_broken = 0.0;
        let mut clear_res = self.mark_clears_targetted();

        while clear_res.0 {
            extra_broken += clear_res.1 + clear_count() as f32;

            let max_y = self.remove_clears_max();
            self.shift_tracked(max_y);
            clear_res = self.mark_clears_targetted();
        }

        extra_broken
    }

    #[inline]
    fn shift_tracked(&mut self, max_y: usize) {
        unsafe {
            POSITION_TRACKER.iter_mut().for_each(|m| *m = -1);

            for x in 0..6 {
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

                        POSITION_TRACKER[x] = last_pos as isize;
                        last -= 1;
                    }
                }
            }
        }
    }

    #[inline]
    // New function which will return the biggest y cleared
    pub fn remove_clears_max(&mut self) -> usize {
        if clear_count() == 0 {
            return 0;
        }

        let mut max = 0;

        for count in 0..clear_count() {
            unsafe {
                let loc = get_position(count);
                *self.board.get_unchecked_mut(loc) = CLEARED;
                max = std::cmp::max(loc, max);
            }
        }

        reset_clears();

        y_pos!(max)
    }

    #[inline]
    pub fn remove_clears_tracker(&mut self) -> Option<ShifterTracked> {
        if clear_count() == 0 {
            return None;
        }

        let mut max_y = 0;
        let mut rows: RowArr = BitArray::from_elem(false);

        for count in 0..clear_count() {
            unsafe {
                let loc = get_position(count);
                *self.board.get_unchecked_mut(loc) = CLEARED;

                max_y = std::cmp::max(max_y, y_pos!(loc));
                rows.set(x_pos!(loc), true);
            }
        }

        reset_clears();
        Some((max_y, rows))
    }

    /// Alternative to mark clears which will check around a point
    #[inline]
    fn mark_clears_targetted(&mut self) -> (bool, f32) {
        let mut returning = false;
        let mut bonus_score = 0.0;

        unsafe {
            for pos in POSITION_TRACKER.iter() {
                let pos = match pos {
                    d if *d < 0 => continue,
                    a => *a as usize,
                };

                let piece = *self.board.get_unchecked(pos);

                let x = x_pos!(pos);
                let y = y_pos!(pos);

                let mut x_left_range = 0;
                let mut x_right_range = 0;

                let mut y_up_range = 0;
                let mut y_down_range = 0;

                if piece == CRAB && y > self.water_level {
                    set_to_clear(pos);

                    returning = true;
                    bonus_score += (self.water_level * 2) as f32;

                    continue;
                }

                if !can_move(piece) {
                    continue;
                }

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

                    set_to_clear(pos);
                    if x_right_range > 0 {
                        for x_range in 1..x_right_range + 1 {
                            set_to_clear(pos + x_range);
                        }
                    }

                    if x_left_range > 0 {
                        for x_range in 1..x_left_range + 1 {
                            set_to_clear(pos - x_range);
                        }
                    }
                }

                // Move than 3
                if y_up_range + y_down_range > 1 {
                    returning = true;
                    set_to_clear(pos);

                    if y_up_range > 0 {
                        for y_range in 1..y_up_range + 1 {
                            set_to_clear(pos + (y_range * 6));
                        }
                    }

                    if y_down_range > 0 {
                        for y_range in 1..y_down_range + 1 {
                            set_to_clear(pos - (y_range * 6));
                        }
                    }
                }
            }
        }

        (returning, bonus_score)
    }
}

#[inline]
pub fn setup_array(position: usize) {
    unsafe {
        POSITION_TRACKER.iter_mut().for_each(|m| *m = -1);

        let x_1 = x_pos!(position);
        let x_2 = x_pos!(position + 1);

        POSITION_TRACKER[x_1] = position as isize;
        POSITION_TRACKER[x_2] = (position + 1) as isize;
    }
}
