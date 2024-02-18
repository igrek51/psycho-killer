use ratatui::style::{Style, Stylize};
use ratatui::text::Line;

use crate::numbers::{BytesFormatterExt, PercentFormatterExt};
use crate::sysinfo::{PartitionUsage, SystemStat};

use itertools::Itertools;

impl SystemStat {
    pub fn summarize(&self, init_stat: &SystemStat, previous_stat: &SystemStat) -> Vec<Line> {
        let mut lines: Vec<Line> = Vec::new();

        self.add_body_line(&mut lines, format!("OS: {}", self.os_version));
        self.add_body_line(&mut lines, format!("Host: {}", self.host_name));

        self.add_empty_line(&mut lines);
        self.add_header_line(&mut lines, "# Memory");
        self.add_body_line(
            &mut lines,
            format!(
                "Used: {} / {} ({})",
                self.memory.used.to_kilobytes(),
                self.memory.total.to_kilobytes(),
                self.memory.usage.to_percent1(),
            ),
        );

        self.add_body_line(&mut lines, format!("Cache: {}", self.memory.cache.to_kilobytes()));
        self.add_body_line(&mut lines, format!("Buffers: {}", self.memory.buffers.to_kilobytes()));
        self.add_body_line(
            &mut lines,
            format!("Dirty & Writeback: {}", self.memory.dirty_writeback().to_kilobytes()),
        );

        if self.memory.swap_total > 0 {
            self.add_body_line(
                &mut lines,
                format!(
                    "Swap: {} / {} ({})",
                    self.memory.swap_used.to_kilobytes(),
                    self.memory.swap_total.to_kilobytes(),
                    self.memory.swap_usage.to_percent1(),
                ),
            );
        }

        lines.push(Line::raw(""));
        self.add_header_line(&mut lines, "# CPU");
        if self.cpu_num > 0 {
            self.add_body_line(&mut lines, format!("Cores: {}", self.cpu_num));
        }
        if self.cpu.total_time > 0 {
            let busy_delta = self.cpu.busy_time as i32 - previous_stat.cpu.busy_time as i32;
            let total_delta = self.cpu.total_time as i32 - previous_stat.cpu.total_time as i32;
            let usage: f64 = match total_delta {
                0 => 0f64,
                _ => busy_delta as f64 / total_delta as f64,
            };
            self.add_body_line(&mut lines, format!("Usage: {}", usage.to_percent2()));
            // 0-100%
        }
        self.add_body_line(
            &mut lines,
            format!("1m Load average: {}", self.cpu.load_avg.load_1m.to_percent2()),
        );
        self.add_body_line(
            &mut lines,
            format!("5m Load average: {}", self.cpu.load_avg.load_5m.to_percent2()),
        );
        self.add_body_line(
            &mut lines,
            format!("15m Load average: {}", self.cpu.load_avg.load_15m.to_percent2()),
        );

        if self.disk_space_usages.len() > 0 {
            lines.push(Line::raw(""));
            self.add_header_line(&mut lines, "# Disk space usage");
            for name in self.disk_space_usages.keys().sorted() {
                let usage: PartitionUsage = self.disk_space_usages[name].clone();
                self.add_body_line(
                    &mut lines,
                    format!(
                        "{}: {} / {} ({})",
                        name,
                        usage.used.to_bytes(),
                        usage.total.to_bytes(),
                        usage.usage.to_percent1(),
                    ),
                );
            }
        }

        if self.has_io_stats() {
            lines.push(Line::raw(""));
            self.add_header_line(&mut lines, "# Disk IO utilization");

            let disk_read_delta = self.disk.time_reading_ms as i32 - previous_stat.disk.time_reading_ms as i32;
            let disk_write_delta = self.disk.time_writing_ms as i32 - previous_stat.disk.time_writing_ms as i32;
            let time_delta = self.time_ms - previous_stat.time_ms;
            if time_delta > 0 {
                self.add_body_line(
                    &mut lines,
                    format!(
                        "Reading: {}",
                        (disk_read_delta as f64 / time_delta as f64).to_percent1(),
                    ),
                );
                self.add_body_line(
                    &mut lines,
                    format!(
                        "Writing: {}",
                        (disk_write_delta as f64 / time_delta as f64).to_percent1(),
                    ),
                );
            }
        }

        if self.network_total_rx + self.network_total_tx > 0 {
            lines.push(Line::raw(""));
            self.add_header_line(&mut lines, "# Network transfer so far");
            let network_tx_delta = self.network_total_tx as i32 - init_stat.network_total_tx as i32;
            let network_rx_delta = self.network_total_rx as i32 - init_stat.network_total_rx as i32;
            self.add_body_line(&mut lines, format!("Received: {}", network_rx_delta.to_bytes()));
            self.add_body_line(&mut lines, format!("Transmitted: {}", network_tx_delta.to_bytes()));
        }

        if self.temperatures.len() > 0 {
            lines.push(Line::raw(""));
            self.add_header_line(&mut lines, "# Temperatures");
            for label in self.temperatures.keys().sorted() {
                let temperature = self.temperatures[label];
                self.add_body_line(&mut lines, format!("{}: {:.0}°C", label, temperature.round()));
            }
        }

        lines
    }

    fn add_header_line<T>(&self, lines: &mut Vec<Line>, line: T)
    where
        T: Into<String>,
    {
        let header_style: Style = Style::new().bold().light_yellow();
        lines.push(Line::styled(line.into(), header_style));
    }

    fn add_body_line(&self, lines: &mut Vec<Line>, line: String) {
        let body_style: Style = Style::new().yellow();
        lines.push(Line::styled(line, body_style));
    }

    fn add_empty_line(&self, lines: &mut Vec<Line>) {
        lines.push(Line::raw(""));
    }
}
