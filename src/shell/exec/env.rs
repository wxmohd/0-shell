use std::env;

/// Very small env expander: expands leading $VAR in tokens. (Bonus-ready)
pub fn expand_vars(tokens: &[String]) -> Vec<String> {
    tokens.iter().map(|t| {
        if let Some(stripped) = t.strip_prefix('$') {
            env::var(stripped).unwrap_or_default()
        } else {
            t.clone()
        }
    }).collect()
}
