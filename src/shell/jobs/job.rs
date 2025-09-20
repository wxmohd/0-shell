#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    Running,
    Stopped,
    Terminated,
}

#[derive(Debug, Clone)]
pub struct Process {
    pub pid: i32,          // placeholder; when you spawn, store child pid
    pub cmdline: String,
}

#[derive(Debug, Clone)]
pub struct Job {
    pub id: usize,         // [%1], [%2], ...
    pub pgid: i32,         // process group id (when implemented)
    pub state: JobState,
    pub procs: Vec<Process>,
    pub is_current: bool,  // '+' marker
}

impl Job {
    pub fn summary(&self) -> String {
        let cmd = self.procs.first().map(|p| p.cmdline.clone()).unwrap_or_default();
        cmd
    }
}
