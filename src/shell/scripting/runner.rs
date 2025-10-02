use crate::prelude::*;
use std::fs;
use std::path::Path;

pub fn run_script_file(shell: &mut crate::shell::Shell, path: &Path) -> Result<i32> {
    let src = fs::read_to_string(path)?;
    run_script_string(shell, &src)
}

#[allow(dead_code)]
pub fn run_script_string(shell: &mut crate::shell::Shell, src: &str) -> Result<i32> {
    let mut last = 0;
    for (lineno, raw) in src.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // very simple: support ';' chaining already handled by parser
        for cmd in crate::shell::parser::parse_line(line) {
            last = crate::shell::exec::run_parsed_command(shell, cmd)?;
            if last == crate::shell::exec::status::EXIT_SIGNAL {
                return Ok(0);
            }
        }
    }
    Ok(last)
}
