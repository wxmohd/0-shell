use std::collections::VecDeque;

#[cfg(unix)]
use nix::unistd::Pid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    Running,
    Stopped,
    Terminated,
}

#[derive(Debug, Clone)]
pub struct Job {
    pub id: usize,         // [%1], [%2], ...
    #[cfg(unix)]
    pub pgid: Pid,
    #[cfg(not(unix))]
    pub pgid: i32,         // placeholder on non-Unix
    pub state: JobState,
    pub cmdline: String,
    pub pids: Vec<i32>,
    pub current: bool,     // '+' marker
}

impl Job {
    pub fn summary(&self) -> String {
        self.cmdline.clone()
    }
}

#[derive(Default)]
pub struct JobTable {
    pub jobs: VecDeque<Job>,
    next_id: usize,
}

impl JobTable {
    #[cfg(unix)]
    pub fn add_job(&mut self, pgid: Pid, state: JobState, cmd: String, args: Vec<String>) -> usize {
        self.next_id += 1;
        let id = self.next_id;
        for j in self.jobs.iter_mut() { j.current = false; }
        let mut cmdline = cmd;
        if !args.is_empty() {
            cmdline.push(' ');
            cmdline.push_str(&args.join(" "));
        }
        self.jobs.push_back(Job {
            id,
            pgid,
            state,
            cmdline,
            pids: vec![pgid.as_raw()],
            current: true, // newest is current (+)
        });
        id
    }

    #[cfg(not(unix))]
    pub fn add_job(&mut self, _pgid: i32, state: JobState, cmd: String, args: Vec<String>) -> usize {
        self.next_id += 1;
        let id = self.next_id;
        for j in self.jobs.iter_mut() { j.current = false; }
        let mut cmdline = cmd;
        if !args.is_empty() {
            cmdline.push(' ');
            cmdline.push_str(&args.join(" "));
        }
        self.jobs.push_back(Job {
            id,
            pgid: 0,
            state,
            cmdline,
            pids: vec![],
            current: true,
        });
        id
    }

    pub fn by_id(&mut self, id: usize) -> Option<&mut Job> {
        self.jobs.iter_mut().find(|j| j.id == id)
    }

    pub fn by_percent(&mut self, s: &str) -> Option<&mut Job> {
        // %1, %+ (current), %- (previous)
        if s == "%+" {
            return self.jobs.iter_mut().rev().find(|j| j.current);
        }
        if s == "%-" {
            if self.jobs.len() > 1 {
                let mut iter = self.jobs.iter_mut().rev();
                let _cur = iter.next();
                return iter.next();
            }
            return None;
        }
        if let Some(num) = s.strip_prefix('%') {
            if let Ok(id) = num.parse::<usize>() {
                return self.by_id(id);
            }
        }
        None
    }

    /// Return (current_job_id, previous_job_id) for `+` and `-` markers.
    pub fn current_prev_ids(&self) -> (Option<usize>, Option<usize>) {
        if self.jobs.is_empty() {
            return (None, None);
        }
        if let Some((i, _)) = self.jobs.iter().enumerate().rev().find(|(_, j)| j.current) {
            let cur = Some(self.jobs[i].id);
            let prev = if i >= 1 { Some(self.jobs[i - 1].id) } else { None };
            return (cur, prev);
        }
        let last = self.jobs.back().map(|j| j.id);
        let prev = if self.jobs.len() >= 2 {
            self.jobs.get(self.jobs.len() - 2).map(|j| j.id)
        } else {
            None
        };
        (last, prev)
    }

    // ---- New index-based helpers (immutable) ----

    pub fn index_by_id(&self, id: usize) -> Option<usize> {
        self.jobs.iter().position(|j| j.id == id)
    }

    pub fn index_by_percent(&self, s: &str) -> Option<usize> {
        if s == "%+" {
            return self.jobs.iter().rposition(|j| j.current);
        }
        if s == "%-" {
            if self.jobs.len() > 1 {
                return Some(self.jobs.len() - 2);
            }
            return None;
        }
        if let Some(num) = s.strip_prefix('%') {
            if let Ok(id) = num.parse::<usize>() {
                return self.index_by_id(id);
            }
        }
        None
    }
}

#[cfg(unix)]
#[derive(Debug, Clone, Copy)]
pub enum UpdateKind {
    Running { pgid: Pid },
    Stopped { pgid: Pid },
    Terminated { pgid: Pid, code: i32 },
}

#[cfg(unix)]
impl UpdateKind {
    pub fn from_waitstatus(ws: nix::sys::wait::WaitStatus) -> Option<Self> {
        use nix::sys::wait::WaitStatus::*;
        match ws {
            Exited(pid, code) => Some(UpdateKind::Terminated { pgid: pid, code }),
            Signaled(pid, _sig, _core) => Some(UpdateKind::Terminated { pgid: pid, code: 128 }),
            Stopped(pid, _sig) => Some(UpdateKind::Stopped { pgid: pid }),
            Continued(pid) => Some(UpdateKind::Running { pgid: pid }),
            StillAlive => None,
            PtraceEvent(_, _, _) | PtraceSyscall(_) => None,
        }
    }
}

#[cfg(unix)]
impl JobTable {
    pub fn apply_update(&mut self, upd: UpdateKind) {
        match upd {
            UpdateKind::Running { pgid } => {
                if let Some(j) = self.jobs.iter_mut().find(|j| j.pgid == pgid) {
                    j.state = JobState::Running;
                }
            }
            UpdateKind::Stopped { pgid } => {
                if let Some(j) = self.jobs.iter_mut().find(|j| j.pgid == pgid) {
                    j.state = JobState::Stopped;
                    println!("[{}]+  Stopped                 {}", j.id, j.summary());
                }
            }
            UpdateKind::Terminated { pgid, .. } => {
                if let Some(pos) = self.jobs.iter().position(|j| j.pgid == pgid) {
                    let j = self.jobs.remove(pos).unwrap();
                    println!("[{}]+  Terminated              {}", j.id, j.summary());
                    if let Some(last) = self.jobs.back_mut() {
                        last.current = true;
                    }
                }
            }
        }
    }
}
