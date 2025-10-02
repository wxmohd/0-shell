use super::{exec, input::read_line_with_history, parser, prompt::render_prompt};
use crate::prelude::*;

pub struct Repl;

impl Repl { pub fn new() -> Self { Self } }

impl Repl {
    pub fn run(&mut self, shell: &mut crate::shell::Shell) -> Result<()> {
        loop {
            // If background jobs printed, start prompt on a fresh line.
            if exec::maybe_reap(shell) { println!(); }

            let prompt = render_prompt(); 
            let Some(line) = read_line_with_history(&prompt, &mut shell.history)? else {
                println!();
                break;
            };

            let line = line.trim();
            if line.is_empty() { continue; }

            for cmd in parser::parse_line(line) {
                let status = exec::run_parsed_command(shell, cmd)?;
                shell.last_status = status;
                if status == exec::status::EXIT_SIGNAL { return Ok(()); }
            }
        }
        Ok(())
    }
}
