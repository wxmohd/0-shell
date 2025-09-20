use std::time::{SystemTime, UNIX_EPOCH};

/// Format an mtime-like string (placeholder)
pub fn format_mtime(_t: SystemTime) -> String {
    // Keep simple for now
    "-------- -- --:--".to_string()
}

pub fn now_unix() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}
