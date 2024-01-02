#[derive(Debug, PartialEq, Eq)]
pub enum WindowPhase {
    Browse,
    ProcessFilter,
    SignalPick,
}

impl Default for WindowPhase {
    fn default() -> Self {
        WindowPhase::Browse
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Ordering {
    ByUptime,
    ByMemory,
    ByCpu,
}

impl Default for Ordering {
    fn default() -> Self {
        Ordering::ByUptime
    }
}
