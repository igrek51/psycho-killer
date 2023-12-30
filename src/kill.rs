use std::process::Command;

#[derive(Debug, Clone)]
pub struct KillSignal {
    pub name: &'static str,
    pub command: &'static str,
}

pub fn generate_knwon_signals() -> Vec<KillSignal> {
    vec![
        KillSignal {
            name: "Interrupt: kill -2",
            command: "kill -2 ",
        },
        KillSignal {
            name: "Terminate (gracefully): kill -15",
            command: "kill -15 ",
        },
        KillSignal {
            name: "Kill process: kill -9",
            command: "kill -9 ",
        },
        KillSignal {
            name: "Superuser Terminate: sudo kill -15",
            command: "sudo kill -15 ",
        },
        KillSignal {
            name: "Superuser Kill: sudo kill -9",
            command: "sudo kill -9 ",
        },
    ]
}

pub fn kill_pid(pid: &String, signal: &KillSignal) {
    let command = signal.command;
    let cmd = format!("{command}{pid}");
    Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("failed to execute command");
}
