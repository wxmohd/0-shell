use crate::prelude::*;
use std::io::Read;

/// Returns Some(line) or None if EOF (Ctrl+D)
pub fn read_line() -> Result<Option<String>> {
    let mut buf = String::new();
    let n = io::stdin().read_line(&mut buf)?;
    if n == 0 {
        // EOF
        return Ok(None);
    }
    Ok(Some(buf))
}
