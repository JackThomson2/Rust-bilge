macro_rules! safe_get {
    ($arr:expr, $pos:expr) => {
        if cfg!(not(debug_assertions)) {
            unsafe { $arr.get_unchecked($pos) }
        } else {
            $arr.get($pos).unwrap()
        }
    };
}

macro_rules! get_mut_safely {
    ($arr:expr, $pos:expr) => {
        if cfg!(not(debug_assertions)) {
            unsafe { $arr.get_unchecked_mut($pos) }
        } else {
            $arr.get_mut($pos).unwrap()
        }
    };
}


pub trait SafeGetters<T> {
    fn get_safely(&self, idx: usize) -> &T;

    fn get_mut_safely(&mut self, idx: usize) -> &mut T;
}

impl<T> SafeGetters<T> for [T] {
    fn get_safely(&self, idx:usize) -> &T {
        safe_get!(self, idx)
    }

    fn get_mut_safely(&mut self, idx:usize) -> &mut T {
        get_mut_safely!(self, idx)
    }
}

impl <T, const C: usize> SafeGetters<T> for [T; C] {
    fn get_safely(&self, idx:usize) -> &T {
        safe_get!(self, idx)
    }

    fn get_mut_safely(&mut self, idx:usize) -> &mut T {
        get_mut_safely!(self, idx)
    }
}
