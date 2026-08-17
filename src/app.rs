use std::{collections::VecDeque, thread, time::Duration};

use crate::{
    collectors::{self, LiveCollector, StaticCollector},
    config::Settings,
    theme::{Theme, builtin},
};

pub const HISTORY_LIMIT: usize = 180;
const NETWORK_MIN_SCALE: f64 = 128.0 * 1024.0;
const STARTUP_SAMPLES: usize = 6;
const STARTUP_SAMPLE_INTERVAL: Duration = Duration::from_millis(40);

pub struct App {
    pub settings: Settings,
    pub snapshot: collectors::Snapshot,
    pub cpu_history: VecDeque<f64>,
    pub gpu_history: VecDeque<f64>,
    pub memory_history: VecDeque<f64>,
    pub network_down_history: VecDeque<f64>,
    pub network_up_history: VecDeque<f64>,
    pub network_scale: f64,
    pub activity_sort: collectors::processes::ActivitySort,
    pub show_help: bool,
    pub show_graphs: bool,
    pub show_logo: bool,
    pub compact: bool,
    pub animation_step: usize,
    theme_index: usize,
    theme_changed: bool,
    ascii_index: usize,
    cpu: collectors::cpu::LinuxCpuCollector,
    gpu: collectors::gpu::LinuxGpuLiveCollector,
    memory: collectors::memory::LinuxMemoryCollector,
    disk: collectors::disks::LinuxDiskCollector,
    network: collectors::network::LinuxNetworkCollector,
    battery: collectors::battery::LinuxBatteryCollector,
    processes: collectors::processes::LinuxProcessCollector,
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
            gpu_history: VecDeque::new(),
            memory_history: VecDeque::new(),
            network_down_history: VecDeque::new(),
            network_up_history: VecDeque::new(),
            network_scale: NETWORK_MIN_SCALE,
            activity_sort: collectors::processes::ActivitySort::Cpu,
            theme_index,
            theme_changed: false,
            ascii_index: 0,
            cpu: collectors::cpu::LinuxCpuCollector::new(),
            gpu: collectors::gpu::LinuxGpuLiveCollector::new(),
            memory: collectors::memory::LinuxMemoryCollector::new(),
            disk: collectors::disks::LinuxDiskCollector::new(),
            network,
            battery: collectors::battery::LinuxBatteryCollector,
            processes: collectors::processes::LinuxProcessCollector::new(),
        };
        app.refresh_static();
        app.refresh_live();
        app
    }

    pub fn prime_history(&mut self) {
        // Build a small, genuine initial trail. It avoids a single-point graph
        // without inventing a flat history that visibly shifts on every tick.
        for _ in 1..STARTUP_SAMPLES {
            thread::sleep(STARTUP_SAMPLE_INTERVAL);
            self.refresh_live_without_network();
        }
    }
    pub fn theme(&self) -> &'static Theme {
        &builtin::all()[self.theme_index]
    }
    pub fn cycle_theme(&mut self) {
        self.theme_index = (self.theme_index + 1) % builtin::all().len();
        self.settings.theme = self.theme().name.into();
        self.theme_changed = true;
    }
    pub fn theme_changed(&self) -> bool {
        self.theme_changed
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
        self.snapshot.static_info.filesystems = disks.collect_filesystems();
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
        self.refresh_live_with_network(true);
    }

    fn refresh_live_without_network(&mut self) {
        self.refresh_live_with_network(false);
    }

    fn refresh_live_with_network(&mut self, collect_network: bool) {
        if self.settings.animation {
            self.animation_step = self.animation_step.wrapping_add(1);
        }
        if let Ok(value) = self.cpu.refresh_live() {
            self.snapshot.live.cpu = value;
        }
        if let Ok(value) = self.gpu.refresh_live() {
            self.snapshot.live.gpu = value;
        }
        if let Ok(value) = self.memory.refresh_live() {
            self.snapshot.live.memory = value;
        }
        if let Ok(value) = self.disk.refresh_live() {
            self.snapshot.live.disk = value;
        }
        if collect_network && let Ok(value) = self.network.refresh_live() {
            self.snapshot.live.network = value;
        }
        self.snapshot.live.battery = self.battery.collect();
        self.snapshot.live.top_processes = self.processes.top(self.activity_sort);
        self.snapshot.static_info.battery = self.snapshot.live.battery.clone();
        let cpu = self.snapshot.live.cpu.usage_percent as f64;
        let memory = self.snapshot.live.memory.percent();
        push(&mut self.cpu_history, cpu);
        push(
            &mut self.gpu_history,
            self.snapshot.live.gpu.usage_percent.unwrap_or(0.0) as f64,
        );
        push(&mut self.memory_history, memory);
        if collect_network {
            let download = self.snapshot.live.network.download_bytes_per_sec;
            let upload = self.snapshot.live.network.upload_bytes_per_sec;
            // A new live sample is normalized once against a stable stepped
            // ceiling. Its visual tower is therefore immutable as it
            // scrolls left, matching btop's history behavior.
            let peak = download.max(upload);
            // Use powers-of-two ceilings and 25% headroom. Values already in
            // history are normalized when captured, so a later ceiling change
            // never redraws or morphs an old tower.
            self.network_scale = self.network_scale.max(network_scale_for(peak));
            push(
                &mut self.network_down_history,
                download / self.network_scale * 100.0,
            );
            push(
                &mut self.network_up_history,
                upload / self.network_scale * 100.0,
            );
        }
    }

    pub fn toggle_activity_sort(&mut self) {
        self.activity_sort.toggle();
        self.snapshot.live.top_processes = self.processes.top(self.activity_sort);
    }
}

fn network_scale_for(peak: f64) -> f64 {
    let required = (peak.max(0.0) * 1.25).max(NETWORK_MIN_SCALE);
    let mut scale = NETWORK_MIN_SCALE;
    while scale < required {
        scale *= 2.0;
    }
    scale
}
fn push(history: &mut VecDeque<f64>, sample: f64) {
    if history.len() >= HISTORY_LIMIT {
        history.pop_front();
    }
    history.push_back(sample);
}

#[cfg(test)]
mod tests {
    use super::{NETWORK_MIN_SCALE, network_scale_for, push};
    use std::collections::VecDeque;

    #[test]
    fn history_only_contains_real_samples() {
        let mut history = VecDeque::new();
        push(&mut history, 42.0);
        assert_eq!(history, VecDeque::from([42.0]));

        push(&mut history, 58.0);
        assert_eq!(history.len(), 2);
        assert_eq!(history.back(), Some(&58.0));
    }

    #[test]
    fn network_scale_uses_rounded_headroom() {
        assert_eq!(network_scale_for(0.0), NETWORK_MIN_SCALE);
        assert_eq!(network_scale_for(130.0 * 1024.0), 256.0 * 1024.0);
        assert_eq!(network_scale_for(900.0 * 1024.0), 2.0 * 1024.0 * 1024.0);
    }
}
