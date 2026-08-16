//! Linux-specific collectors. Each collector is intentionally fallible so a missing
//! sensor or permission never prevents the rest of RigGlow from rendering.
pub mod battery;
pub mod cpu;
pub mod disks;
pub mod display;
pub mod gpu;
pub mod memory;
pub mod network;
pub mod system;

use serde::Serialize;

#[derive(Debug, Clone, Serialize, Default)]
pub struct StaticSnapshot {
    pub system: system::SystemInfo,
    pub hardware: system::HardwareInfo,
    pub gpu: gpu::GpuInfo,
    pub disks: Vec<disks::DiskInfo>,
    pub battery: battery::BatteryInfo,
    pub display: display::DisplayInfo,
    pub network: network::NetworkInfo,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct LiveSnapshot {
    pub cpu: cpu::CpuLive,
    pub memory: memory::MemoryLive,
    pub disk: disks::IoRate,
    pub network: network::NetworkRate,
    pub battery: battery::BatteryInfo,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Snapshot {
    pub static_info: StaticSnapshot,
    pub live: LiveSnapshot,
}

pub trait StaticCollector<T> {
    fn collect_static(&self) -> anyhow::Result<T>;
}
pub trait LiveCollector<T> {
    fn refresh_live(&mut self) -> anyhow::Result<T>;
}
