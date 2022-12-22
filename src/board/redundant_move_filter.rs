use crate::macros::SafeGetters;

const MOVE_MASK: u64 = 0b0111_1111;
const SCORED_MASK: u64 = 1 << 63;

#[inline]

pub fn check_if_previously_run(checking: usize, lookup: u64) -> bool {

    let scored = lookup & SCORED_MASK;

    if scored != 0 {
        return false;
    }

    let mut returning = false;
    let pair = *AFFECTED_PAIRS.get_safely(checking);

    for i in 0..8 {
        let shifted = lookup >> (i * 8);
        let move_position = shifted & MOVE_MASK;

        if pair == move_position {
            return false
        }

        if move_position as usize == checking {
            returning = true;
        }
    }

    returning
}

#[inline]
pub fn record_move(total_depth: u8, curr_depth: u8, curr_mask: u64, move_loc: usize, scored: bool) -> u64 {
    let actual_depth = total_depth - curr_depth;

    if actual_depth > 8 {
        return curr_mask;
    }

    let mut res = curr_mask | ((move_loc as u64) << (actual_depth * 8));

    if scored {
        res |= SCORED_MASK;
    }

    res
}

#[inline]
const fn build_pair_array() -> [u64; 72] {
    let mut end = [255; 72];
    let mut cntr = 0;

    loop {
        let x = cntr % 6;
        if x < 5 {
            end[cntr] = cntr as u64 + 1;
        }

        cntr += 1;
        if cntr >= 72 {
            break;
        }
    }

    end
}

pub const AFFECTED_PAIRS: [u64; 72] = build_pair_array();
