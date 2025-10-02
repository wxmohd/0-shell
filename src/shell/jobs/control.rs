use crate::prelude::*;
use super::{JobTable, JobState};

#[cfg(unix)]
use nix::{
    sys::{
        signal::{kill, killpg, Signal},
        wait::{waitpid, WaitPidFlag, WaitStatus},
    },
    unistd::Pid,
};

/// Print jobs, with support for:
/// -l (include pid), -p (only pid), -r (running only), -s (stopped only)
pub fn builtin_jobs(table: &mut JobTable, args: &[String]) -> Result<i32> {
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

    let (cur_id, prev_id) = table.current_prev_ids();

    for j in table.jobs.iter() {
        if let Some(f) = filter {
            if j.state != f { continue; }
        }
        if show_pids_only {
            #[cfg(unix)]
            { println!("{}", j.pgid.as_raw()); }
            #[cfg(not(unix))]
            { println!("0"); }
            continue;
        }

        let mark = if Some(j.id) == cur_id { '+' }
                   else if Some(j.id) == prev_id { '-' }
                   else { ' ' };

        let status = match j.state {
            JobState::Running => "Running",
            JobState::Stopped => "Stopped",
            JobState::Terminated => "Terminated",
        };

        // Append " &" only for Running background jobs to mirror the audit examples.
        let trailer = if j.state == JobState::Running { " &" } else { "" };

        if with_pid {
            #[cfg(unix)]
            println!("[{}]{}  {:<6} {:<20} {}{}", j.id, mark, j.pgid.as_raw(), status, j.summary(), trailer);
            #[cfg(not(unix))]
            println!("[{}]{}  {:<6} {:<20} {}{}", j.id, mark, 0, status, j.summary(), trailer);
        } else {
            println!("[{}]{}  {:<10}  {}{}", j.id, mark, status, j.summary(), trailer);
        }
    }
    Ok(0)
}

pub fn builtin_bg(table: &mut JobTable, args: &[String]) -> Result<i32> {
    #[cfg(unix)]
    {
        let idx = resolve_job_index(table, args)?;
        if table.jobs[idx].state == JobState::Stopped {
            let pgid = table.jobs[idx].pgid;
            killpg(pgid, Signal::SIGCONT).ok();
            table.jobs[idx].state = JobState::Running;
            let summary = table.jobs[idx].summary();
            let id = table.jobs[idx].id;
            println!("[{}]+ {} &", id, summary);
            return Ok(0);
        }
        eprintln!("bg: job is not stopped");
        Ok(1)
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

        let idx = resolve_job_index(&shell.jobs, args)?;
        let pgid = shell.jobs.jobs[idx].pgid;
        let id = shell.jobs.jobs[idx].id;

        // Give terminal to job and continue it
        tty::give_terminal_to(pgid).ok();
        killpg(pgid, Signal::SIGCONT).ok();
        shell.jobs.jobs[idx].state = JobState::Running;

        // Wait until it exits or stops again
        let mut exited = false;
        loop {
            match waitpid(Pid::from_raw(-pgid.as_raw()), Some(WaitPidFlag::WUNTRACED)) {
                Ok(WaitStatus::Exited(_, _)) | Ok(WaitStatus::Signaled(_, _, _)) => {
                    exited = true;
                    break;
                }
                Ok(WaitStatus::Stopped(_, _)) => {
                    // mark stopped and print
                    if let Some(j) = shell.jobs.jobs.iter().find(|j| j.id == id) {
                        println!("[{}]+  Stopped                 {}", j.id, j.summary());
                    } else {
                        println!("[{}]+  Stopped", id);
                    }
                    if let Some(j) = shell.jobs.jobs.iter_mut().find(|j| j.id == id) {
                        j.state = JobState::Stopped;
                    }
                    break;
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }

        // Take terminal back to the shell
        tty::give_terminal_back_to_shell().ok();

        // If it exited, remove from table after we dropped &mut borrows
        if exited {
            shell.jobs.jobs.retain(|x| x.id != id);
        }
        Ok(0)
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

        // %n notation
        if let Some(i) = table.index_by_percent(target) {
            let pgid = table.jobs[i].pgid;
            killpg(pgid, Signal::SIGTERM).ok();
            return Ok(0);
        }
        // raw PID
        if let Ok(pid) = target.parse::<i32>() {
            kill(Pid::from_raw(pid), Signal::SIGTERM).ok();
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
fn resolve_job_index(table: &JobTable, args: &[String]) -> Result<usize> {
    if let Some(tok) = args.get(0) {
        if tok.starts_with('%') || tok == "%+" || tok == "%-" {
            if let Some(i) = table.index_by_percent(tok) {
                return Ok(i);
            }
        } else if let Ok(id) = tok.parse::<usize>() {
            if let Some(i) = table.index_by_id(id) {
                return Ok(i);
            }
        }
    }
    // default to current '+'
    if let Some(i) = table.jobs.iter().rposition(|j| j.current) {
        return Ok(i);
    }
    Err("fg/bg: no current job".into())
}
