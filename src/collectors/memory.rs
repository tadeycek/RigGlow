use serde::Serialize;
use sysinfo::System;

use super::LiveCollector;

#[derive(Debug, Clone, Serialize, Default)]
pub struct MemoryLive {
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub swap_used_bytes: u64,
    pub swap_total_bytes: u64,
}
impl MemoryLive {
    pub fn percent(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            self.used_bytes as f64 / self.total_bytes as f64 * 100.0
        }
    }
}
pub struct LinuxMemoryCollector {
    system: System,
}
impl LinuxMemoryCollector {
    pub fn new() -> Self {
        Self {
            system: System::new(),
        }
    }
}
impl LiveCollector<MemoryLive> for LinuxMemoryCollector {
    fn refresh_live(&mut self) -> anyhow::Result<MemoryLive> {
        self.system.refresh_memory();
        Ok(MemoryLive {
            used_bytes: self.system.used_memory(),
            total_bytes: self.system.total_memory(),
            swap_used_bytes: self.system.used_swap(),
            swap_total_bytes: self.system.total_swap(),
        })
    }
}
