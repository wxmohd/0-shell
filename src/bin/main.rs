mod prelude;
mod shell;

use prelude::*;
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    // If an argument is provided, treat it as a script path (to be implemented later).
    // For now, always start interactive REPL.
    let args: Vec<String> = env::args().collect();

    let mut sh = shell::Shell::new()?;

    if args.len() > 1 {
        // Placeholder: run script file (we’ll implement in scripting module later)
        let script_path = PathBuf::from(&args[1]);
        eprintln!("(scripting not implemented yet) Would run script: {}", script_path.display());
        Ok(())
    } else {
        sh.repl()
    }
}
