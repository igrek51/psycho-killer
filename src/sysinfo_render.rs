use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};

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
        lines.push(Line::from(vec![
            Span::styled("Used: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!(
                    "{} / {} (",
                    self.memory.used.to_kilobytes(),
                    self.memory.total.to_kilobytes()
                ),
                Style::default().fg(Color::LightYellow),
            ),
            Span::styled(self.memory.usage.to_percent1(), get_usage_style(self.memory.usage)),
            Span::styled(")", Style::default().fg(Color::LightYellow)),
        ]));

        self.add_body_line(&mut lines, format!("Cache: {}", self.memory.cache.to_kilobytes()));
        self.add_body_line(&mut lines, format!("Buffers: {}", self.memory.buffers.to_kilobytes()));
        self.add_body_line(
            &mut lines,
            format!("Dirty & Writeback: {}", self.memory.dirty_writeback().to_kilobytes()),
        );

        if self.memory.swap_total > 0 {
            lines.push(Line::from(vec![
                Span::styled("Swap: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!(
                        "{} / {} (",
                        self.memory.swap_used.to_kilobytes(),
                        self.memory.swap_total.to_kilobytes()
                    ),
                    Style::default().fg(Color::LightYellow),
                ),
                Span::styled(
                    self.memory.swap_usage.to_percent1(),
                    get_usage_style(self.memory.swap_usage),
                ),
                Span::styled(")", Style::default().fg(Color::LightYellow)),
            ]));
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
            lines.push(Line::from(vec![
                Span::styled("Usage: ", Style::default().fg(Color::Cyan)),
                Span::styled(usage.to_percent2(), get_usage_style(usage)),
                Span::styled(" / 100%", Style::default().fg(Color::LightYellow)),
            ]));
        }
        lines.push(Line::from(vec![
            Span::styled("1m Load average: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                self.cpu.load_avg.load_1m.to_percent2(),
                get_usage_style(self.cpu.load_avg.load_1m),
            ),
            Span::styled(" / 100%", Style::default().fg(Color::LightYellow)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("5m Load average: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                self.cpu.load_avg.load_5m.to_percent2(),
                get_usage_style(self.cpu.load_avg.load_5m),
            ),
            Span::styled(" / 100%", Style::default().fg(Color::LightYellow)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("15m Load average: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                self.cpu.load_avg.load_15m.to_percent2(),
                get_usage_style(self.cpu.load_avg.load_15m),
            ),
            Span::styled(" / 100%", Style::default().fg(Color::LightYellow)),
        ]));

        if self.disk_space_usages.len() > 0 {
            lines.push(Line::raw(""));
            self.add_header_line(&mut lines, "# Disk space usage");
            for name in self.disk_space_usages.keys().sorted() {
                let usage: PartitionUsage = self.disk_space_usages[name].clone();
                lines.push(Line::from(vec![
                    Span::styled(format!("{}: ", name), Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!("{} / {} (", usage.used.to_bytes(), usage.total.to_bytes()),
                        Style::default().fg(Color::LightYellow),
                    ),
                    Span::styled(usage.usage.to_percent1(), get_usage_style(usage.usage)),
                    Span::styled(")", Style::default().fg(Color::LightYellow)),
                ]));
            }
        }

        if self.has_io_stats() {
            lines.push(Line::raw(""));
            self.add_header_line(&mut lines, "# Disk IO utilization");

            let disk_read_delta = self.disk.time_reading_ms as i32 - previous_stat.disk.time_reading_ms as i32;
            let disk_write_delta = self.disk.time_writing_ms as i32 - previous_stat.disk.time_writing_ms as i32;
            let time_delta = self.time_ms - previous_stat.time_ms;
            if time_delta > 0 {
                let read_usage = disk_read_delta as f64 / time_delta as f64;
                lines.push(Line::from(vec![
                    Span::styled("Reading: ", Style::default().fg(Color::Cyan)),
                    Span::styled(read_usage.to_percent1(), get_usage_style(read_usage)),
                ]));
                let write_usage = disk_write_delta as f64 / time_delta as f64;
                lines.push(Line::from(vec![
                    Span::styled("Writing: ", Style::default().fg(Color::Cyan)),
                    Span::styled(write_usage.to_percent1(), get_usage_style(write_usage)),
                ]));
            }
        }

        if self.network_total_rx + self.network_total_tx > 0 {
            lines.push(Line::raw(""));
            self.add_header_line(&mut lines, "# Network transfer so far");
            let network_tx_delta = self.network_total_tx as i64 - init_stat.network_total_tx as i64;
            let network_rx_delta = self.network_total_rx as i64 - init_stat.network_total_rx as i64;
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
        let header_style: Style = Style::new().bold().light_cyan();
        lines.push(Line::styled(line.into(), header_style));
    }

    fn add_body_line(&self, lines: &mut Vec<Line>, line: String) {
        if let Some(idx) = line.find(':') {
            let (label, value) = line.split_at(idx);
            let value = &value[1..]; // skip ':'
            lines.push(Line::from(vec![
                Span::styled(format!("{}:", label), Style::default().fg(Color::Cyan)),
                Span::styled(value.to_string(), Style::default().fg(Color::LightYellow)),
            ]));
        } else {
            lines.push(Line::styled(line, Style::default().fg(Color::White)));
        }
    }

    fn add_empty_line(&self, lines: &mut Vec<Line>) {
        lines.push(Line::raw(""));
    }
}

fn get_usage_style(usage: f64) -> Style {
    if usage > 0.9 {
        Style::default().fg(Color::Red)
    } else if usage > 0.75 {
        Style::default().fg(Color::LightRed)
    } else if usage > 0.5 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Green)
    }
}
