use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{app::App, appdata::WindowFocus::*, logs::log};

pub fn update(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Enter | KeyCode::Esc if app.has_error() => {
            app.clear_error();
            return;
        }
        KeyCode::Enter | KeyCode::Esc if app.has_info() => {
            app.clear_info();
            return;
        }
        _ => {}
    };

    match key_event.code {
        KeyCode::Esc => match app.window_focus {
            ProcessFilter => app.window_focus = Browse,
            SignalPick => app.window_focus = ProcessFilter,
            _ => app.quit(),
        },
        KeyCode::Char('c') | KeyCode::Char('C') if is_ctrl(key_event) => app.quit(),
        KeyCode::Down => app.move_cursor(1),
        KeyCode::Up => app.move_cursor(-1),
        KeyCode::Left => app.move_horizontal_scroll(-10),
        KeyCode::Right => app.move_horizontal_scroll(10),
        KeyCode::PageDown => app.move_cursor(10),
        KeyCode::PageUp => app.move_cursor(-10),
        KeyCode::Home => app.move_cursor(-(app.filtered_processes.len() as i32)),
        KeyCode::End => app.move_cursor(app.filtered_processes.len() as i32),
        KeyCode::Char('u') if is_ctrl(key_event) && app.window_focus == ProcessFilter => app.filter_clear(),
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
        KeyCode::Backspace if app.window_focus == ProcessFilter => app.filter_backspace(),
        KeyCode::Char('w') if is_ctrl(key_event) && app.window_focus == ProcessFilter => app.filter_backspace(),
        KeyCode::Char('f') if is_ctrl(key_event) && app.window_focus == Browse => {
            app.window_focus = ProcessFilter;
        }
        KeyCode::Char('/') | KeyCode::F(4) if app.window_focus == Browse => {
            app.window_focus = ProcessFilter;
        }
        KeyCode::Char(c) if app.window_focus == ProcessFilter => {
            app.filter_text.push(c);
            app.filter_processes();
        }
        KeyCode::Char('q') if app.window_focus == Browse || app.window_focus == SystemStats => {
            app.quit();
        }
        KeyCode::Char('?') if app.window_focus == Browse => {
            app.show_help();
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
        _ => log(format!("Unknown key event: {:?}", key_event).as_str()),
    };
}

fn is_ctrl(key_event: KeyEvent) -> bool {
    key_event.modifiers == KeyModifiers::CONTROL
}
