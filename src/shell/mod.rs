pub mod repl;
pub mod prompt;
pub mod input;
pub mod parser;    // contains lexer/ast/grammar
pub mod exec;
pub mod jobs;
pub mod signals;
pub mod scripting;

use crate::prelude::*;
use jobs::JobTable;
use std::collections::HashMap;

#[cfg(unix)]
use nix::unistd::{getpid, Pid};

pub struct Shell {
    pub last_status: i32,
    pub vars: HashMap<String, String>,
    pub jobs: JobTable,
    pub history: Vec<String>, // if you added Up/Down support

    #[cfg(unix)]
    pub shell_pgid: Pid,
}

impl Shell {
    pub fn new() -> Result<Self> {
        #[cfg(unix)]
        let shell_pgid = {
            let pid = getpid();
            signals::install_handlers();
            signals::tty::take_control_of_terminal(pid).ok();
            pid
        };

        Ok(Self {
            last_status: 0,
            vars: HashMap::new(),
            jobs: JobTable::default(),
            history: Vec::new(),
            #[cfg(unix)]
            shell_pgid,
        })
    }

    pub fn repl(&mut self) -> Result<()> {
        let mut repl = repl::Repl::new();
        repl.run(self)
    }
}
