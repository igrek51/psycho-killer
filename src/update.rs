use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{app::App, appdata::WindowFocus::*};

pub fn update(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Esc => match app.window_focus {
            ProcessFilter => {
                app.window_focus = Browse;
            }
            SignalPick => {
                app.window_focus = ProcessFilter;
            }
            _ => {
                app.quit();
            }
        },
        KeyCode::Char('c') | KeyCode::Char('C') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.quit()
        }
        KeyCode::Down => app.move_cursor(1),
        KeyCode::Up => app.move_cursor(-1),
        KeyCode::Left => {
            app.move_horizontal_scroll(-10);
        }
        KeyCode::Right => {
            app.move_horizontal_scroll(10);
        }
        KeyCode::PageDown => app.move_cursor(10),
        KeyCode::PageUp => app.move_cursor(-10),
        KeyCode::Home => app.move_cursor(-(app.filtered_processes.len() as i32)),
        KeyCode::End => app.move_cursor(app.filtered_processes.len() as i32),
        KeyCode::Char('u')
            if key_event.modifiers == KeyModifiers::CONTROL
                && app.window_focus == ProcessFilter =>
        {
            app.filter_text.clear();
            app.filter_processes();
        }
        KeyCode::Tab => {
            app.window_focus = match app.window_focus {
                Browse => SystemStats,
                SystemStats => ProcessFilter,
                ProcessFilter => Browse,
                _ => app.window_focus,
            };
        }
        KeyCode::BackTab => {
            app.window_focus = match app.window_focus {
                SystemStats => Browse,
                ProcessFilter => SystemStats,
                Browse => ProcessFilter,
                _ => app.window_focus,
            };
        }
        KeyCode::Backspace if app.window_focus == ProcessFilter => {
            app.filter_text.pop();
            app.filter_processes();
        }
        KeyCode::Char('w')
            if key_event.modifiers == KeyModifiers::CONTROL
                && app.window_focus == ProcessFilter =>
        {
            app.filter_text.pop();
            app.filter_processes();
        }
        KeyCode::Char(c) if app.window_focus == ProcessFilter => {
            app.filter_text.push(c);
            app.filter_processes();
        }
        KeyCode::Char('/') | KeyCode::F(4) if app.window_focus == Browse => {
            app.window_focus = ProcessFilter;
        }
        KeyCode::Char('q') if app.window_focus == Browse || app.window_focus == SystemStats => {
            app.quit();
        }
        KeyCode::F(5) => {
            app.refresh_processes();
        }
        KeyCode::Char('r') if app.window_focus == Browse => {
            app.refresh_processes();
        }
        KeyCode::F(6) => {
            app.switch_ordering();
        }
        KeyCode::Char('s') if app.window_focus == Browse => {
            app.switch_ordering();
        }
        KeyCode::Char('j') if app.window_focus == SignalPick => {
            app.move_cursor(1);
        }
        KeyCode::Char('k') if app.window_focus == SignalPick => {
            app.move_cursor(-1);
        }
        KeyCode::Enter => match app.window_focus {
            Browse | ProcessFilter => {
                app.confirm_process();
            }
            SignalPick => {
                app.confirm_signal();
            }
            _ => {}
        },
        KeyCode::Char(c) if app.window_focus == Browse => {
            app.window_focus = ProcessFilter;
            app.filter_text.push(c);
            app.filter_processes();
        }
        _ => {}
    };
}
