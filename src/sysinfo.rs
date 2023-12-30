use std::ops::Deref;

use sysinfo::{NetworkExt, ProcessExt, System, SystemExt, Uid};

#[derive(Debug, Default)]
pub struct SystemStats {
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
    pub cpu_usage: f32,
    pub memory_usage: f64,
    pub user_id: Option<u32>,
    pub display_name: String,
    pub run_time: u64,
}

pub fn get_system_stats() -> SystemStats {
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

        let process_stat = ProcessStat {
            pid: pid.to_string(),
            name: proc_name,
            cmd,
            exe: process.exe().to_string_lossy().to_string(),
            cpu_usage: process.cpu_usage(),
            memory_usage: process.memory() as f64,
            user_id,
            display_name,
            run_time: process.run_time(),
        };
        processes.push(process_stat);
    }

    SystemStats { processes }
}

pub fn show_statistics() {
    let mut sys = System::new_all();

    // First we update all information of our `System` struct.
    sys.refresh_all();

    // We display all disks' information:
    println!("=> disks:");
    for disk in sys.disks() {
        println!("{:?}", disk);
    }

    // Network interfaces name, data received and data transmitted:
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
    // RAM and swap information:
    println!("total memory: {} bytes", sys.total_memory());
    println!("used memory : {} bytes", sys.used_memory());
    println!("total swap  : {} bytes", sys.total_swap());
    println!("used swap   : {} bytes", sys.used_swap());

    // Display system information:
    println!("System name:             {:?}", sys.name());
    println!("System kernel version:   {:?}", sys.kernel_version());
    println!("System OS version:       {:?}", sys.os_version());
    println!("System host name:        {:?}", sys.host_name());

    // Number of CPUs:
    println!("Number of CPUs: {}", sys.cpus().len());

    // Display processes ID, name na disk usage:
    for (pid, process) in sys.processes() {
        // println!("[{}] {} {:?}", pid, process.name(), process.disk_usage());
        println!(
            "[{}] {:?} {:?} {:?} {:?} {:?}",
            pid,
            process.name(), // chrome
            process.cmd(),  // /opt/google/chrome/chrome --type=renderer ...
            process.exe(),  // /opt/google/chrome/chrome
            process.memory(),
            process.cpu_usage(),
        );
    }
}
