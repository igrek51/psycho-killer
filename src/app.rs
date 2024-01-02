use anyhow::Result;
use ratatui::widgets::TableState;
use signal_hook::{consts::SIGINT, consts::SIGTERM, iterator::Signals};
use std::sync::mpsc;
use std::thread;

use crate::appdata::WindowPhase;
use crate::kill::{generate_knwon_signals, kill_pid, KillSignal};
use crate::sysinfo::{
    get_proc_stats, get_system_stats, show_debug_statistics, ProcessStat, SystemProcStats,
    SystemStat, PRINT_SYS_STATS,
};
use crate::tui::Tui;

#[derive(Debug, Default)]
pub struct App {
    pub should_quit: bool,
    pub window_phase: WindowPhase,
    pub process_cursor: usize,
    pub proc_stats: SystemProcStats,
    pub sys_stat: SystemStat,
    pub init_stat: SystemStat,
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

    pub fn run(&mut self) -> Result<()> {
        if PRINT_SYS_STATS {
            show_debug_statistics();
        }
        let signal_rx = self.handle_signals();
        self.refresh_system_stats();
        self.refresh_processes();
        self.init_stat = self.sys_stat.clone();
        let mut tui = Tui::new();
        tui.enter()?;

        while !self.should_quit {
            tui.draw(self)?;
            tui.handle_events(self)?;

            signal_rx.try_recv().ok().map(|_| {
                self.quit();
            });
        }

        tui.exit()?;
        Ok(())
    }

    pub fn handle_signals(&mut self) -> mpsc::Receiver<i32> {
        let (tx, rx) = mpsc::channel();
        let mut signals = Signals::new(&[SIGINT, SIGTERM]).unwrap();
        thread::spawn(move || {
            for sig in signals.forever() {
                println!("Received signal {:?}", sig);
                tx.send(sig).unwrap();
            }
        });
        return rx;
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
        self.sys_stat.summarize(&self.init_stat)
    }
}

pub fn contains_all_words(text: &str, words: &Vec<String>) -> bool {
    let lower_text = text.to_lowercase();
    words.iter().all(|it| lower_text.contains(it))
}
