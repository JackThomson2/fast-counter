macro_rules! safe_get {
    ($arr:expr, $pos:expr) => {
        if cfg!(not(debug_assertions)) {
            unsafe { $arr.get_unchecked($pos) }
        } else {
            $arr.get($pos).unwrap()
        }
    };
}

macro_rules! debug_unwrap {
    ($arr:expr) => {
        if cfg!(not(debug_assertions)) {
            $arr.unwrap_unchecked()
        } else {
            $arr.unwrap()
        }
    };
}


pub trait SafeGetters<T> {
    fn safely_get(&self, idx: usize) -> &T;
}

impl<T> SafeGetters<T> for [T] {
    fn safely_get(&self, idx:usize) -> &T {
        safe_get!(self, idx)
    }
}

impl <T, const C: usize> SafeGetters<T> for [T; C] {
    fn safely_get(&self, idx:usize) -> &T {
        safe_get!(self, idx)
    }
}