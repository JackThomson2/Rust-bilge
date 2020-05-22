use bit_array::BitArray;

use typenum::U6;

pub type RowArr = BitArray<u8, U6>;

pub type ShifterTracked = (usize, RowArr);

pub struct RowTracker {
    inner: RowArr,
}

impl RowTracker {
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    #[inline]
    pub fn set_visible(&mut self, position: usize) {
        self.inner.set(position, true);
    }

    #[inline]
    pub fn get_inner(&self) -> &RowArr {
        &self.inner
    }
}

#[inline]
pub fn new_row_tracker() -> RowTracker {
    let new_tracker: RowArr = BitArray::from_elem(false);

    RowTracker { inner: new_tracker }
}
