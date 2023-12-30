#[derive(Debug, PartialEq, Eq)]
pub enum WindowPhase {
    ProcessPick,
    SignalPick,
}

impl Default for WindowPhase {
    fn default() -> Self {
        WindowPhase::ProcessPick
    }
}
