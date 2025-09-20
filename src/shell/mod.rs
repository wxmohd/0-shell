pub mod repl;
pub mod prompt;
pub mod input;
pub mod parser;
pub mod exec;

use crate::prelude::*;
use repl::Repl;

pub struct Shell {
    pub last_status: i32,
}

impl Shell {
    pub fn new() -> Result<Self> {
        Ok(Self { last_status: 0 })
    }

    pub fn repl(&mut self) -> Result<()> {
        let mut repl = Repl::new();
        repl.run(self)
    }
}
