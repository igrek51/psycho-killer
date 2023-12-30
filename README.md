# 😵‍💫 PSycho KILLer

Interactive process killer, manager and system resources monitor. Does `ps` + `kill`.

<div align="center">
    <a href="https://github.com/igrek51/psycho-killer">GitHub</a>
    -
    <a href="https://crates.io/crates/psycho-killer">Crates</a>
    -
    <a href="https://docs.rs/crate/psycho-killer/">docs.rs</a>
</div>

## Features
- Find and kill process quickly in an interactive way
- Kill process with sudo privileges, if it stays alive
- Monitor system resources usage

## Installation
Install Rust and then:
```sh
cargo install psycho-killer
```
This will install `psycho` binary in Rust's Path.

## Usage
Run `psycho` to start interactive process manager.

Enter name of a process you want to kill.

Select how to kill the process:

- Interrupt with `SIGINT` signal
- Terminate process gracefully with `SIGTERM`
- Kill process with `SIGKILL` signal
- Kill process with `SIGTERM` signal as Superuser
- Kill process with `SIGKILL` signal as Superuser
