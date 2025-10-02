use crate::prelude::*;
use std::io::{Read, Write};

#[cfg(unix)]
use nix::sys::termios::{tcgetattr, tcsetattr, LocalFlags, SetArg, Termios};
#[cfg(unix)]
use std::os::fd::AsRawFd;

/// Read one line with in-memory history and Up/Down navigation (Unix).
/// Returns Ok(Some(line)) on Enter, Ok(None) on Ctrl+D at empty line.
pub fn read_line_with_history(prompt: &str, history: &mut Vec<String>) -> io::Result<Option<String>> {
    #[cfg(unix)]
    {
        let stdin = std::io::stdin();
        let fd = stdin.as_raw_fd();

        let mut orig = tcgetattr(fd).map_err(to_io)?;
        let mut raw = orig.clone();
        // raw-ish: line-by-line, no echo, keep signals (^C)
        raw.local_flags.remove(LocalFlags::ICANON | LocalFlags::ECHO);
        tcsetattr(fd, SetArg::TCSANOW, &raw).map_err(to_io)?;

        struct Restore(i32, Termios);
        impl Drop for Restore {
            fn drop(&mut self) { let _ = tcsetattr(self.0, SetArg::TCSANOW, &self.1); }
        }
        let _restore = Restore(fd, orig.clone());

        print!("{prompt}");
        std::io::stdout().flush()?;

        let mut buf = String::new();
        let mut hist_index: Option<usize> = None;
        let mut b = [0u8; 1];

        loop {
            let n = stdin.lock().read(&mut b)?;
            if n == 0 {
                if buf.is_empty() { print!("\r\n"); std::io::stdout().flush()?; return Ok(None); }
                continue;
            }

            match b[0] {
                b'\r' | b'\n' => {
                    print!("\r\n"); std::io::stdout().flush()?;
                    if !buf.is_empty() { history.push(buf.clone()); }
                    return Ok(Some(buf));
                }
                0x04 => { // Ctrl+D
                    if buf.is_empty() { print!("\r\n"); std::io::stdout().flush()?; return Ok(None); }
                }
                0x03 => { // Ctrl+C
                    buf.clear(); hist_index = None;
                    print!("\r\n{prompt}"); std::io::stdout().flush()?;
                }
                0x7f | 0x08 => { // Backspace
                    if !buf.is_empty() {
                        buf.pop();
                        print!("\r\x1b[K{prompt}{buf}");
                        std::io::stdout().flush()?;
                    }
                }
                0x1b => {
                    // ESC [ A/B  => arrows; read two more bytes
                    let mut seq = [0u8; 2];
                    if stdin.lock().read(&mut seq).unwrap_or(0) == 2 && seq[0] == b'[' {
                        match seq[1] {
                            b'A' => { // Up
                                if history.is_empty() { continue; }
                                let i = match hist_index { None => history.len().saturating_sub(1), Some(0) => 0, Some(i) => i.saturating_sub(1) };
                                hist_index = Some(i);
                                buf = history[i].clone();
                                print!("\r\x1b[K{prompt}{buf}");
                                std::io::stdout().flush()?;
                            }
                            b'B' => { // Down
                                if history.is_empty() { continue; }
                                if let Some(i) = hist_index {
                                    if i + 1 < history.len() {
                                        hist_index = Some(i + 1);
                                        buf = history[i + 1].clone();
                                    } else {
                                        hist_index = None;
                                        buf.clear();
                                    }
                                    print!("\r\x1b[K{prompt}{buf}");
                                    std::io::stdout().flush()?;
                                }
                            }
                            _ => { /* ignore left/right for now */ }
                        }
                    }
                }
                c => {
                    if c.is_ascii() && !c.is_ascii_control() {
                        buf.push(c as char);
                        print!("{}", c as char);
                        std::io::stdout().flush()?;
                    }
                }
            }
        }
    }

    // Fallback (non-Unix): plain read_line (no arrows)
    #[cfg(not(unix))]
    {
        print!("{prompt}");
        std::io::stdout().flush()?;
        let mut line = String::new();
        let n = std::io::stdin().read_line(&mut line)?;
        if n == 0 { print!("\n"); return Ok(None); }
        let line = line.trim_end_matches(['\n','\r']).to_string();
        if !line.is_empty() { history.push(line.clone()); }
        Ok(Some(line))
    }
}

#[cfg(unix)]
fn to_io(err: nix::Error) -> std::io::Error {
    match err.as_errno() {
        Some(e) => std::io::Error::from_raw_os_error(e as i32),
        None => std::io::Error::new(std::io::ErrorKind::Other, err.to_string()),
    }
}
