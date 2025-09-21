use std::collections::HashMap;
use std::env;

/// Expand $VAR using shell vars first, then OS env.
pub fn expand_vars(tokens: &[String], vars: &HashMap<String, String>) -> Vec<String> {
    tokens.iter().map(|t| {
        if let Some(name) = t.strip_prefix('$') {
            if let Some(v) = vars.get(name) {
                v.clone()
            } else {
                env::var(name).unwrap_or_default()
            }
        } else {
            t.clone()
        }
    }).collect()
}
