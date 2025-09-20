#[cfg(unix)]
pub fn give_terminal_to(_pgid: libc::pid_t) -> std::io::Result<()> {
    // Use tcsetpgrp(STDIN_FILENO, pgid) later
    Ok(())
}

#[cfg(not(unix))]
pub fn give_terminal_to(_pgid: i32) -> std::io::Result<()> {
    Ok(())
}
