use crate::prelude::*;
use super::{Job, JobState};

/// Placeholder implementations so code compiles.
/// Wire these into your builtin dispatcher when you implement job control.

pub fn builtin_jobs(args: &[String]) -> Result<i32> {
    // Accept flags: -l, -p, -r, -s (ignored for now)
    let _ = args;
    println!("[job control not implemented yet]");
    Ok(0)
}

pub fn builtin_fg(_args: &[String]) -> Result<i32> {
    eprintln!("fg: job control not implemented yet");
    Ok(1)
}

pub fn builtin_bg(_args: &[String]) -> Result<i32> {
    eprintln!("bg: job control not implemented yet");
    Ok(1)
}

pub fn builtin_kill(_args: &[String]) -> Result<i32> {
    eprintln!("kill: job control not implemented yet");
    Ok(1)
}
