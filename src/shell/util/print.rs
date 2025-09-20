use std::io::{self, Write};

pub fn eprintln_flush<S: AsRef<str>>(s: S) {
    let _ = writeln!(&mut io::stderr().lock(), "{}", s.as_ref());
}
