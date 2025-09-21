use super::{Job, JobState};

pub fn format_jobs_simple(jobs: &[Job]) -> String {
    let mut out = String::new();
    for j in jobs {
        let mark = if j.current { '+' } else { ' ' };
        let status = match j.state {
            JobState::Running => "Running",
            JobState::Stopped => "Stopped",
            JobState::Terminated => "Terminated",
        };
        out.push_str(&format!("[{}]{}  {:<10}  {}\n", j.id, mark, status, j.summary()));
    }
    out
}
