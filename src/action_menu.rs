use anyhow::{anyhow, Context, Ok, Result};
use std::process::{Command, Stdio};

use crate::logs::log;

#[derive(Debug, Clone)]
pub struct MenuAction {
    pub name: &'static str,
    pub operation: Operation,
}

#[derive(Debug, Clone)]
pub enum Operation {
    KillSignal { template: &'static str },
    ShowDetails,
}

pub fn generate_known_menu_actions() -> Vec<MenuAction> {
    vec![
        MenuAction {
            name: "Process details",
            operation: Operation::ShowDetails,
        },
        MenuAction {
            name: "Interrupt: kill -2",
            operation: Operation::KillSignal { template: "kill -2 " },
        },
        MenuAction {
            name: "Terminate gracefully: kill -15",
            operation: Operation::KillSignal { template: "kill -15 " },
        },
        MenuAction {
            name: "Kill forcefully: kill -9",
            operation: Operation::KillSignal { template: "kill -9 " },
        },
        MenuAction {
            name: "Superuser Terminate: sudo kill -15",
            operation: Operation::KillSignal {
                template: "sudo kill -15 ",
            },
        },
        MenuAction {
            name: "Superuser Kill: sudo kill -9",
            operation: Operation::KillSignal {
                template: "sudo kill -9 ",
            },
        },
    ]
}

pub fn kill_pid(pid: &String, command_template: &'static str) -> Result<()> {
    let cmd = format!("{command_template}{pid}");
    execute_shell(cmd)
}

pub fn execute_shell(cmd: String) -> Result<()> {
    log(format!("Executing command: {:?}", cmd).as_str());
    let c = Command::new("sh")
        .arg("-c")
        .arg(cmd.clone())
        .stdin(Stdio::inherit())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to start a command")?;
    let output = c.wait_with_output().context("failed to read command output")?;

    if !output.status.success() {
        let error = format!(
            "Failed to execute command: {:?}, {}\n{}\n{}",
            cmd,
            output.status,
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout),
        );
        log(error.as_str());
        return Err(anyhow!(error));
    }
    Ok(())
}
