#[cfg(unix)]
use nix::{libc, unistd::Pid};

#[cfg(unix)]
pub fn take_control_of_terminal(pgid: Pid) -> std::io::Result<()> {
    give_terminal_to(pgid)
}

#[cfg(unix)]
pub fn give_terminal_to(pgid: Pid) -> std::io::Result<()> {
    let fd = libc::STDIN_FILENO;
    let r = unsafe { libc::tcsetpgrp(fd, pgid.as_raw()) };
    if r == -1 {
        // Use std::io::Error::last_os_error() instead of nix::Errno conversion
        let e = std::io::Error::last_os_error();
        // Not fatal if there is no controlling TTY
        if e.raw_os_error() == Some(libc::ENOTTY) {
            return Ok(());
        }
        return Err(e);
    }
    Ok(())
}

#[cfg(unix)]
pub fn give_terminal_back_to_shell() -> std::io::Result<()> {
    let pgid = nix::unistd::getpgrp();
    give_terminal_to(pgid)
}

#[cfg(not(unix))]
pub fn take_control_of_terminal(_pgid: i32) -> std::io::Result<()> { Ok(()) }
#[cfg(not(unix))]
pub fn give_terminal_to(_pgid: i32) -> std::io::Result<()> { Ok(()) }
#[cfg(not(unix))]
pub fn give_terminal_back_to_shell() -> std::io::Result<()> { Ok(()) }
