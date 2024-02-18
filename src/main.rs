mod app;
mod appdata;
mod event;
mod kill;
mod numbers;
mod strings;
mod sysinfo;
mod sysinfo_render;
mod tui;
mod ui;
mod update;

use anyhow::Result;

use crate::app::App;

fn main() -> Result<()> {
    let mut app = App::new();
    app.run()?;
    Ok(())
}
