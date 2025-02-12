# 😵‍💫 PSycho KILLer

PSycho KILLer is an interactive process manager and system resource monitor that combines the functionalities of `ps` and `kill`.

<div align="center">
    <a href="https://github.com/igrek51/psycho-killer">GitHub</a>
    -
    <a href="https://crates.io/crates/psycho-killer">Crates</a>
    -
    <a href="https://docs.rs/crate/psycho-killer/">docs.rs</a>
</div>

![](./docs/img/screenshot1.png)

## Features
- "Seek & Destroy" - Quickly find and terminate processes in an interactive manner.
- If a process still remains alive, kill it with sudo privileges and stronger signals.
- Monitor usage of system resources:
  - Memory usage, including `Dirty` and `Writeback` memory to keep an eye on ongoing copying
  - CPU usage
  - Disk space usage
  - Disk IO utliziation
  - Network transfer
  - Temperatures
- Group processes by executable path and show aggregated stats, e.g. total memory usage of a multiprocess application

## Installation
### Cargo
```sh
cargo install psycho-killer
```
This will install latest version of `psycho` binary in Rust's Path.

### Binary
Alternatively, you can download the compiled binary:

```sh
curl -L https://github.com/igrek51/psycho-killer/releases/download/0.6.4/psycho -o ~/bin/psycho
chmod +x ~/bin/psycho
```

## Usage
Launch the interactive process manager by running `psycho`.

Enter the phrase of a process you want to kill.

Choose the preferred method to terminate the process:

- Interrupt with `SIGINT` signal
- Gracefully terminate the process with `SIGTERM`
- Forcefully kill the process with `SIGKILL` signal
- Terminate the process with `SIGTERM` signal as Superuser
- Forcefully kill the process with `SIGKILL` signal as Superuser

## Keyboard controls
- `?` to show help.
- `Ctrl+F` or `F` to filter processes.
- Arrows `↑` and `↓` to navigate list.
- `F5` or `R` to refresh list.
- `S` to sort.
- `M` to order by memory usage.
- `C` to order by CPU usage.
- `U` to order by uptime.
- `G` group processes by executable path.
- `Enter` to select or confirm.
- `Tab` to switch tab.
- `Esc` to cancel or quit.
