/// Unix-like exit statuses: 0 = success, non-zero = error.
#[inline]
pub fn ok() -> i32 { 0 }

#[inline]
pub fn err() -> i32 { 1 }
