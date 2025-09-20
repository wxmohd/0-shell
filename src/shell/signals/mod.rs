#[cfg(unix)]
pub fn install_handlers() {
    // Later:
    // - set up SIGINT (Ctrl+C) to interrupt fg job
    // - set up SIGTSTP (Ctrl+Z) to stop fg job
    // - set up SIGCHLD to reap children
}

#[cfg(not(unix))]
pub fn install_handlers() {
    // No-op on non-Unix
}
