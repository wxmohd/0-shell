pub mod builtins;
pub mod status;
pub mod fileops;
pub mod env;

use crate::prelude::*;
use crate::shell::parser::ParsedCommand;

pub use builtins::dispatch_builtin;

#[cfg(unix)]
use nix::{
    sys::wait::{waitpid, WaitPidFlag, WaitStatus},
    unistd::{execve, fork, getpid, setpgid, ForkResult, Pid},
};
#[cfg(unix)]
use std::{ffi::CString, os::unix::ffi::OsStrExt, path::PathBuf};

/// Run one parsed command (maybe background). Returns exit status.
/// If the builtin was `exit`, return EXIT_SIGNAL so caller can break the REPL.
pub fn run_parsed_command(shell: &mut crate::shell::Shell, p: ParsedCommand) -> Result<i32> {
    // Builtins first (jobs/bg/fg/kill/sleep/etc.)
    if builtins::is_builtin(&p.cmd) {
        return builtins::dispatch_builtin(shell, p.cmd, &p.args);
    }

    // Otherwise try external (Unix). On non-Unix, print not found.
    #[cfg(unix)]
    {
        return run_external(shell, &p.cmd, &p.args, p.background);
    }

    #[cfg(not(unix))]
    {
        eprintln!("Command '{}' not found", p.cmd);
        Ok(127)
    }
}

/// Opportunistic reaper to keep job table fresh (Unix)
pub fn maybe_reap(shell: &mut crate::shell::Shell) {
    #[cfg(unix)]
    {
        use crate::shell::jobs::UpdateKind;
        loop {
            match waitpid(Pid::from_raw(-1), Some(WaitPidFlag::WNOHANG | WaitPidFlag::WUNTRACED | WaitPidFlag::WCONTINUED)) {
                Ok(WaitStatus::StillAlive) => break,
                Ok(status) => {
                    if let Some(upd) = UpdateKind::from_waitstatus(status) {
                        shell.jobs.apply_update(upd);
                    }
                }
                Err(_) => break,
            }
        }
    }
}

/// Search PATH for program (Unix)
#[cfg(unix)]
fn which(cmd: &str) -> Option<PathBuf> {
    use std::{env, fs};
    if cmd.contains('/') {
        let p = PathBuf::from(cmd);
        if p.exists() { return Some(p); }
        return None;
    }
    let path = env::var("PATH").ok()?;
    for dir in path.split(':') {
        let cand = PathBuf::from(dir).join(cmd);
        if let Ok(md) = fs::metadata(&cand) {
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                if md.is_file() && (md.mode() & 0o111) != 0 {
                    return Some(cand);
                }
            }
        }
    }
    None
}

/// Exec external with job-control semantics (Unix)
#[cfg(unix)]
fn run_external(shell: &mut crate::shell::Shell, cmd: &str, args: &[String], background: bool) -> Result<i32> {
    use crate::shell::jobs::{JobState, UpdateKind};
    use crate::shell::signals::tty;

    let program = match which(cmd) {
        Some(p) => p,
        None => {
            eprintln!("Command '{}' not found", cmd);
            return Ok(127);
        }
    };

    // Build argv for execve
    let mut argv: Vec<CString> = Vec::with_capacity(args.len() + 1);
    argv.push(CString::new(program.as_os_str().as_bytes().to_vec()).unwrap());
    for a in args {
        argv.push(CString::new(a.as_str()).unwrap());
    }
    let envp: Vec<CString> = std::env::vars().map(|(k, v)| CString::new(format!("{k}={v}")).unwrap()).collect();

    // Fork
    match unsafe { fork() }? {
        ForkResult::Child => {
            // Child: new process group
            let pid = getpid();
            let _ = setpgid(pid, pid);

            if !background {
                let _ = tty::give_terminal_to(pid);
            }

            // exec
            match execve(&CString::new(program.as_os_str().as_bytes().to_vec()).unwrap(), &argv, &envp) {
                Ok(_) => unreachable!(),
                Err(e) => {
                    eprintln!("exec: {}: {}", cmd, e);
                    std::process::exit(127);
                }
            }
        }
        ForkResult::Parent { child } => {
            // Parent: set child's pgid
            let _ = setpgid(child, child);

            // Register job
            let id = shell.jobs.add_job(child, JobState::Running, cmd.to_string(), args.to_vec());

            if background {
                println!("[{}] {}", id, child.as_raw());
                Ok(0)
            } else {
                // Give terminal, wait, then take back
                let _ = tty::give_terminal_to(child);
                let status = wait_foreground(shell, child);
                let _ = tty::give_terminal_back_to_shell();
                status
            }
        }
    }
}

#[cfg(unix)]
fn wait_foreground(shell: &mut crate::shell::Shell, pgid: Pid) -> Result<i32> {
    use crate::shell::jobs::UpdateKind;
    loop {
        match waitpid(Pid::from_raw(-pgid.as_raw()), Some(WaitPidFlag::WUNTRACED | WaitPidFlag::WCONTINUED)) {
            Ok(WaitStatus::Exited(_, code)) => {
                shell.jobs.apply_update(UpdateKind::Terminated { pgid, code });
                return Ok(code);
            }
            Ok(WaitStatus::Signaled(_, _sig, _core)) => {
                shell.jobs.apply_update(UpdateKind::Terminated { pgid, code: 128 });
                return Ok(128);
            }
            Ok(WaitStatus::Stopped(_, _sig)) => {
                shell.jobs.apply_update(UpdateKind::Stopped { pgid });
                return Ok(0);
            }
            Ok(WaitStatus::Continued(_)) => {
                shell.jobs.apply_update(UpdateKind::Running { pgid });
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("wait: {e}");
                return Ok(1);
            }
        }
    }
}
