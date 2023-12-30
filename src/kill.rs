use std::process::Command;

#[derive(Debug, Clone)]
pub struct KillSignal {
    pub name: &'static str,
    pub command: &'static str,
}

pub fn generate_signals() -> Vec<KillSignal> {
    vec![
        KillSignal {
            name: "SIGINT (2) - interrupt",
            command: "kill -2 ",
        },
        KillSignal {
            name: "SIGTERM (15) - graceful shutdown",
            command: "kill -15 ",
        },
        KillSignal {
            name: "SIGKILL (9) - kill",
            command: "kill -9 ",
        },
        KillSignal {
            name: "sudo SIGKILL (9)",
            command: "sudo kill -9 ",
        },
    ]
}

pub fn kill_pid(pid: &String, signal: &KillSignal) {
    let cmd = format!("{}{}", signal.command, pid);
    Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("failed to execute command");
}
