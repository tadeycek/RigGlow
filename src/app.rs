use std::collections::VecDeque;

use crate::{
    collectors::{self, LiveCollector, StaticCollector},
    config::Settings,
    theme::{Theme, builtin},
};

pub const HISTORY_LIMIT: usize = 180;

pub struct App {
    pub settings: Settings,
    pub snapshot: collectors::Snapshot,
    pub cpu_history: VecDeque<f64>,
    pub memory_history: VecDeque<f64>,
    pub network_history: VecDeque<f64>,
    pub show_help: bool,
    pub show_graphs: bool,
    pub show_logo: bool,
    pub compact: bool,
    pub animation_step: usize,
    theme_index: usize,
    ascii_index: usize,
    cpu: collectors::cpu::LinuxCpuCollector,
    memory: collectors::memory::LinuxMemoryCollector,
    disk: collectors::disks::LinuxDiskCollector,
    network: collectors::network::LinuxNetworkCollector,
    battery: collectors::battery::LinuxBatteryCollector,
}

impl App {
    pub fn new(settings: Settings) -> Self {
        let theme_index = builtin::all()
            .iter()
            .position(|t| t.name.eq_ignore_ascii_case(&settings.theme))
            .unwrap_or(0);
        let network = collectors::network::LinuxNetworkCollector::new();
        let mut app = Self {
            show_graphs: settings.graphs,
            compact: settings.compact,
            animation_step: 0,
            show_help: false,
            show_logo: true,
            settings,
            snapshot: collectors::Snapshot::default(),
            cpu_history: VecDeque::new(),
            memory_history: VecDeque::new(),
            network_history: VecDeque::new(),
            theme_index,
            ascii_index: 0,
            cpu: collectors::cpu::LinuxCpuCollector::new(),
            memory: collectors::memory::LinuxMemoryCollector::new(),
            disk: collectors::disks::LinuxDiskCollector::new(),
            network,
            battery: collectors::battery::LinuxBatteryCollector,
        };
        app.refresh_static();
        app.refresh_live();
        app
    }
    pub fn theme(&self) -> &'static Theme {
        &builtin::all()[self.theme_index]
    }
    pub fn cycle_theme(&mut self) {
        self.theme_index = (self.theme_index + 1) % builtin::all().len();
        self.settings.theme = self.theme().name.into();
    }
    pub fn cycle_ascii(&mut self) {
        self.ascii_index = (self.ascii_index + 1) % 4;
        self.settings.ascii.source = ["auto", "rigglow", "cat", "retro"][self.ascii_index].into();
    }
    pub fn refresh_static(&mut self) {
        let system = collectors::system::LinuxSystemCollector;
        if let Ok((system_info, hardware)) = system.collect_static() {
            self.snapshot.static_info.system = system_info;
            self.snapshot.static_info.hardware = hardware;
        }
        let disks = collectors::disks::LinuxDiskCollector::new();
        if let Ok(found) = disks.collect_static() {
            self.snapshot.static_info.disks = found;
        }
        let gpu = collectors::gpu::LinuxGpuCollector;
        if let Ok(found) = gpu.collect_static() {
            self.snapshot.static_info.gpu = found;
        }
        let display = collectors::display::LinuxDisplayCollector;
        if let Ok(found) = display.collect_static() {
            self.snapshot.static_info.display = found;
        }
        let battery = collectors::battery::LinuxBatteryCollector;
        if let Ok(found) = battery.collect_static() {
            self.snapshot.static_info.battery = found;
        }
        self.snapshot.static_info.network = self.network.static_info();
    }
    pub fn refresh_live(&mut self) {
        if self.settings.animation {
            self.animation_step = self.animation_step.wrapping_add(1);
        }
        if let Ok(value) = self.cpu.refresh_live() {
            self.snapshot.live.cpu = value;
        }
        if let Ok(value) = self.memory.refresh_live() {
            self.snapshot.live.memory = value;
        }
        if let Ok(value) = self.disk.refresh_live() {
            self.snapshot.live.disk = value;
        }
        if let Ok(value) = self.network.refresh_live() {
            self.snapshot.live.network = value;
        }
        self.snapshot.live.battery = self.battery.collect();
        self.snapshot.static_info.battery = self.snapshot.live.battery.clone();
        let cpu = self.snapshot.live.cpu.usage_percent as f64;
        let memory = self.snapshot.live.memory.percent();
        let net = self.snapshot.live.network.download_bytes_per_sec
            + self.snapshot.live.network.upload_bytes_per_sec;
        push(&mut self.cpu_history, cpu);
        push(&mut self.memory_history, memory);
        push(&mut self.network_history, net);
    }
}
fn push(history: &mut VecDeque<f64>, sample: f64) {
    if history.len() >= HISTORY_LIMIT {
        history.pop_front();
    }
    history.push_back(sample);
}
