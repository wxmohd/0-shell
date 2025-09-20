pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub use std::io::{self, Write};
