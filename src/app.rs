use ratatui::widgets::TableState;

use crate::appdata::WindowPhase;
use crate::kill::{generate_knwon_signals, kill_pid, KillSignal};
use crate::numbers::{BytesFormatterExt, PercentFormatterExt};
use crate::sysinfo::{
    get_proc_stats, get_system_stats, show_statistics, ProcessStat, SystemProcStats, SystemStat,
    PRINT_SYS_STATS,
};

#[derive(Debug, Default)]
pub struct App {
    pub should_quit: bool,
    pub window_phase: WindowPhase,
    pub process_cursor: usize,
    pub proc_stats: SystemProcStats,
    pub sys_stat: SystemStat,
    pub filter_text: String,
    pub filtered_processes: Vec<ProcessStat>,
    pub signal_cursor: usize,
    pub known_signals: Vec<KillSignal>,
    pub proc_list_table_state: TableState,
    pub horizontal_scroll: i32,
}

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
        self.refresh_system_stats();
        self.refresh_processes();
    }

    pub fn refresh_system_stats(&mut self) {
        self.sys_stat = get_system_stats();
    }

    pub fn refresh_processes(&mut self) {
        self.proc_stats = get_proc_stats(&self.sys_stat.memory);
        self.filter_processes();
    }

    pub fn tick(&mut self) {
        self.refresh_system_stats();
    }

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
        self.proc_list_table_state.select(Some(self.process_cursor));
    }

    pub fn move_horizontal_scroll(&mut self, delta: i32) {
        let mut new_value = self.horizontal_scroll + delta;
        new_value = new_value.max(0);
        self.horizontal_scroll = new_value;
    }

    pub fn filter_processes(&mut self) {
        let filter_words: Vec<String> = self
            .filter_text
            .split_whitespace()
            .map(|it| it.to_lowercase())
            .collect();
        self.filtered_processes = self
            .proc_stats
            .processes
            .iter()
            .filter(|it: &&ProcessStat| contains_all_words(it.display_name.as_str(), &filter_words))
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
        self.refresh_processes();
    }

    pub fn format_sys_stats(&self) -> String {
        format!(
            "
OS: {}
Host: {}

Memory usage: {} / {} ({})
Cache: {}
Buffers: {}
Dirty: {}
Writeback: {}
Swap: {} / {} ({})

CPU cores: {}
",
            self.sys_stat.os_version,
            self.sys_stat.host_name,
            self.sys_stat.memory.used.format_kib(),
            self.sys_stat.memory.total.format_kib(),
            self.sys_stat.memory.usage.format_percent(),
            self.sys_stat.memory.cache.format_kib(),
            self.sys_stat.memory.buffers.format_kib(),
            self.sys_stat.memory.dirty.format_kib(),
            self.sys_stat.memory.writeback.format_kib(),
            self.sys_stat.memory.swap_used.format_kib(),
            self.sys_stat.memory.swap_total.format_kib(),
            self.sys_stat.memory.swap_usage.format_percent(),
            self.sys_stat.cpu_num,
        )
        .trim()
        .to_string()
    }
}

pub fn contains_all_words(text: &str, words: &Vec<String>) -> bool {
    let lower_text = text.to_lowercase();
    words.iter().all(|it| lower_text.contains(it))
}
