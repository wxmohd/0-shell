use super::{env::expand_vars, fileops::* , status::*};
use crate::prelude::*;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub fn is_builtin(name: &str) -> bool {
    matches!(name,
        "exit" | "echo" | "pwd" | "cd" | "mkdir" | "ls" | "cat" | "cp" | "rm" | "mv" |
        "jobs" | "fg" | "bg" | "kill" | "sleep" | "read"
    )
}

/// Dispatches builtins. Returns an exit status.
pub fn dispatch_builtin(shell: &mut crate::shell::Shell, cmd: String, args: &[String]) -> Result<i32> {
    match cmd.as_str() {
        "exit" => {
            // Return special signal so REPL exits immediately
            Ok(EXIT_SIGNAL)
        }
        "echo" => cmd_echo(shell, args),
        "pwd"  => cmd_pwd(),
        "cd"   => cmd_cd(args),
        "mkdir"=> cmd_mkdir(args),
        "ls"   => cmd_ls(args),
        "cat"  => cmd_cat(args),
        "cp"   => cmd_cp(args),
        "rm"   => cmd_rm(args),
        "mv"   => cmd_mv(args),

        // job control
        "jobs" => crate::shell::jobs::control::builtin_jobs(&mut shell.jobs, args),
        "fg"   => crate::shell::jobs::control::builtin_fg(shell, args),
        "bg"   => crate::shell::jobs::control::builtin_bg(&mut shell.jobs, args),
        "kill" => crate::shell::jobs::control::builtin_kill(&mut shell.jobs, args),

        // utilities for scripting demos
        "sleep" => cmd_sleep(args),
        "read"  => cmd_read(shell, args),
        _ => {
            eprintln!("Command '{cmd}' not found");
            Ok(127)
        }
    }
}

fn cmd_echo(shell: &crate::shell::Shell, args: &[String]) -> Result<i32> {
    let expanded = expand_vars(args, &shell.vars);
    println!("{}", expanded.join(" "));
    Ok(ok())
}

fn cmd_pwd() -> Result<i32> {
    let cwd = env::current_dir()?;
    println!("{}", cwd.display());
    Ok(ok())
}

fn cmd_cd(args: &[String]) -> Result<i32> {
    let target = if args.is_empty() {
        env::var("HOME").unwrap_or_else(|_| "/".into())
    } else {
        args[0].clone()
    };
    if let Err(e) = env::set_current_dir(&target) {
        eprintln!("cd: {}: {}", target, e);
        return Ok(err());
    }
    Ok(ok())
}

fn cmd_mkdir(args: &[String]) -> Result<i32> {
    if args.is_empty() {
        eprintln!("mkdir: missing operand");
        return Ok(err());
    }
    for a in args {
        if let Err(e) = fs::create_dir(a) {
            eprintln!("mkdir: {}: {}", a, e);
            return Ok(err());
        }
    }
    Ok(ok())
}

fn parse_ls_flags(args: &[String]) -> (bool, bool, bool, Vec<String>) {
    let mut long = false;
    let mut all = false;
    let mut classify = false;
    let mut rest = Vec::new();
    for a in args {
        if a.starts_with('-') && a.len() > 1 {
            for ch in a.chars().skip(1) {
                match ch {
                    'l' => long = true,
                    'a' => all = true,
                    'F' => classify = true,
                    _ => {}
                }
            }
        } else {
            rest.push(a.clone());
        }
    }
    (long, all, classify, rest)
}

fn cmd_ls(args: &[String]) -> Result<i32> {
    let (long, all, classify, paths) = parse_ls_flags(args);
    let targets = if paths.is_empty() { vec![".".to_string()] } else { paths };

    for (i, t) in targets.iter().enumerate() {
        let path = Path::new(t);
        let meta = match fs::metadata(path) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("ls: {}: {}", t, e);
                continue;
            }
        };

        if targets.len() > 1 {
            if i > 0 { println!(); }
            println!("{}:", t);
        }

        if meta.is_dir() {
            let mut entries: Vec<_> = fs::read_dir(path)?
                .filter_map(|e| e.ok())
                .collect();
            entries.sort_by_key(|e| e.file_name());

            for e in entries {
                let name = e.file_name();
                let name = name.to_string_lossy();
                if !all && is_hidden(&name) { continue; }

                let p = e.path();
                let md = e.metadata()?;
                if long {
                    let mode = super::fileops::mode_string(&md);
                    let size = md.len();
                    print!("{mode} {:>8} ", size);
                }
                if classify {
                    print!("{}{}", name, super::fileops::classify_suffix(&p, md.is_dir()));
                } else {
                    print!("{name}");
                }
                println!();
            }
        } else {
            if long {
                let mode = super::fileops::mode_string(&meta);
                let size = meta.len();
                print!("{mode} {:>8} ", size);
            }
            if classify {
                print!("{}{}", t, super::fileops::classify_suffix(path, meta.is_dir()));
            } else {
                print!("{t}");
            }
            println!();
        }
    }

    Ok(ok())
}

fn cmd_cat(args: &[String]) -> Result<i32> {
    if args.is_empty() {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        print!("{buf}");
        return Ok(ok());
    }
    for a in args {
        let mut f = match fs::File::open(a) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("cat: {}: {}", a, e);
                return Ok(err());
            }
        };
        let mut buf = String::new();
        f.read_to_string(&mut buf)?;
        print!("{buf}");
    }
    Ok(ok())
}

fn cmd_cp(args: &[String]) -> Result<i32> {
    if args.len() != 2 {
        eprintln!("cp: usage: cp <src> <dst>");
        return Ok(err());
    }
    let (src, dst) = (&args[0], &args[1]);

    let meta = match fs::metadata(src) {
        Ok(m) => m,
        Err(e) => { eprintln!("cp: {}: {}", src, e); return Ok(err()); }
    };

    if meta.is_dir() {
        eprintln!("cp: copying directories recursively not implemented");
        return Ok(err());
    } else {
        fs::copy(src, dst).map_err(|e| format!("cp: {} -> {}: {}", src, dst, e))?;
    }
    Ok(ok())
}

fn cmd_rm(args: &[String]) -> Result<i32> {
    if args.is_empty() {
        eprintln!("rm: missing operand");
        return Ok(err());
    }
    let mut recursive = false;
    let mut targets = Vec::new();
    for a in args {
        if a == "-r" {
            recursive = true;
        } else {
            targets.push(a);
        }
    }

    for t in targets {
        let meta = match fs::symlink_metadata(t) {
            Ok(m) => m,
            Err(e) => { eprintln!("rm: {}: {}", t, e); return Ok(err()); }
        };
        if meta.is_dir() {
            if !recursive {
                eprintln!("rm: {}: is a directory (use -r)", t);
                return Ok(err());
            }
            fs::remove_dir_all(t)?;
        } else {
            fs::remove_file(t)?;
        }
    }
    Ok(ok())
}

fn cmd_mv(args: &[String]) -> Result<i32> {
    if args.len() != 2 {
        eprintln!("mv: usage: mv <src> <dst>");
        return Ok(err());
    }
    let (src, dst) = (&args[0], &args[1]);
    match fs::rename(src, dst) {
        Ok(_) => Ok(ok()),
        Err(_) => {
            fs::copy(src, dst).map_err(|e| format!("mv copy: {} -> {}: {}", src, dst, e))?;
            fs::remove_file(src).or_else(|_| fs::remove_dir_all(src)).ok();
            Ok(ok())
        }
    }
}

fn cmd_sleep(args: &[String]) -> Result<i32> {
    let secs: u64 = args.get(0).and_then(|s| s.parse().ok()).unwrap_or(1);
    std::thread::sleep(std::time::Duration::from_secs(secs));
    Ok(ok())
}

fn cmd_read(shell: &mut crate::shell::Shell, args: &[String]) -> Result<i32> {
    // usage: read NAME
    let Some(var) = args.get(0) else {
        eprintln!("read: missing variable name");
        return Ok(err());
    };
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let line = line.strip_suffix('\n').unwrap_or(&line).to_string();
    shell.vars.insert(var.clone(), line);
    Ok(ok())
}
