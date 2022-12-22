use crate::macros::SafeGetters;

const MOVE_MASK: u64 = 0b0111_1111;
const SCORED_MASK: u64 = 0b1000_0000;

#[inline]
pub fn check_if_previously_run(checking: usize, lookup: u64) -> bool {
    let mut returning = false;

    let left = *AFFECTED_PAIRS.get_safely(checking).get_safely(0);
    let right = *AFFECTED_PAIRS.get_safely(checking).get_safely(1);

    for i in 0..8 {
        let shifted = lookup >> (i * 8);
        let scored = shifted & SCORED_MASK;        
        let move_position = shifted & MOVE_MASK;
        
        if move_position as usize == checking {
            returning = true;
        }

        if scored > 0 {
            returning = false;
        }

        if left == move_position || right == move_position {
            returning = false;
        }
    }

    returning
}

#[inline]
pub fn record_move(total_depth: u8, curr_depth: u8, curr_mask: u64, move_loc: usize, scored: bool) -> u64 {
    let actual_depth = total_depth - curr_depth;
    let mut move_loc = move_loc as u64;

    if actual_depth > 8 {
        return curr_mask;
    }

    if scored {
        move_loc |= SCORED_MASK;
    }

    curr_mask | (move_loc << (actual_depth * 8))
}

#[inline]
const fn build_pair_array() -> [[u64; 2]; 72] {
    let mut end = [[255; 2]; 72];
    let mut cntr = 0;

    loop {
        let x = cntr % 6;
        if x > 0 {
            end[cntr][0] = cntr as u64 - 1;
        }

        if x < 5 {
            end[cntr][1] = cntr as u64 + 1;
        }

        cntr += 1;
        if cntr >= 72 {
            break;
        }
    }

    end
}

pub const AFFECTED_PAIRS: [[u64; 2]; 72] = build_pair_array();
