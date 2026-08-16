use std::{fs, path::Path};

use serde::Serialize;
use sysinfo::System;

use super::StaticCollector;

#[derive(Debug, Clone, Serialize, Default)]
pub struct SystemInfo {
    pub os: String,
    pub kernel: String,
    pub hostname: String,
    pub uptime_seconds: u64,
    pub desktop: String,
    pub shell: String,
    pub terminal: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct HardwareInfo {
    pub manufacturer: String,
    pub model: String,
    pub board: String,
    pub bios: String,
    pub cpu_model: String,
    pub physical_cpus: usize,
    pub logical_cpus: usize,
    pub total_memory_bytes: u64,
}

pub struct LinuxSystemCollector;

impl StaticCollector<(SystemInfo, HardwareInfo)> for LinuxSystemCollector {
    fn collect_static(&self) -> anyhow::Result<(SystemInfo, HardwareInfo)> {
        let mut sys = System::new_all();
        sys.refresh_all();
        let system = SystemInfo {
            os: os_pretty_name()
                .unwrap_or_else(|| System::long_os_version().unwrap_or_else(|| "Unknown".into())),
            kernel: System::kernel_version().unwrap_or_else(|| "Unknown".into()),
            hostname: System::host_name().unwrap_or_else(|| "Unknown".into()),
            uptime_seconds: System::uptime(),
            desktop: env_any(&["XDG_CURRENT_DESKTOP", "DESKTOP_SESSION"]),
            shell: std::env::var("SHELL")
                .ok()
                .and_then(|v| {
                    Path::new(&v)
                        .file_name()
                        .map(|x| x.to_string_lossy().into_owned())
                })
                .unwrap_or_else(|| "Unknown".into()),
            terminal: env_any(&["TERM_PROGRAM", "TERM"]),
        };
        let hardware = HardwareInfo {
            manufacturer: sysfs("sys/class/dmi/id/sys_vendor"),
            model: sysfs("sys/class/dmi/id/product_name"),
            board: format_pair(
                "sys/class/dmi/id/board_vendor",
                "sys/class/dmi/id/board_name",
            ),
            bios: sysfs("sys/class/dmi/id/bios_version"),
            cpu_model: sys
                .cpus()
                .first()
                .map(|cpu| cpu.brand().trim().to_owned())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "Unknown".into()),
            physical_cpus: sys.physical_core_count().unwrap_or(0),
            logical_cpus: sys.cpus().len(),
            total_memory_bytes: sys.total_memory(),
        };
        Ok((system, hardware))
    }
}

fn os_pretty_name() -> Option<String> {
    let contents = fs::read_to_string("/etc/os-release").ok()?;
    contents.lines().find_map(|line| {
        line.strip_prefix("PRETTY_NAME=")
            .map(|v| v.trim_matches('"').to_owned())
    })
}

fn sysfs(relative: &str) -> String {
    fs::read_to_string(format!("/{relative}"))
        .ok()
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "Unknown".into())
}
fn format_pair(first: &str, second: &str) -> String {
    let a = sysfs(first);
    let b = sysfs(second);
    if a == "Unknown" && b == "Unknown" {
        a
    } else {
        format!("{a} {b}").trim().to_owned()
    }
}
fn env_any(keys: &[&str]) -> String {
    keys.iter()
        .find_map(|key| std::env::var(key).ok())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "Unknown".into())
}

pub fn uptime_human(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = seconds % 86_400 / 3_600;
    let minutes = seconds % 3_600 / 60;
    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}
