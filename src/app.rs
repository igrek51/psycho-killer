use anyhow::Result;
use ratatui::widgets::TableState;
use signal_hook::{consts::SIGINT, consts::SIGTERM, iterator::Signals};
use std::sync::mpsc;
use std::thread;
use sysinfo::{System, SystemExt};

use crate::appdata::{Ordering, WindowFocus};
use crate::kill::{generate_knwon_signals, KillSignal};
use crate::logs::print_logs;
use crate::sysinfo::{ProcessStat, SystemProcStats, SystemStat};
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
    pub error_message: Option<String>,
    pub info_message: Option<String>,
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
        print_logs();
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

    pub fn tick(&mut self) {
        self.previous_stat = self.sys_stat.clone();
        self.refresh_system_stats();
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
