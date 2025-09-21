use crate::prelude::*;
use super::{JobTable, JobState};
#[cfg(unix)]
use nix::{sys::signal::{killpg, Signal}, unistd::Pid};

pub fn builtin_jobs(table: &mut JobTable, args: &[String]) -> Result<i32> {
    // Flags: -l (with pid), -p (only pid), -r (running), -s (stopped)
    let mut show_pids_only = false;
    let mut with_pid = false;
    let mut filter: Option<JobState> = None;
    for a in args {
        match a.as_str() {
            "-p" => show_pids_only = true,
            "-l" => with_pid = true,
            "-r" => filter = Some(JobState::Running),
            "-s" => filter = Some(JobState::Stopped),
            _ => {}
        }
    }

    for j in table.jobs.iter() {
        if let Some(f) = filter {
            if j.state != f { continue; }
        }
        if show_pids_only {
            #[cfg(unix)]
            println!("{}", j.pgid.as_raw());
            #[cfg(not(unix))]
            println!("0");
            continue;
        }
        let mark = if j.current { '+' } else { ' ' };
        let status = match j.state {
            JobState::Running => "Running",
            JobState::Stopped => "Stopped",
            JobState::Terminated => "Terminated",
        };
        if with_pid {
            #[cfg(unix)]
            println!("[{}]{} {:<6} {}", j.id, mark, j.pgid.as_raw(), status.to_string() + "                 " + &j.summary());
            #[cfg(not(unix))]
            println!("[{}]{} {:<6} {}", j.id, mark, 0, status.to_string() + "                 " + &j.summary());
        } else {
            println!("[{}]{}  {:<10}  {} &", j.id, mark, status, j.summary());
        }
    }
    Ok(0)
}

pub fn builtin_bg(table: &mut JobTable, args: &[String]) -> Result<i32> {
    #[cfg(unix)]
    {
        let j = resolve_job_mut(table, args)?;
        if j.state == JobState::Stopped {
            killpg(j.pgid, Signal::SIGCONT).ok();
            j.state = JobState::Running;
            println!("[{}]+ {} &", j.id, j.summary());
            return Ok(0);
        }
        eprintln!("bg: job is not stopped");
        return Ok(1);
    }
    #[cfg(not(unix))]
    {
        eprintln!("bg: job control not available on this platform");
        Ok(1)
    }
}

pub fn builtin_fg(shell: &mut crate::shell::Shell, args: &[String]) -> Result<i32> {
    #[cfg(unix)]
    {
        use crate::shell::signals::tty;
        use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};

        let j = resolve_job_mut(&mut shell.jobs, args)?;
        // Give terminal to job and continue
        tty::give_terminal_to(j.pgid).ok();
        nix::sys::signal::killpg(j.pgid, nix::sys::signal::Signal::SIGCONT).ok();
        j.state = JobState::Running;

        // Wait until it exits or stops again
        loop {
            match waitpid(nix::unistd::Pid::from_raw(-j.pgid.as_raw()), Some(WaitPidFlag::WUNTRACED)) {
                Ok(WaitStatus::Exited(_, _)) | Ok(WaitStatus::Signaled(_, _, _)) => {
                    // Mark as terminated; JobTable reaper will print when update occurs
                    shell.jobs.jobs.retain(|x| x.id != j.id);
                    break;
                }
                Ok(WaitStatus::Stopped(_, _)) => {
                    j.state = JobState::Stopped;
                    println!("[{}]+  Stopped                 {}", j.id, j.summary());
                    break;
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }

        // Take terminal back
        tty::give_terminal_back_to_shell().ok();
        return Ok(0);
    }
    #[cfg(not(unix))]
    {
        eprintln!("fg: job control not available on this platform");
        Ok(1)
    }
}

pub fn builtin_kill(table: &mut JobTable, args: &[String]) -> Result<i32> {
    #[cfg(unix)]
    {
        if args.is_empty() {
            eprintln!("kill: usage: kill <pid>|%<jobid>");
            return Ok(1);
        }
        let target = &args[0];
        if let Some(j) = table.by_percent(target) {
            nix::sys::signal::killpg(j.pgid, nix::sys::signal::Signal::SIGTERM).ok();
            return Ok(0);
        }
        if let Ok(pid) = target.parse::<i32>() {
            nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), nix::sys::signal::Signal::SIGTERM).ok();
            return Ok(0);
        }
        eprintln!("kill: invalid target '{}'", target);
        Ok(1)
    }
    #[cfg(not(unix))]
    {
        eprintln!("kill: job control not available on this platform");
        Ok(1)
    }
}

#[cfg(unix)]
fn resolve_job_mut<'a>(table: &'a mut JobTable, args: &[String]) -> Result<&'a mut super::job::Job> {
    if let Some(tok) = args.get(0) {
        if tok.starts_with('%') || tok == "%+" || tok == "%-" {
            if let Some(j) = table.by_percent(tok) {
                return Ok(j);
            }
        } else if let Ok(id) = tok.parse::<usize>() {
            if let Some(j) = table.by_id(id) {
                return Ok(j);
            }
        }
    }
    // default: current '+'
    for j in table.jobs.iter_mut().rev() {
        if j.current {
            return Ok(j);
        }
    }
    Err("fg/bg: no current job".into())
}
