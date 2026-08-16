use std::{fs, time::Duration};

use serde::Serialize;
use sysinfo::System;

use super::LiveCollector;

#[derive(Debug, Clone, Serialize, Default)]
pub struct CpuLive {
    pub usage_percent: f32,
    pub per_core_percent: Vec<f32>,
    pub frequency_mhz: u64,
    pub temperature_c: Option<f32>,
}

pub struct LinuxCpuCollector {
    system: System,
    previous_cores: Option<Vec<CoreTimes>>,
}

impl LinuxCpuCollector {
    pub fn new() -> Self {
        Self {
            system: System::new(),
            previous_cores: None,
        }
    }
}

impl LiveCollector<CpuLive> for LinuxCpuCollector {
    fn refresh_live(&mut self) -> anyhow::Result<CpuLive> {
        if self.previous_cores.is_none() {
            self.previous_cores = Some(read_core_times());
        }
        self.system.refresh_cpu_usage();
        std::thread::sleep(Duration::from_millis(20));
        self.system.refresh_cpu_usage();
        self.system.refresh_cpu_frequency();
        let current_cores = read_core_times();
        let per_core_percent = self
            .previous_cores
            .as_ref()
            .map(|previous| core_usage(previous, &current_cores))
            .unwrap_or_else(|| vec![0.0; current_cores.len()]);
        self.previous_cores = Some(current_cores);
        Ok(CpuLive {
            usage_percent: self.system.global_cpu_usage(),
            per_core_percent,
            frequency_mhz: self
                .system
                .cpus()
                .first()
                .map(|cpu| cpu.frequency())
                .unwrap_or(0),
            temperature_c: cpu_temperature(),
        })
    }
}

#[derive(Clone, Copy)]
struct CoreTimes {
    total: u64,
    idle: u64,
}

fn read_core_times() -> Vec<CoreTimes> {
    let Ok(content) = fs::read_to_string("/proc/stat") else {
        return Vec::new();
    };
    content
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let name = fields.next()?;
            if !name.starts_with("cpu") || name == "cpu" {
                return None;
            }
            let values: Vec<u64> = fields.filter_map(|value| value.parse().ok()).collect();
            if values.len() < 5 {
                return None;
            }
            Some(CoreTimes {
                total: values.iter().sum(),
                idle: values[3].saturating_add(values[4]),
            })
        })
        .collect()
}

fn core_usage(previous: &[CoreTimes], current: &[CoreTimes]) -> Vec<f32> {
    previous
        .iter()
        .zip(current)
        .map(|(old, new)| {
            let total = new.total.saturating_sub(old.total);
            if total == 0 {
                return 0.0;
            }
            let busy = total.saturating_sub(new.idle.saturating_sub(old.idle));
            busy as f32 / total as f32 * 100.0
        })
        .collect()
}

fn cpu_temperature() -> Option<f32> {
    let mut candidates = Vec::new();
    for entry in fs::read_dir("/sys/class/thermal").ok()?.flatten() {
        let path = entry.path().join("temp");
        if let Ok(raw) = fs::read_to_string(path)
            && let Ok(value) = raw.trim().parse::<f32>()
            && (1_000.0..150_000.0).contains(&value)
        {
            candidates.push(value / 1000.0);
        }
    }
    candidates.into_iter().reduce(f32::max)
}
