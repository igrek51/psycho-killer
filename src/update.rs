use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    app::App,
    appdata::{Ordering, WindowFocus::*},
    logs::log,
};

pub fn update(app: &mut App, key_event: KeyEvent) {
    let mut consumed = true;
    match key_event.code {
        KeyCode::Enter | KeyCode::Esc => {
            if app.has_error() {
                app.clear_error();
            } else if app.has_info() {
                app.clear_info();
            } else {
                consumed = false;
            }
        }
        KeyCode::Char('c') | KeyCode::Char('C') if is_ctrl(key_event) => app.quit(),
        KeyCode::Tab => {
            app.window_focus = match app.window_focus {
                Browse => SystemStats,
                SystemStats => ProcessFilter,
                ProcessFilter => Browse,
                _ => app.window_focus,
            }
        }
        KeyCode::BackTab => {
            app.window_focus = match app.window_focus {
                SystemStats => Browse,
                ProcessFilter => SystemStats,
                Browse => ProcessFilter,
                _ => app.window_focus,
            }
        }
        _ => consumed = false,
    };
    if !consumed {
        match app.window_focus {
            Browse => on_key_browse(app, key_event),
            ProcessFilter => on_key_process_filter(app, key_event),
            SignalPick => on_key_signal_pick(app, key_event),
            SystemStats => on_key_system_stats(app, key_event),
        };
    }
}

pub fn on_key_browse(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Esc | KeyCode::Char('q') => app.quit(),
        KeyCode::Down => app.move_cursor(1),
        KeyCode::Up => app.move_cursor(-1),
        KeyCode::Left => app.move_horizontal_scroll(-10),
        KeyCode::Right => app.move_horizontal_scroll(10),
        KeyCode::PageDown => app.move_cursor(10),
        KeyCode::PageUp => app.move_cursor(-10),
        KeyCode::Home => app.move_cursor(-(app.filtered_processes.len() as i32)),
        KeyCode::End => app.move_cursor(app.filtered_processes.len() as i32),
        KeyCode::Char('f') if is_ctrl(key_event) => app.window_focus = ProcessFilter,
        KeyCode::Char('f') => app.window_focus = ProcessFilter,
        KeyCode::Char('/') | KeyCode::F(4) => app.window_focus = ProcessFilter,
        KeyCode::Char('?') => app.show_help(),
        KeyCode::F(5) => app.refresh_processes(),
        KeyCode::Char('r') => app.refresh_processes(),
        KeyCode::F(6) => app.switch_ordering(),
        KeyCode::Char('s') | KeyCode::Char('o') => app.switch_ordering(),
        KeyCode::Char('m') => app.set_process_ordering(Ordering::ByMemory),
        KeyCode::Char('c') => app.set_process_ordering(Ordering::ByCpu),
        KeyCode::Char('u') => app.set_process_ordering(Ordering::ByUptime),
        KeyCode::Char('g') => app.toggle_group_by_exe(),
        KeyCode::Enter => app.confirm_process(),
        _ => log(format!("Unknown key event: {:?}", key_event).as_str()),
    };
}

pub fn on_key_process_filter(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Esc => app.window_focus = Browse,
        KeyCode::Down => app.move_cursor(1),
        KeyCode::Up => app.move_cursor(-1),
        KeyCode::PageDown => app.move_cursor(10),
        KeyCode::PageUp => app.move_cursor(-10),
        KeyCode::Home => app.move_cursor(-(app.filtered_processes.len() as i32)),
        KeyCode::End => app.move_cursor(app.filtered_processes.len() as i32),
        KeyCode::Backspace => app.filter_backspace(),
        KeyCode::Char('u') if is_ctrl(key_event) => app.filter_clear(),
        KeyCode::Char('w') if is_ctrl(key_event) => app.filter_backspace(),
        KeyCode::Char(c) => app.filter_append(c),
        KeyCode::F(5) => app.refresh_processes(),
        KeyCode::Enter => app.confirm_process(),
        _ => log(format!("Unknown key event: {:?}", key_event).as_str()),
    };
}

pub fn on_key_signal_pick(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Esc => app.window_focus = ProcessFilter,
        KeyCode::Down | KeyCode::Char('j') => app.move_cursor(1),
        KeyCode::Up | KeyCode::Char('k') => app.move_cursor(-1),
        KeyCode::PageDown => app.move_cursor(10),
        KeyCode::PageUp => app.move_cursor(-10),
        KeyCode::Home => app.move_cursor(-(app.known_menu_actions.len() as i32)),
        KeyCode::End => app.move_cursor(app.known_menu_actions.len() as i32),
        KeyCode::Enter => app.confirm_signal(),
        _ => log(format!("Unknown key event: {:?}", key_event).as_str()),
    };
}

pub fn on_key_system_stats(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Esc | KeyCode::Char('q') => app.quit(),
        KeyCode::Down => app.move_cursor(1),
        KeyCode::Up => app.move_cursor(-1),
        KeyCode::PageDown => app.move_cursor(10),
        KeyCode::PageUp => app.move_cursor(-10),
        KeyCode::F(5) => app.refresh_processes(),
        KeyCode::Char('?') => app.show_help(),
        _ => log(format!("Unknown key event: {:?}", key_event).as_str()),
    };
}

fn is_ctrl(key_event: KeyEvent) -> bool {
    key_event.modifiers == KeyModifiers::CONTROL
}
