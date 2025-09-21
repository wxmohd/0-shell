use super::{exec, input::read_line, parser, prompt::render_prompt};
use crate::prelude::*;

pub struct Repl;

impl Repl {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self, shell: &mut crate::shell::Shell) -> Result<()> {
        loop {
            // Opportunistic reap (Unix): update job statuses between prompts
            exec::maybe_reap(shell);

            print!("{}", render_prompt());
            io::stdout().flush()?;

            let Some(line) = read_line()? else {
                println!(); // Ctrl+D newline
                break;
            };
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Parse possibly multiple commands separated by ';'
            let cmds = parser::parse_line(line);
            for cmd in cmds {
                // `exit` handled as builtin; keep running until it returns
                let status = exec::run_parsed_command(shell, cmd)?;
                shell.last_status = status;
                if status == exec::status::EXIT_SIGNAL {
                    return Ok(());
                }
            }
        }
        Ok(())
    }
}
