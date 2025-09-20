# 0-Shell 🐚

A minimalist Unix-like shell written in **Rust**, built from scratch to mimic essential shell behavior without relying on existing binaries like `bash` or `sh`.

This project introduces core concepts in **Unix system programming**: process creation, file system interaction, job control, and scripting — all implemented safely in Rust.

---

## ✨ Features

- **Interactive Shell**
  - Displays a prompt (`$ `) and reads user input.
  - Parses and executes built-in commands.
  - Exits gracefully with `exit` or `Ctrl+D`.

- **Built-in Commands**
  Implemented using Rust system calls, not external binaries:
  - `echo`
  - `cd`
  - `ls` (supports `-l`, `-a`, `-F`)
  - `pwd`
  - `cat`
  - `cp`
  - `rm` (supports `-r`)
  - `mv`
  - `mkdir`
  - `exit`

- **Error Handling**
  - Prints `Command '<name>' not found` for unrecognized commands.
  - Handles invalid input gracefully without crashing.

---

## 📜 Scripting

0-Shell can also run **shell scripts** in addition to interactive use.

- Supports script files (`./0-shell examples/create-dir.sh`).
- Inline control structures (loops, functions):
  ```sh
  for ((i = 0 ; i < 5 ; i++)); do
    echo $i
  done
