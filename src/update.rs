use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;

pub fn update(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Esc => match app.window_phase {
            crate::app::WindowPhase::ProcessPick => {
                app.quit();
            }
            crate::app::WindowPhase::SignalPick => {
                app.window_phase = crate::app::WindowPhase::ProcessPick;
            }
        },
        KeyCode::Char('c') | KeyCode::Char('C') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.quit()
        }
        KeyCode::Down => app.move_cursor(1),
        KeyCode::Up => app.move_cursor(-1),
        KeyCode::Char(c) => {
            app.filter_text.push(c);
            app.filter_processes();
        }
        KeyCode::Backspace => {
            app.filter_text.pop();
            app.filter_processes();
        }
        KeyCode::Enter => match app.window_phase {
            crate::app::WindowPhase::ProcessPick => {
                app.confirm_process();
            }
            crate::app::WindowPhase::SignalPick => {
                app.confirm_signal();
            }
        },
        _ => {}
    };
}
