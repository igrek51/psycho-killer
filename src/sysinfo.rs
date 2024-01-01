use std::ops::Deref;

use anyhow::Context;
use sysinfo::{NetworkExt, ProcessExt, System, SystemExt, Uid};

use crate::numbers::{BytesFormatterExt, PercentFormatterExt};

pub const PRINT_SYS_STATS: bool = false;

#[derive(Debug, Default)]
pub struct SystemProcStats {
    pub processes: Vec<ProcessStat>,
}

#[derive(Debug, Default, Clone)]
pub struct ProcessStat {
    pub pid: String,
    // Short: chrome
    pub name: String,
    // Full command: /opt/google/chrome/chrome --type=renderer ...
    pub cmd: String,
    // Full executable path: /opt/google/chrome/chrome
    pub exe: String,
    pub cpu_usage: f64,    // fraction of 1 core
    pub memory_usage: f64, // fraction of total memory
    pub disk_usage: f64,
    pub user_id: Option<u32>,
    pub display_name: String,
    pub run_time: u64,
}

#[derive(Debug, Default, Clone)]
pub struct SystemStat {
    pub os_version: String,
    pub host_name: String,

    pub cpu_num: usize,
    pub cpu_usage: f64,

    pub memory: MemoryStat,

    pub disk_space_usage: f64,
    pub disk_io_usage: f64,

    pub network_total_tx: u64, // total number of bytes transmitted
    pub network_total_rx: u64,
}

#[derive(Debug, Default, Clone)]
pub struct MemoryStat {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub cache: u64,
    pub buffers: u64,
    pub dirty: u64,
    pub writeback: u64,
    pub usage: f64,

    pub swap_total: u64,
    pub swap_used: u64,
    pub swap_usage: f64,
}

impl MemoryStat {
    pub fn dirty_writeback(&self) -> u64 {
        self.dirty + self.writeback
    }
}

pub fn get_proc_stats(memstat: &MemoryStat) -> SystemProcStats {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut processes = Vec::new();
    for (pid, process) in sys.processes() {
        let user_id: Option<u32> = process.user_id().map(|uid: &Uid| *uid.deref());
        let cmd = process.cmd().join(" ");
        let proc_name = process.name().to_string();
        let display_name: String = match cmd.is_empty() {
            false => cmd.clone(),
            _ => proc_name.clone(),
        };
        let mem_usage_fraction: f64 = process.memory() as f64 / 1024f64 / memstat.total as f64;
        let disk_usage = process.disk_usage().total_written_bytes as f64
            + process.disk_usage().total_read_bytes as f64;

        let process_stat = ProcessStat {
            pid: pid.to_string(),
            name: proc_name,
            cmd,
            exe: process.exe().to_string_lossy().to_string(),
            cpu_usage: process.cpu_usage() as f64 / 100f64,
            memory_usage: mem_usage_fraction,
            disk_usage: disk_usage,
            user_id,
            display_name,
            run_time: process.run_time(),
        };
        processes.push(process_stat);
    }

    SystemProcStats { processes }
}

pub fn get_system_stats() -> SystemStat {
    let mut sys = System::new_all();
    sys.refresh_all();

    let os_version = sys.long_os_version().unwrap_or(String::new());
    let host_name = sys.host_name().unwrap_or(String::new());
    let cpu_num = sys.cpus().len();

    let memory: MemoryStat = read_memory_stats();

    let mut network_total_tx: u64 = 0;
    let mut network_total_rx: u64 = 0;
    for (interface_name, data) in sys.networks() {
        if !is_net_iface_physical(interface_name) {
            continue;
        }
        network_total_tx += data.total_transmitted();
        network_total_rx += data.total_received();
    }

    SystemStat {
        os_version,
        host_name,
        cpu_num,
        memory,
        network_total_tx,
        network_total_rx,
        ..SystemStat::default()
    }
}

fn is_net_iface_physical(name: &str) -> bool {
    name.starts_with("enp") || name.starts_with("eth") || name.starts_with("wlp")
}

pub fn read_memory_stats() -> MemoryStat {
    let meminfo_lines: Vec<String> = std::fs::read_to_string("/proc/meminfo")
        .unwrap_or(String::new())
        .split('\n')
        .map(|x| x.to_string()) // avoid dropping temporary var
        .collect();

    let mut memory_total: u64 = 0;
    let mut memory_free: u64 = 0;
    let mut memory_available: u64 = 0;
    let mut memory_cache: u64 = 0;
    let mut memory_buffers: u64 = 0;
    let mut memory_dirty: u64 = 0;
    let mut memory_writeback: u64 = 0;
    let mut swap_total: u64 = 0;
    let mut swap_free: u64 = 0;

    for line in meminfo_lines {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 3 {
            continue;
        }
        let key: &str = parts[0];
        let value_kb: u64 = parts[1]
            .parse()
            .context("failed to parse meminfo value as u64")
            .unwrap();
        match key {
            "MemTotal:" => {
                memory_total = value_kb;
            }
            "MemFree:" => {
                memory_free = value_kb;
            }
            "MemAvailable:" => {
                memory_available = value_kb;
            }
            "Buffers:" => {
                memory_buffers = value_kb;
            }
            "Cached:" => {
                memory_cache = value_kb;
            }
            "Dirty:" => {
                memory_dirty = value_kb;
            }
            "Writeback:" => {
                memory_writeback = value_kb;
            }
            "SwapTotal:" => {
                swap_total = value_kb;
            }
            "SwapFree:" => {
                swap_free = value_kb;
            }
            _ => {}
        }
    }

    let memory_used = memory_total - memory_available;
    let swap_used = swap_total - swap_free;
    MemoryStat {
        total: memory_total,
        used: memory_used,
        free: memory_free,
        cache: memory_cache,
        buffers: memory_buffers,
        dirty: memory_dirty,
        writeback: memory_writeback,
        usage: memory_used as f64 / memory_total as f64,
        swap_total,
        swap_used,
        swap_usage: swap_used as f64 / swap_total as f64,
    }
}

pub fn show_statistics() {
    let mut sys = System::new_all();
    sys.refresh_all();

    println!("=> disks:");
    for disk in sys.disks() {
        println!("{:?}", disk);
    }

    println!("=> networks:");
    for (interface_name, data) in sys.networks() {
        println!(
            "{}: {}/{} B",
            interface_name,
            data.total_received(),
            data.total_transmitted()
        );
    }

    // Components temperature:
    println!("=> components:");
    for component in sys.components() {
        println!("{:?}", component);
    }

    println!("=> system:");
    println!("total memory: {} bytes", sys.total_memory());
    println!("used memory : {} bytes", sys.used_memory());
    println!("total swap  : {} bytes", sys.total_swap());
    println!("used swap   : {} bytes", sys.used_swap());

    println!("System name:             {:?}", sys.name());
    println!("System kernel version:   {:?}", sys.kernel_version());
    println!("System OS version:       {:?}", sys.os_version());
    println!("System host name:        {:?}", sys.host_name());

    println!("Number of CPUs: {}", sys.cpus().len());
}

impl SystemStat {
    pub fn summarize(&self, init_stat: &SystemStat) -> String {
        let mut lines = Vec::new();
        lines.push(format!("OS: {}", self.os_version));
        lines.push(format!("Host: {}", self.host_name));

        lines.push(String::new());
        lines.push(String::from("# Memory"));
        lines.push(format!(
            "Used: {} / {} ({})",
            self.memory.used.to_kilobytes(),
            self.memory.total.to_kilobytes(),
            self.memory.usage.to_percent1(),
        ));
        lines.push(format!("Cache: {}", self.memory.cache.to_kilobytes()));
        lines.push(format!("Buffers: {}", self.memory.buffers.to_kilobytes()));
        lines.push(format!(
            "Dirty & Writeback: {}",
            self.memory.dirty_writeback().to_kilobytes()
        ));

        if self.memory.swap_total > 0 {
            lines.push(format!(
                "Swap: {} / {} ({})",
                self.memory.swap_used.to_kilobytes(),
                self.memory.swap_total.to_kilobytes(),
                self.memory.swap_usage.to_percent1(),
            ));
        }

        lines.push(String::new());
        lines.push(String::from("# CPU"));
        lines.push(format!("Cores: {}", self.cpu_num));

        let network_tx_delta = self.network_total_tx as i32 - init_stat.network_total_tx as i32;
        let network_rx_delta = self.network_total_rx as i32 - init_stat.network_total_rx as i32;
        lines.push(String::new());
        lines.push(String::from("# Network transfer so far"));
        lines.push(format!("Received: {}", network_rx_delta.to_bytes()));
        lines.push(format!("Transmitted: {}", network_tx_delta.to_bytes()));

        lines.join("\n")
    }
}
