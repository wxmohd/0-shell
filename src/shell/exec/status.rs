/// 0 = success, non-zero = error.
pub const EXIT_SIGNAL: i32 = -7777;

#[inline]
pub fn ok() -> i32 { 0 }

#[inline]
pub fn err() -> i32 { 1 }
