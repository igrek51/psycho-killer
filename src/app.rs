use anyhow::Result;
use ratatui::text::Line;
use ratatui::widgets::TableState;
use signal_hook::{consts::SIGINT, consts::SIGTERM, iterator::Signals};
use std::cmp::Ordering::Equal;
use std::sync::mpsc;
use std::thread;
use sysinfo::{System, SystemExt};

use crate::appdata::{Ordering, WindowFocus};
use crate::kill::{generate_knwon_signals, kill_pid, KillSignal};
use crate::numbers::ClampNumExt;
use crate::sysinfo::{get_proc_stats, get_system_stats, ProcessStat, SystemProcStats, SystemStat};
use crate::tui::Tui;

#[derive(Debug, Default)]
pub struct App {
    pub should_quit: bool,
    pub window_focus: WindowFocus,
    pub process_cursor: usize,
    pub proc_stats: SystemProcStats,
    pub previous_proc_stats: SystemProcStats,
    pub sys_stat: SystemStat,
    pub previous_stat: SystemStat,
    pub init_stat: SystemStat,
    pub filter_text: String,
    pub filtered_processes: Vec<ProcessStat>,
    pub signal_cursor: usize,
    pub known_signals: Vec<KillSignal>,
    pub proc_list_table_state: TableState,
    pub horizontal_scroll: i32,
    pub sysinfo_scroll: i32,
    pub sysinfo_sys: System,
    pub ordering: Ordering,
}

impl App {
    pub fn new() -> Self {
        Self {
            known_signals: generate_knwon_signals(),
            sysinfo_sys: System::new_all(),
            ..Self::default()
        }
    }

    pub fn run(&mut self) -> Result<()> {
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
        self.previous_proc_stats = self.proc_stats.clone();
        self.proc_stats = get_proc_stats(&self.sys_stat.memory, &mut self.sysinfo_sys);
        self.filter_processes();
    }

    pub fn tick(&mut self) {
        self.previous_stat = self.sys_stat.clone();
        self.refresh_system_stats();
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn move_cursor(&mut self, delta: i32) {
        match self.window_focus {
            WindowFocus::Browse | WindowFocus::ProcessFilter => {
                self.process_cursor = (self.process_cursor as i32 + delta)
                    .clamp_max(self.filtered_processes.len() as i32 - 1)
                    .clamp_min(0) as usize;
                self.proc_list_table_state.select(Some(self.process_cursor));
            }
            WindowFocus::SignalPick => {
                self.signal_cursor = (self.signal_cursor as i32 + delta)
                    .clamp_max(self.known_signals.len() as i32 - 1)
                    .clamp_min(0) as usize;
            }
            WindowFocus::SystemStats => {
                self.sysinfo_scroll = (self.sysinfo_scroll + delta).clamp_min(0);
            }
        }
    }

    pub fn move_horizontal_scroll(&mut self, delta: i32) {
        let mut new_value = self.horizontal_scroll + delta;
        new_value = new_value.max(0);
        self.horizontal_scroll = new_value;
    }

    pub fn switch_ordering(&mut self) {
        self.ordering = match self.ordering {
            Ordering::ByUptime => Ordering::ByMemory,
            Ordering::ByMemory => Ordering::ByCpu,
            Ordering::ByCpu => Ordering::ByUptime,
        };
        self.filter_processes();
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
            .filter(|it: &&ProcessStat| contains_all_words(it.search_name().as_str(), &filter_words))
            .cloned()
            .collect();
        let sort_fn = self.get_sort_fn();
        self.filtered_processes.sort_unstable_by(sort_fn);

        self.move_cursor(0);
    }

    pub fn get_sort_fn(&self) -> fn(&ProcessStat, &ProcessStat) -> std::cmp::Ordering {
        match self.ordering {
            Ordering::ByUptime => |x, y| {
                let run_time_cmp = x.run_time.cmp(&y.run_time);
                if run_time_cmp != Equal {
                    return run_time_cmp;
                }
                return x.pid_num.cmp(&y.pid_num).reverse();
            },
            Ordering::ByMemory => |x, y| {
                let memory_usage_cmp = x.memory_usage.partial_cmp(&y.memory_usage).unwrap_or(Equal);
                if memory_usage_cmp != Equal {
                    return memory_usage_cmp.reverse();
                }
                return x.pid_num.cmp(&y.pid_num).reverse();
            },
            Ordering::ByCpu => |x, y| {
                let cpu_usage_cmp = x.cpu_usage.partial_cmp(&y.cpu_usage).unwrap_or(Equal);
                if cpu_usage_cmp != Equal {
                    return cpu_usage_cmp.reverse();
                }
                return x.pid_num.cmp(&y.pid_num).reverse();
            },
        }
    }

    pub fn confirm_process(&mut self) {
        if self.process_cursor >= self.filtered_processes.len() {
            return;
        }
        self.window_focus = WindowFocus::SignalPick;
        self.signal_cursor = 0;
    }

    pub fn confirm_signal(&mut self) {
        let process: &ProcessStat = &self.filtered_processes[self.process_cursor];
        let signal: &KillSignal = &self.known_signals[self.signal_cursor];
        kill_pid(&process.pid, signal);

        self.window_focus = WindowFocus::ProcessFilter;
        self.refresh_processes();
    }

    pub fn format_sys_stats(&self) -> Vec<Line> {
        self.sys_stat
            .summarize(&self.init_stat, &self.previous_stat)
            .iter()
            .skip(self.sysinfo_scroll as usize)
            .cloned()
            .collect()
    }
}

pub fn contains_all_words(text: &str, words: &Vec<String>) -> bool {
    let lower_text = text.to_lowercase();
    words.iter().all(|it| lower_text.contains(it))
}
