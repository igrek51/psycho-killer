use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, KeyEventKind};

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
/// Events are polled on demand by the thread that owns the UI. Nothing reads the
/// terminal in the background, so keys pressed while the app is busy stay in the
/// terminal input queue instead of being consumed and thrown away.
#[derive(Debug)]
pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }

    /// Wait up to `timeout` for the next event. None means timeout.
    pub fn next(&self, timeout: Duration) -> Result<Option<Event>> {
        self.poll(timeout)
    }

    /// Take an already-available event without waiting.
    pub fn try_next(&self) -> Result<Option<Event>> {
        self.poll(Duration::ZERO)
    }

    fn poll(&self, timeout: Duration) -> Result<Option<Event>> {
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
