mod app;
mod appdata;
mod event;
mod kill;
mod numbers;
mod strings;
mod sysinfo;
mod tui;
mod ui;
mod update;

use anyhow::Result;
use app::App;
use tui::Tui;

fn main() -> Result<()> {
    let mut app = App::new();
    app.startup();
    let mut tui = Tui::new();
    tui.enter()?;
    while !app.should_quit {
        tui.draw(&mut app)?;
        tui.handle_events(&mut app)?;
    }
    tui.exit()?;
    Ok(())
}
