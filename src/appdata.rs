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
