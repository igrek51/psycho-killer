#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum WindowFocus {
    Browse,
    ProcessFilter,
    SignalPick,
    SystemStats,
}

impl Default for WindowFocus {
    fn default() -> Self {
        WindowFocus::Browse
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
