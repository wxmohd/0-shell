use super::{exec::dispatch_builtin, input::read_line, parser::parse_command, prompt::render_prompt};
use crate::prelude::*;

pub struct Repl;

impl Repl {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self, shell: &mut crate::shell::Shell) -> Result<()> {
        loop {
            // Print prompt
            print!("{}", render_prompt());
            io::stdout().flush()?;

            // Read a line (Ctrl+D returns None)
            let Some(line) = read_line()? else {
                // EOF -> exit gracefully
                println!();
                break;
            };

            // Skip empty
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Parse into (cmd, args)
            let Some((cmd, args)) = parse_command(line) else {
                continue;
            };

            // Dispatch builtins (no external binaries allowed)
            match dispatch_builtin(shell, cmd, &args) {
                Ok(status) => shell.last_status = status,
                Err(e) => {
                    eprintln!("Error: {e}");
                    shell.last_status = 1;
                }
            }

            // If exit was requested, break
            if cmd == "exit" {
                break;
            }
        }
        Ok(())
    }
}
