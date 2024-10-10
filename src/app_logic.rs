use ratatui::text::Line;
use std::cmp::Ordering::Equal;

use crate::app::App;
use crate::appdata::{Ordering, WindowFocus};
use crate::kill::{kill_pid, KillSignal};
use crate::numbers::ClampNumExt;
use crate::strings::contains_all_words;
use crate::sysinfo::{get_proc_stats, get_system_stats, ProcessStat};

const HELP_INFO: &str = "Keyboard controls:
`?` to show help.
`Ctrl+F` to filter processes.
`F5` or `R` to refresh list.
`S` to sort.
`Enter` to confirm.
`Tab` to switch focus.
`Esc` to cancel.";

impl App {
    pub fn refresh_system_stats(&mut self) {
        self.sys_stat = get_system_stats(&mut self.sysinfo_sys);
    }

    pub fn refresh_processes(&mut self) {
        self.previous_proc_stats = self.proc_stats.clone();
        self.proc_stats = get_proc_stats(&self.sys_stat.memory, &mut self.sysinfo_sys);
        self.filter_processes();
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

    pub fn filter_clear(&mut self) {
        self.filter_text.clear();
        self.filter_processes();
    }

    pub fn filter_backspace(&mut self) {
        self.filter_text.pop();
        self.filter_processes();
    }

    pub fn has_error(&self) -> bool {
        self.error_message.is_some()
    }

    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    pub fn has_info(&self) -> bool {
        self.info_message.is_some()
    }

    pub fn clear_info(&mut self) {
        self.info_message = None;
    }

    pub fn show_help(&mut self) {
        self.info_message = Some(HELP_INFO.to_string());
    }
}
