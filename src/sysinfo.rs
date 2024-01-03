use std::time::SystemTime;
use std::{collections::HashMap, ops::Deref};

use anyhow::Context;
use itertools::Itertools;
use sysinfo::{ComponentExt, DiskExt, NetworkExt, ProcessExt, System, SystemExt, Uid};

use crate::numbers::{BytesFormatterExt, PercentFormatterExt};

pub const PRINT_SYS_STATS: bool = true;

#[derive(Debug, Default)]
pub struct SystemProcStats {
    pub processes: Vec<ProcessStat>,
}

#[derive(Debug, Default, Clone)]
pub struct ProcessStat {
    pub pid: String,
    pub pid_num: u32,
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
    pub run_time: u64, // in seconds
}

impl ProcessStat {
    pub fn search_name(&self) -> String {
        format!("{} {}", self.pid, self.display_name)
    }
}

#[derive(Debug, Default, Clone)]
pub struct SystemStat {
    time_ms: u64,

    pub os_version: String,
    pub host_name: String,

    pub cpu_num: usize,

    pub memory: MemoryStat,
    pub disk: DiskStat,
    pub cpu: CpuStat,

    pub disk_space_usages: HashMap<String, PartitionUsage>,

    pub network_total_tx: u64, // total number of bytes transmitted
    pub network_total_rx: u64,

    pub temperatures: HashMap<String, f32>,
}

impl SystemStat {
    pub fn has_io_stats(&self) -> bool {
        self.disk.time_reading_ms > 0 || self.disk.time_writing_ms > 0
    }
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

#[derive(Debug, Default, Clone)]
pub struct DiskStat {
    pub time_reading_ms: u64,
    pub time_writing_ms: u64,
}

#[derive(Debug, Default, Clone)]
pub struct PartitionUsage {
    pub mount_point: String,
    pub used: u64,
    pub total: u64,
    pub usage: f64,
}

#[derive(Debug, Default, Clone)]
pub struct CpuStat {
    pub busy_time: u64,
    pub total_time: u64,
}

pub fn get_proc_stats(memstat: &MemoryStat, sys: &mut System) -> SystemProcStats {
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
            pid_num: pid.to_string().parse().unwrap_or(0),
            name: proc_name,
            cmd,
            exe: process.exe().to_string_lossy().to_string(),
            cpu_usage: process.cpu_usage() as f64 / 100f64,
            memory_usage: mem_usage_fraction,
            disk_usage,
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
    let disk: DiskStat = read_disk_stats();
    let cpu: CpuStat = read_cpu_stats();

    let mut network_total_tx: u64 = 0;
    let mut network_total_rx: u64 = 0;
    for (interface_name, data) in sys.networks() {
        if !is_net_iface_physical(interface_name) {
            continue;
        }
        network_total_tx += data.total_transmitted();
        network_total_rx += data.total_received();
    }

    let mut disk_space_usages: HashMap<String, PartitionUsage> = HashMap::new();
    for disk in sys.disks() {
        let mount_point = disk.mount_point().to_str().unwrap_or("");
        if include_mount_point(mount_point) && disk.total_space() > 0 {
            let used = disk.total_space() - disk.available_space();
            let partition_usage = PartitionUsage {
                mount_point: mount_point.to_string(),
                total: disk.total_space(),
                used,
                usage: used as f64 / disk.total_space() as f64,
            };
            disk_space_usages.insert(mount_point.to_string(), partition_usage);
        }
    }

    let mut temperatures: HashMap<String, f32> = HashMap::new();
    for component in sys.components() {
        let temperature = component.temperature();
        if temperature.is_normal() {
            temperatures.insert(component.label().to_string(), temperature);
        }
    }

    SystemStat {
        time_ms: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
        os_version,
        host_name,
        cpu_num,
        memory,
        network_total_tx,
        network_total_rx,
        disk_space_usages,
        disk,
        cpu,
        temperatures,
    }
}

fn is_net_iface_physical(name: &str) -> bool {
    name.starts_with("enp")
        || name.starts_with("eth")
        || name.starts_with("wlp")
        || name.starts_with("wlan")
}

fn include_mount_point(mount_point: &str) -> bool {
    mount_point == "/"
        || mount_point.starts_with("/mnt/")
        || mount_point.starts_with("/media/")
        || mount_point.starts_with("/storage/")
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

fn read_disk_stats() -> DiskStat {
    let lines: Vec<String> = std::fs::read_to_string("/proc/diskstats")
        .unwrap_or(String::new())
        .split('\n')
        .map(|x| x.to_string()) // avoid dropping temporary var
        .collect();

    let mut time_reading_ms: u64 = 0;
    let mut time_writing_ms: u64 = 0;

    for line in lines {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 11 {
            continue;
        }
        time_reading_ms += parts[6].parse().unwrap_or(0);
        time_writing_ms += parts[10].parse().unwrap_or(0);
    }

    return DiskStat {
        time_reading_ms,
        time_writing_ms,
    };
}

fn read_cpu_stats() -> CpuStat {
    let lines: Vec<String> = std::fs::read_to_string("/proc/stat")
        .unwrap_or(String::new())
        .split('\n')
        .map(|x| x.to_string())
        .collect();
    if lines.len() < 1 {
        return CpuStat::default();
    }
    let first_line = lines[0].trim();
    assert!(
        first_line.starts_with("cpu "),
        "unexpected first line of /proc/stat: {}",
        first_line
    );
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 11 {
        return CpuStat::default();
    }

    let user = parts[1].parse::<u64>().unwrap_or(0);
    let nice = parts[2].parse::<u64>().unwrap_or(0);
    let system = parts[3].parse::<u64>().unwrap_or(0);
    let idle = parts[4].parse::<u64>().unwrap_or(0);
    let iowait = parts[5].parse::<u64>().unwrap_or(0);
    let irq = parts[6].parse::<u64>().unwrap_or(0);
    let softirq = parts[7].parse::<u64>().unwrap_or(0);
    let steal = parts[8].parse::<u64>().unwrap_or(0);
    let guest = parts[9].parse::<u64>().unwrap_or(0);
    let guest_nice = parts[10].parse::<u64>().unwrap_or(0);

    let total_time =
        user + nice + system + idle + iowait + irq + softirq + steal + guest + guest_nice;
    let busy_time = user + nice + system + irq + softirq + steal + guest + guest_nice;
    return CpuStat {
        busy_time,
        total_time,
    };
}

pub fn show_debug_statistics() {
    let mut sys = System::new_all();
    sys.refresh_all();

    // Components temperature:
    println!("=> components:");
    for component in sys.components() {
        println!("{:?}", component);
    }
}

impl SystemStat {
    pub fn summarize(&self, init_stat: &SystemStat, previous_stat: &SystemStat) -> String {
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

        if self.cpu_num > 0 {
            lines.push(String::new());
            lines.push(String::from("# CPU"));
            lines.push(format!("Cores: {}", self.cpu_num));

            let busy_delta = self.cpu.busy_time as i32 - previous_stat.cpu.busy_time as i32;
            let total_delta = self.cpu.total_time as i32 - previous_stat.cpu.total_time as i32;
            let usage: f64 = match total_delta {
                0 => 0f64,
                _ => busy_delta as f64 / total_delta as f64,
            };
            lines.push(format!("Usage: {}", usage.to_percent2()));
        }

        if self.disk_space_usages.len() > 0 {
            lines.push(String::new());
            lines.push(String::from("# Disk space usage"));
            for name in self.disk_space_usages.keys().sorted() {
                let usage: PartitionUsage = self.disk_space_usages[name].clone();
                lines.push(format!(
                    "{}: {} / {} ({})",
                    name,
                    usage.used.to_bytes(),
                    usage.total.to_bytes(),
                    usage.usage.to_percent1(),
                ));
            }
        }

        if self.has_io_stats() {
            lines.push(String::new());
            lines.push(String::from("# Disk IO utilization"));

            let disk_read_delta =
                self.disk.time_reading_ms as i32 - previous_stat.disk.time_reading_ms as i32;
            let disk_write_delta =
                self.disk.time_writing_ms as i32 - previous_stat.disk.time_writing_ms as i32;
            let time_delta = self.time_ms - previous_stat.time_ms;
            if time_delta > 0 {
                lines.push(format!(
                    "Reading: {}",
                    (disk_read_delta as f64 / time_delta as f64).to_percent1(),
                ));
                lines.push(format!(
                    "Writing: {}",
                    (disk_write_delta as f64 / time_delta as f64).to_percent1(),
                ));
            }
        }

        if self.network_total_rx + self.network_total_tx > 0 {
            lines.push(String::new());
            lines.push(String::from("# Network transfer so far"));
            let network_tx_delta = self.network_total_tx as i32 - init_stat.network_total_tx as i32;
            let network_rx_delta = self.network_total_rx as i32 - init_stat.network_total_rx as i32;
            lines.push(format!("Received: {}", network_rx_delta.to_bytes()));
            lines.push(format!("Transmitted: {}", network_tx_delta.to_bytes()));
        }

        if self.temperatures.len() > 0 {
            lines.push(String::new());
            lines.push(String::from("# Temperatures"));
            for label in self.temperatures.keys().sorted() {
                let temperature = self.temperatures[label];
                lines.push(format!("{}: {:.0}°C", label, temperature.round()));
            }
        }

        lines.join("\n")
    }
}
