use std::{fs, time::Duration};

use serde::Serialize;
use sysinfo::System;

use super::LiveCollector;

#[derive(Debug, Clone, Serialize, Default)]
pub struct CpuLive {
    pub usage_percent: f32,
    pub frequency_mhz: u64,
    pub temperature_c: Option<f32>,
}

pub struct LinuxCpuCollector {
    system: System,
}

impl LinuxCpuCollector {
    pub fn new() -> Self {
        Self {
            system: System::new(),
        }
    }
}

impl LiveCollector<CpuLive> for LinuxCpuCollector {
    fn refresh_live(&mut self) -> anyhow::Result<CpuLive> {
        self.system.refresh_cpu_usage();
        std::thread::sleep(Duration::from_millis(20));
        self.system.refresh_cpu_usage();
        self.system.refresh_cpu_frequency();
        Ok(CpuLive {
            usage_percent: self.system.global_cpu_usage(),
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
