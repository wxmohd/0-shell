pub use std::io;
pub use std::io::Write;

pub type AnyError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T> = std::result::Result<T, AnyError>;
