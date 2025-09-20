use crate::prelude::*;
use std::fs;
use std::path::Path;
use crate::shell::parser::parse_command;
use crate::shell::exec::dispatch_builtin;

/// Execute a script file line-by-line using the same parser + builtins.
pub fn run_script_file(shell: &mut crate::shell::Shell, path: &Path) -> Result<i32> {
    let text = fs::read_to_string(path)?;
    run_script_string(shell, &text)
}

/// Execute a multi-line string as a script.
pub fn run_script_string(shell: &mut crate::shell::Shell, script: &str) -> Result<i32> {
    for raw_line in script.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // TODO: handle multi-line constructs (for, if, functions)
        if let Some((cmd, args)) = parse_command(line) {
            match dispatch_builtin(shell, cmd.clone(), &args) {
                Ok(status) => shell.last_status = status,
                Err(e) => {
                    eprintln!("script error: {e}");
                    shell.last_status = 1;
                }
            }
            if cmd == "exit" {
                break;
            }
        }
    }
    Ok(shell.last_status)
}
