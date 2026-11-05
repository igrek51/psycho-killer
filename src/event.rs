use std::time::Duration;

use anyhow::Result;
use crossterm::event::KeyEvent;

#[cfg(unix)]
use crate::tty_input::TtyInput;

/// Terminal events.
#[derive(Clone, Copy, Debug)]
pub enum Event {
    /// Key press.
    Key(KeyEvent),
    /// Terminal resize.
    Resize,
}

/// Terminal event handler.
///
/// On Unix keys are read one at a time straight from the tty, so a key pressed
/// right after the quit key stays in the terminal input queue for the shell.
/// Other platforms fall back to crossterm's reader.
#[derive(Debug)]
pub struct EventHandler {
    #[cfg(unix)]
    input: TtyInput,
}

impl EventHandler {
    pub fn new() -> Result<Self> {
        Ok(Self {
            #[cfg(unix)]
            input: TtyInput::new()?,
        })
    }

    /// Wait up to `timeout` for the next event. None means timeout.
    pub fn next(&self, timeout: Duration) -> Result<Option<Event>> {
        self.poll(timeout)
    }

    /// Take an already-available event without waiting.
    pub fn try_next(&self) -> Result<Option<Event>> {
        self.poll(Duration::ZERO)
    }

    #[cfg(unix)]
    fn poll(&self, timeout: Duration) -> Result<Option<Event>> {
        self.input.next_event(timeout)
    }

    #[cfg(not(unix))]
    fn poll(&self, timeout: Duration) -> Result<Option<Event>> {
        use crossterm::event::{self, Event as CrosstermEvent, KeyEventKind};

        if !event::poll(timeout)? {
            return Ok(None);
        }
        match event::read()? {
            CrosstermEvent::Key(k) if k.kind == KeyEventKind::Press => Ok(Some(Event::Key(k))),
            CrosstermEvent::Key(_) => Ok(None),
            CrosstermEvent::Resize(_, _) => Ok(Some(Event::Resize)),
            _ => Ok(None),
        }
    }
}
