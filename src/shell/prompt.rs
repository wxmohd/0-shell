use std::env;

pub fn render_prompt() -> String {
    // Minimal: "$ " ; bonus later: current dir, colors, etc.
    if let Ok(cwd) = env::current_dir() {
        if let Some(name) = cwd.file_name().and_then(|s| s.to_str()) {
            return format!("{name} $ ");
        }
    }
    "$ ".into()
}
