use std::fs::{self, Metadata};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

pub fn is_hidden(name: &str) -> bool {
    name.starts_with('.')
}

pub fn mode_string(md: &Metadata) -> String {
    // Minimal -l mode string (not perfectly POSIX, but good starter)
    let file_type = if md.is_dir() { 'd' } else { '-' };
    let perm = md.mode() & 0o777;
    let to_triplet = |p: u32| -> String {
        format!(
            "{}{}{}",
            if p & 0o4 != 0 { 'r' } else { '-' },
            if p & 0o2 != 0 { 'w' } else { '-' },
            if p & 0o1 != 0 { 'x' } else { '-' },
        )
    };
    let s = format!(
        "{}{}{}{}",
        file_type,
        to_triplet((perm >> 6) & 0o7),
        to_triplet((perm >> 3) & 0o7),
        to_triplet(perm & 0o7)
    );
    s
}

pub fn classify_suffix(path: &Path, is_dir: bool) -> &'static str {
    // -F: append "/" for dirs, "*" for executables (simplified)
    if is_dir {
        "/"
    } else {
        // On Unix, check executable bit
        if let Ok(md) = path.metadata() {
            if md.mode() & 0o111 != 0 { "*" } else { "" }
        } else { "" }
    }
}
