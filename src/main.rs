mod prelude;
mod shell;

use crate::prelude::*;
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut sh = shell::Shell::new()?;

    if args.len() > 1 {
        let script_path = PathBuf::from(&args[1]);
        shell::scripting::run_script_file(&mut sh, &script_path).map(|_| ())
    } else {
        sh.repl()
    }
}
