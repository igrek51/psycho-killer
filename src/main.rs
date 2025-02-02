mod action_menu;
mod app;
mod app_logic;
mod appdata;
mod event;
mod keyboard;
mod logs;
mod numbers;
mod strings;
mod sysinfo;
mod sysinfo_render;
mod tui;
mod ui;

use anyhow::{Context, Result};

use crate::app::App;

fn main() -> Result<()> {
    let mut app = App::new();
    app.run().context("app failed")?;
    Ok(())
}
