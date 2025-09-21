use std::fs::Metadata;
use std::path::Path;

#[inline]
pub fn is_hidden(name: &str) -> bool {
    name.starts_with('.')
}

#[cfg(unix)]
fn mode_triplet(bits: u32) -> String {
    format!(
        "{}{}{}",
        if bits & 0o4 != 0 { 'r' } else { '-' },
        if bits & 0o2 != 0 { 'w' } else { '-' },
        if bits & 0o1 != 0 { 'x' } else { '-' },
    )
}

#[cfg(unix)]
pub fn mode_string(md: &Metadata) -> String {
    use std::os::unix::fs::MetadataExt;
    let file_type = if md.is_dir() { 'd' } else { '-' };
    let perm = md.mode() & 0o777;
    let owner = mode_triplet((perm >> 6) & 0o7);
    let group = mode_triplet((perm >> 3) & 0o7);
    let other = mode_triplet(perm & 0o7);
    format!("{file_type}{owner}{group}{other}")
}

#[cfg(not(unix))]
pub fn mode_string(md: &Metadata) -> String {
    let file_type = if md.is_dir() { 'd' } else { '-' };
    // Windows doesn’t expose Unix perms; show placeholders.
    format!("{file_type}---------")
}

#[cfg(unix)]
pub fn classify_suffix(path: &Path, is_dir: bool) -> &'static str {
    use std::os::unix::fs::MetadataExt;
    if is_dir {
        "/"
    } else {
        if let Ok(md) = path.metadata() {
            if md.mode() & 0o111 != 0 { "*" } else { "" }
        } else { "" }
    }
}

#[cfg(not(unix))]
pub fn classify_suffix(_path: &Path, is_dir: bool) -> &'static str {
    // On non-Unix, just mark directories; we don’t check executables.
    if is_dir { "/" } else { "" }
}
