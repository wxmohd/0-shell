#[cfg(unix)]
use nix::sys::signal::{SigHandler, SigSet, Signal};
#[cfg(unix)]
use once_cell::sync::Lazy;
#[cfg(unix)]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(unix)]
pub static CHLD_FLAG: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

#[cfg(unix)]
extern "C" fn on_sigchld(_sig: i32) {
    CHLD_FLAG.store(true, Ordering::SeqCst);
}

#[cfg(unix)]
pub fn install_handlers() {
    unsafe {
        let _ = nix::sys::signal::signal(Signal::SIGCHLD, SigHandler::Handler(on_sigchld));
        // Ignore SIGINT in the shell; foreground jobs get the signal
        let _ = nix::sys::signal::signal(Signal::SIGINT, SigHandler::SigIgn);
        // Shell shouldn't stop on Ctrl+Z; the foreground job should
        let _ = nix::sys::signal::signal(Signal::SIGTSTP, SigHandler::SigIgn);
    }
}

#[cfg(not(unix))]
pub fn install_handlers() {}

pub mod tty;
