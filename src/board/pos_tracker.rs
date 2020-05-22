use bit_array::BitArray;

use typenum::U72;

pub type PositionArr = BitArray<u64, U72>;

pub struct PositionTracker {
    inner: PositionArr,
}

impl PositionTracker {
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    #[inline]
    pub fn set_visible(&mut self, position: usize) {
        self.inner.set(position, true);
    }

    #[inline]
    pub fn set_invisible(&mut self, position: usize) {
        self.inner.set(position, false);
    }

    #[inline]
    pub fn get_inner(&self) -> &PositionArr {
        &self.inner
    }
}

#[inline]
pub fn NewTracker(pos: usize) -> PositionTracker {
    let mut new_tracker: PositionArr = BitArray::from_elem(false);
    new_tracker.set(pos, true);
    new_tracker.set(pos + 1, true);

    PositionTracker { inner: new_tracker }
}
