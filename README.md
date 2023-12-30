# 😵‍💫 PSycho KILLer

PSycho KILLer is an interactive process manager and system resource monitor that combines the functionalities of `ps` and `kill`.

<div align="center">
    <a href="https://github.com/igrek51/psycho-killer">GitHub</a>
    -
    <a href="https://crates.io/crates/psycho-killer">Crates</a>
    -
    <a href="https://docs.rs/crate/psycho-killer/">docs.rs</a>
</div>

## Features
- Quickly find and terminate processes in an interactive manner.
- If a process remains active, kill it with sudo privileges.
- Monitor system resource usage, including `Dirty` and `Writeback` memory to keep an eye on ongoing copying.

## Installation
```sh
cargo install psycho-killer
```
This will install `psycho` binary in Rust's Path.

## Usage
Launch the interactive process manager by running `psycho`.

Enter the phrase of a process you want to kill.

Choose the preferred method to terminate the process:

- Interrupt with `SIGINT` signal
- Gracefully terminate the process with `SIGTERM`
- Forcefully kill the process with `SIGKILL` signal
- Terminate the process with `SIGTERM` signal as Superuser
- Forcefully kill the process with `SIGKILL` signal as Superuser
