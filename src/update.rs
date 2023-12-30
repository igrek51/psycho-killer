use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{app::App, appdata::WindowPhase::*};

pub fn update(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Esc => match app.window_phase {
            Browse => {
                app.quit();
            }
            ProcessFilter => {
                app.window_phase = Browse;
            }
            SignalPick => {
                app.window_phase = ProcessFilter;
            }
        },
        KeyCode::Char('c') | KeyCode::Char('C') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.quit()
        }
        KeyCode::Down => app.move_cursor(1),
        KeyCode::Up => app.move_cursor(-1),
        KeyCode::Char(c) if app.window_phase == ProcessFilter => {
            app.filter_text.push(c);
            app.filter_processes();
        }
        KeyCode::Backspace if app.window_phase == ProcessFilter => {
            app.filter_text.pop();
            app.filter_processes();
        }
        KeyCode::Char('/') if app.window_phase == Browse => {
            app.window_phase = ProcessFilter;
        }
        KeyCode::Char('q') if app.window_phase == Browse => {
            app.quit();
        }
        KeyCode::Enter => match app.window_phase {
            Browse | ProcessFilter => {
                app.confirm_process();
            }
            SignalPick => {
                app.confirm_signal();
            }
        },
        _ => {}
    };
}
