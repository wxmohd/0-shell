use crate::prelude::*;
use std::fs;
use std::path::Path;
use crate::shell::parser::parse_line;
use crate::shell::exec::run_parsed_command;

/// Execute a script file line-by-line using the same parser + builtins.
pub fn run_script_file(shell: &mut crate::shell::Shell, path: &Path) -> Result<i32> {
    let text = fs::read_to_string(path)?;
    run_script_string(shell, &text)
}

/// Execute a multi-line string as a script (very small language).
pub fn run_script_string(shell: &mut crate::shell::Shell, script: &str) -> Result<i32> {
    for raw in script.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        for cmd in parse_line(line) {
            let status = run_parsed_command(shell, cmd)?;
            shell.last_status = status;
            if status == crate::shell::exec::status::EXIT_SIGNAL {
                return Ok(0);
            }
        }
    }
    Ok(shell.last_status)
}
