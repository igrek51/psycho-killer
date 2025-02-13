use std::time::SystemTime;
use std::{collections::HashMap, ops::Deref};

use anyhow::{anyhow, Context, Result};
use libc::{sysconf, _SC_CLK_TCK};
use sysinfo::{ComponentExt, DiskExt, NetworkExt, Process, ProcessExt, System, SystemExt, Uid};

use crate::logs::log;
use crate::numbers::PercentFormatterExt;
use crate::numbers::{format_duration, ClampNumExt};
use crate::strings::first_cmd_part;

#[derive(Debug, Default, Clone)]
pub struct SystemProcStats {
    pub processes: Vec<ProcessStat>,
}

#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub struct ProcessStat {
    pub pid: String,
    pub pid_num: u32,
    pub name: String, // Short name, e.g. chrome
    pub cmd: String,  // Full command, e.g. /opt/google/chrome/chrome --type=renderer ...
    pub exe: String,  // Full executable path, e.g. /opt/google/chrome/chrome
    pub cwd: String,
    pub status: String,
    pub cpu_usage: f64,    // fraction of 1 core, [0-CORES]
    pub memory_usage: f64, // fraction of total memory
    pub disk_usage: f64,   // Total written + read bytes
    pub user_id: Option<u32>,
    pub display_name: String,
    pub run_time: u64, // uptime in seconds
    pub time_ms: u64,  // timestamp of reading statistics
    pub cpu_time: f64, // in seconds
    pub parent_pid: Option<String>,
    pub group_children: Vec<ProcessStat>,
}

impl ProcessStat {
    pub fn search_name(&self) -> String {
        format!("{} {}", self.pid, self.display_name)
    }

    pub fn calculate_cpu_usage(&self, previous_processes: &Vec<ProcessStat>) -> f64 {
        let previous_proc: Option<&ProcessStat> = previous_processes.iter().find(|p| p.pid == self.pid);
        if previous_proc.is_none() {
            return self.cpu_usage;
        }
        let delta_cpu_ms = (self.cpu_time - previous_proc.unwrap().cpu_time) * 1000f64;
        let delta_time_ms = self.time_ms - previous_proc.unwrap().time_ms;
        if delta_time_ms == 0 {
            return 0f64;
        }
        delta_cpu_ms / delta_time_ms as f64
    }

    pub fn format_cpu_usage(&self) -> String {
        if self.cpu_usage == 0f64 {
            return "0%".to_string();
        }
        self.cpu_usage.to_percent_len5()
    }

    fn full_command(&self) -> String {
        match self.cmd.is_empty() {
            false => self.cmd.clone(),
            _ => self.name.clone(),
        }
    }

    pub fn details(&self, sys_stat: &SystemStat) -> String {
        if !self.group_children.is_empty() {
            return self.group_details(sys_stat);
        }

        let uptime = format_duration(self.run_time);
        let mem_usage = self.memory_usage.to_percent1();
        let cpu_usage = self.format_cpu_usage();
        let user_id_str = self.user_id.map(|uid| uid.to_string()).unwrap_or("unknown".to_string());
        let max_cpu_usage = format!("{}%", sys_stat.cpu_num * 100);
        let parent_pid_str = self.parent_pid.clone().unwrap_or("-".to_string());
        format!(
            "Process ID: {}
Parent Process ID: {}
User ID: {}
Uptime: {}
Status: {}
Memory usage: {}
CPU usage: {} / {}

Full command: {}

Executable path: {}

Working directory: {}
",
            self.pid,
            parent_pid_str,
            user_id_str,
            uptime,
            self.status,
            mem_usage,
            cpu_usage,
            max_cpu_usage,
            self.full_command(),
            self.exe,
            self.cwd,
        )
    }

    pub fn group_details(&self, sys_stat: &SystemStat) -> String {
        let uptime = format_duration(self.run_time);
        let mem_usage = self.memory_usage.to_percent1();
        let cpu_usage = self.format_cpu_usage();
        let user_id_str = self.user_id.map(|uid| uid.to_string()).unwrap_or("unknown".to_string());
        let max_cpu_usage = format!("{}%", sys_stat.cpu_num * 100);
        let pids = self
            .group_children
            .iter()
            .map(|p| p.pid.clone())
            .collect::<Vec<String>>()
            .join(", ");
        let parent_pids = self
            .group_children
            .iter()
            .map(|p| p.parent_pid.clone().unwrap_or("-".to_string()))
            .collect::<Vec<String>>()
            .join(", ");
        let commands = self
            .group_children
            .iter()
            .map(|p| p.full_command())
            .collect::<Vec<String>>()
            .join("\n");
        let statuses = self
            .group_children
            .iter()
            .map(|p| p.status.clone())
            .collect::<Vec<String>>()
            .join(", ");
        format!(
            "Processes in the group: {}
Process IDs: {}
Parent Process IDs: {}
User ID: {}
Uptime: {}
Status: {}
Memory usage: {}
CPU usage: {} / {}

Executable path: {}

Full commands:
{}
",
            self.group_children.len(),
            pids,
            parent_pids,
            user_id_str,
            uptime,
            statuses,
            mem_usage,
            cpu_usage,
            max_cpu_usage,
            self.exe,
            commands,
        )
    }
}

#[derive(Debug, Default, Clone)]
pub struct SystemStat {
    pub time_ms: u64,
    pub os_version: String,
    pub host_name: String,
    pub cpu_num: usize,
    pub memory: SystemMemoryStat,
    pub disk: DiskIOStat,
    pub cpu: SystemCpuStat,
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

#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub struct SystemMemoryStat {
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

impl SystemMemoryStat {
    pub fn dirty_writeback(&self) -> u64 {
        self.dirty + self.writeback
    }
}

#[derive(Debug, Default, Clone)]
pub struct DiskIOStat {
    pub time_reading_ms: u64,
    pub time_writing_ms: u64,
}

#[derive(Debug, Default, Clone)]
pub struct PartitionUsage {
    pub used: u64,
    pub total: u64,
    pub usage: f64,
}

#[derive(Debug, Default, Clone)]
pub struct SystemCpuStat {
    pub busy_time: u64,
    pub total_time: u64,
    pub load_avg: CpuLoadAvg,
}

#[derive(Debug, Default, Clone)]
pub struct CpuLoadAvg {
    pub load_1m: f64,  // 0-100%
    pub load_5m: f64,  // 0-100%
    pub load_15m: f64, // 0-100%
}

pub fn get_proc_stats(memstat: &SystemMemoryStat, sys: &mut System) -> SystemProcStats {
    sys.refresh_processes();

    let clk_tck: i64 = get_clock_ticks();
    let mut processes = Vec::new();
    let process_map = sys.processes();
    let timestamp_ms = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    for (pid, process) in process_map {
        let user_id: Option<u32> = process.user_id().map(|uid: &Uid| *uid.deref());
        let cmd = process.cmd().join(" ");
        let proc_name = process.name().to_string();
        let display_name: String = match cmd.is_empty() {
            false => cmd.clone(),
            _ => proc_name.clone(),
        };
        let exe_path: String = extract_exe_path(process);
        let cwd: String = process.cwd().to_string_lossy().to_string();
        let mem_usage_fraction: f64 = process.memory() as f64 / 1024f64 / memstat.total as f64;
        let disk_usage = process.disk_usage().total_written_bytes as f64 + process.disk_usage().total_read_bytes as f64;
        let cpu_time = read_process_cpu_time(pid.to_string()).unwrap_or(0) as f64 / clk_tck as f64;
        let cpu_usage = process.cpu_usage() as f64 / 100f64;
        let parent_pid = process.parent().map(|p| p.to_string());
        let status = process.status().to_string();

        let process_stat = ProcessStat {
            pid: pid.to_string(),
            pid_num: pid.to_string().parse().unwrap_or(0),
            name: proc_name,
            cmd,
            exe: exe_path,
            cwd,
            status,
            cpu_usage,
            memory_usage: mem_usage_fraction,
            disk_usage,
            user_id,
            display_name,
            run_time: process.run_time(),
            time_ms: timestamp_ms,
            cpu_time,
            parent_pid,
            group_children: Vec::new(),
        };
        processes.push(process_stat);
    }

    SystemProcStats { processes }
}

pub fn get_system_stats(sys: &mut System) -> SystemStat {
    sys.refresh_system();
    sys.refresh_disks();
    sys.refresh_networks();

    let os_version = sys.long_os_version().unwrap_or(String::new());
    let host_name = sys.host_name().unwrap_or(String::new());
    let cpu_num = sys.cpus().len();

    let memory: SystemMemoryStat = read_memory_stats();
    let disk: DiskIOStat = read_disk_io_stats();
    let cpu: SystemCpuStat = read_system_cpu_stats(cpu_num).unwrap_or_else(|_| SystemCpuStat::default());

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
        if is_mount_point_eligible(mount_point) && disk.total_space() > 0 {
            let used = disk.total_space() - disk.available_space();
            let partition_usage = PartitionUsage {
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
    name.starts_with("enp") || name.starts_with("eth") || name.starts_with("wlp") || name.starts_with("wlan")
}

fn is_mount_point_eligible(mount_point: &str) -> bool {
    mount_point == "/"
        || mount_point.starts_with("/mnt/")
        || mount_point.starts_with("/media/")
        || mount_point.starts_with("/storage/")
}

pub fn read_memory_stats() -> SystemMemoryStat {
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
            "MemTotal:" => memory_total = value_kb,
            "MemFree:" => memory_free = value_kb,
            "MemAvailable:" => memory_available = value_kb,
            "Buffers:" => memory_buffers = value_kb,
            "Cached:" => memory_cache = value_kb,
            "Dirty:" => memory_dirty = value_kb,
            "Writeback:" => memory_writeback = value_kb,
            "SwapTotal:" => swap_total = value_kb,
            "SwapFree:" => swap_free = value_kb,
            _ => {}
        }
    }

    let memory_used = memory_total - memory_available;
    let swap_used = swap_total - swap_free;
    SystemMemoryStat {
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

fn read_disk_io_stats() -> DiskIOStat {
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
    return DiskIOStat {
        time_reading_ms,
        time_writing_ms,
    };
}

fn read_system_cpu_stats(cpu_num: usize) -> Result<SystemCpuStat> {
    let lines: Vec<String> = std::fs::read_to_string("/proc/stat")
        .context("reading /proc/stat")?
        .split('\n')
        .map(|x| x.to_string())
        .collect();
    if lines.len() < 1 {
        return Err(anyhow!("not enough lines"));
    }
    let first_line = lines[0].trim();
    assert!(
        first_line.starts_with("cpu"),
        "unexpected first line of /proc/stat: {}",
        first_line
    );
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 11 {
        return Err(anyhow!("first line doesn't have enough parts"));
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

    let total_time = user + nice + system + idle + iowait + irq + softirq + steal + guest + guest_nice;
    let busy_time = user + nice + system + irq + softirq + steal + guest + guest_nice;
    let load_avg = read_cpu_load_avg(cpu_num).unwrap_or_else(|_| CpuLoadAvg::default());
    Ok(SystemCpuStat {
        busy_time,
        total_time,
        load_avg,
    })
}

fn read_process_cpu_time(pid: String) -> Result<u64> {
    let line: String = std::fs::read_to_string(format!("/proc/{}/stat", pid)).context("reading /proc/PID/stat")?;
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 14 {
        return Err(anyhow!("not enough parts"));
    }
    let utime = parts[13].parse::<u64>().unwrap_or(0);
    let stime = parts[14].parse::<u64>().unwrap_or(0);
    Ok(utime + stime)
}

fn read_cpu_load_avg(cpu_num: usize) -> Result<CpuLoadAvg> {
    let line: String = std::fs::read_to_string("/proc/loadavg").context("reading /proc/loadavg")?;
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 3 {
        return Err(anyhow!("not enough parts"));
    }
    let cpu_num = (cpu_num as f64).clamp_min(1.into());
    let load_1m = parts[0].parse::<f64>().unwrap_or(0.into()) / cpu_num;
    let load_5m = parts[1].parse::<f64>().unwrap_or(0.into()) / cpu_num;
    let load_15m = parts[2].parse::<f64>().unwrap_or(0.into()) / cpu_num;
    Ok(CpuLoadAvg {
        load_1m,
        load_5m,
        load_15m,
    })
}

fn get_clock_ticks() -> i64 {
    let clk_tck: i64 = unsafe { sysconf(_SC_CLK_TCK) }; // clock ticks per second
    if clk_tck == -1 {
        log("Error: getting clock ticks per second");
        return 100;
    }
    clk_tck
}

pub fn group_by_exe_path(processes: &Vec<ProcessStat>) -> Vec<ProcessStat> {
    let mut process_groups: HashMap<String, Vec<ProcessStat>> = HashMap::new();
    for process_stat in processes {
        process_groups
            .entry(process_stat.exe.clone())
            .or_insert(Vec::new())
            .push(process_stat.clone());
    }
    process_groups
        .into_iter()
        .map(|(_, processes)| merge_processes_group(processes))
        .collect()
}

pub fn merge_processes_group(processes: Vec<ProcessStat>) -> ProcessStat {
    let first = processes.get(0).unwrap();
    let cpu_usage: f64 = processes.iter().map(|p| p.cpu_usage).sum();
    let memory_usage: f64 = processes.iter().map(|p| p.memory_usage).sum();
    let disk_usage: f64 = processes.iter().map(|p| p.disk_usage).sum();
    let cpu_time: f64 = processes.iter().map(|p| p.cpu_time).sum();
    let run_time: u64 = processes.iter().map(|p| p.run_time).max().unwrap_or(0);
    ProcessStat {
        pid: first.pid.clone(),
        pid_num: first.pid_num,
        name: first.name.clone(),
        cmd: first.cmd.clone(),
        exe: first.exe.clone(),
        cwd: first.cwd.clone(),
        status: first.status.clone(),
        user_id: first.user_id,
        display_name: first.exe.clone(),
        time_ms: first.time_ms,
        cpu_usage,
        memory_usage,
        disk_usage,
        run_time,
        cpu_time,
        parent_pid: first.parent_pid.clone(),
        group_children: processes,
    }
}

pub fn extract_exe_path(process: &Process) -> String {
    let mut exe = process.exe().to_string_lossy().to_string();
    if exe.ends_with(" (deleted)") {
        exe = exe.trim_end_matches(" (deleted)").to_string();
    }
    if !exe.is_empty() {
        return first_cmd_part(&exe);
    }
    let cmd = first_cmd_part(process.cmd().get(0).map(|s| &**s).unwrap_or(""));
    if !cmd.is_empty() {
        return cmd;
    }
    process.name().to_string()
}
