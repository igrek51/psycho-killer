use crate::appdata::WindowPhase;
use crate::kill::{generate_knwon_signals, kill_pid, KillSignal};
use crate::sysinfo::{get_system_stats, show_statistics, ProcessStat, SystemStats};

#[derive(Debug, Default)]
pub struct App {
    pub should_quit: bool,
    pub window_phase: WindowPhase,
    pub process_cursor: usize,
    pub system_stats: SystemStats,
    pub filter_text: String,
    pub filtered_processes: Vec<ProcessStat>,
    pub signal_cursor: usize,
    pub known_signals: Vec<KillSignal>,
}

const PRINT_SYS_STATS: bool = false;

impl App {
    pub fn new() -> Self {
        Self {
            known_signals: generate_knwon_signals(),
            ..Self::default()
        }
    }

    pub fn startup(&mut self) {
        if PRINT_SYS_STATS {
            show_statistics();
        }
        self.system_stats = get_system_stats();
        self.filter_processes();
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn move_cursor(&mut self, delta: i32) {
        match self.window_phase {
            WindowPhase::Browse | WindowPhase::ProcessFilter => {
                let mut new_index = self.process_cursor as i32 + delta;
                new_index = new_index.clamp(0, (self.filtered_processes.len() as i32 - 1).max(0));
                self.process_cursor = new_index as usize;
            }
            WindowPhase::SignalPick => {
                let mut new_index = self.signal_cursor as i32 + delta;
                new_index = new_index.clamp(0, (self.known_signals.len() as i32 - 1).max(0));
                self.signal_cursor = new_index as usize;
            }
        }
    }

    pub fn filter_processes(&mut self) {
        let filter_text = self.filter_text.to_lowercase();
        self.filtered_processes = self
            .system_stats
            .processes
            .iter()
            .filter(|it: &&ProcessStat| it.display_name.to_lowercase().contains(&filter_text))
            .cloned()
            .collect();
        self.filtered_processes
            .sort_by(|a, b| a.run_time.cmp(&b.run_time));
        self.move_cursor(0);
    }

    pub fn confirm_process(&mut self) {
        if self.process_cursor >= self.filtered_processes.len() {
            return;
        }
        self.window_phase = WindowPhase::SignalPick;
        self.signal_cursor = 0;
    }

    pub fn confirm_signal(&mut self) {
        let process: &ProcessStat = &self.filtered_processes[self.process_cursor];
        let signal: &KillSignal = &self.known_signals[self.signal_cursor];
        kill_pid(&process.pid, signal);

        self.window_phase = WindowPhase::ProcessFilter;
        self.system_stats = get_system_stats();
        self.filter_processes();
    }
}
