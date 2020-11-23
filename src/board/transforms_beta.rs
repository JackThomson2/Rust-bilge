use crate::board::*;
use defs::*;
use structure::set_to_clear;

#[thread_local]
pub static mut POSITION_TRACKER: [isize; 6] = [-1; 6];

#[thread_local]
pub static mut REMOVING_TRACKER: [usize; 72] = [0; 72];

#[thread_local]
pub static mut REMOVING_COUNT: usize = 0;

impl GameState {
    #[inline]
    pub fn clean_board_beta(&mut self) -> f32 {
        let mut extra_broken = 0.0;
        let mut clear_res = self.mark_clears_targetted();

        while clear_res.0 {
            extra_broken += clear_res.1 + clear_count() as f32;

            self.remove_clears_max();
            self.shift_tracked();
            clear_res = self.mark_clears_targetted();
        }

        extra_broken
    }

    #[inline]
    fn shift_tracked(&mut self) {
        unsafe {
            for (x, max_y) in POSITION_TRACKER.iter().enumerate() {
                let max_y= match max_y {
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

                        REMOVING_TRACKER[REMOVING_COUNT] = last_pos;
                        REMOVING_COUNT += 1;
                        last -= 1;
                    }
                }
            }
        }
    }

    #[inline]
    /// New function which will return the biggest y cleared
    pub fn remove_clears_max(&mut self) {
        if clear_count() == 0 {
            return;
        }

        unsafe {POSITION_TRACKER.iter_mut().for_each(|m| *m = -1);}

        for count in 0..clear_count() {
            unsafe {
                let loc = get_position(count);
                *self.board.get_unchecked_mut(loc) = CLEARED;

                let x_pos = x_pos!(loc);

                *POSITION_TRACKER.get_unchecked_mut(x_pos) =
                    std::cmp::max(*POSITION_TRACKER.get_unchecked(x_pos), y_pos!(loc) as isize);
            }
        }
        reset_clears();
    }

    /*
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
    }*/

    /// Alternative to mark clears which will check around a point
    #[inline]
    fn mark_clears_targetted(&mut self) -> (bool, f32) {
        let mut returning = false;
        let mut bonus_score = 0.0;

        unsafe {
            for pos in REMOVING_TRACKER[0..REMOVING_COUNT].iter() {
                let pos  = *pos;
                let piece = *self.board.get_unchecked(pos);

                let y = y_pos!(pos);

                if piece == CRAB && y > self.water_level {
                    set_to_clear(pos);

                    returning = true;
                    bonus_score += (self.water_level * 2) as f32;

                    continue;
                }

                if !can_move(piece) {
                    continue;
                }

                let mut x_left_range = 0;
                let mut x_right_range = 0;

                let mut y_up_range = 0;
                let mut y_down_range = 0;

                let x = x_pos!(pos);

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

            
            REMOVING_COUNT = 0;
        }


        (returning, bonus_score)
    }
}

#[inline]
pub fn setup_array(position: usize) {
    unsafe {
        REMOVING_TRACKER[0] = position;
        REMOVING_TRACKER[1] = position + 1;
        REMOVING_COUNT = 2;
    }
}
