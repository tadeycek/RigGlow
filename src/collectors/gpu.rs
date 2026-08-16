use super::{LiveCollector, StaticCollector};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone, Serialize, Default)]
pub struct GpuInfo {
    pub model: String,
    pub vendor: String,
    pub temperature_c: Option<f32>,
    pub utilization_percent: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct GpuLive {
    pub usage_percent: Option<f32>,
    pub temperature_c: Option<f32>,
    pub frequency_mhz: Option<u64>,
    pub vram_used_bytes: Option<u64>,
    pub vram_total_bytes: Option<u64>,
}

pub struct LinuxGpuCollector;
impl StaticCollector<GpuInfo> for LinuxGpuCollector {
    fn collect_static(&self) -> anyhow::Result<GpuInfo> {
        Ok(collect_gpu())
    }
}

pub struct LinuxGpuLiveCollector {
    device: Option<PathBuf>,
    vendor: String,
}

impl LinuxGpuLiveCollector {
    pub fn new() -> Self {
        let device = gpu_device();
        let vendor = device
            .as_ref()
            .and_then(|path| fs::read_to_string(path.join("vendor")).ok())
            .map(|id| match id.trim() {
                "0x8086" => "Intel",
                "0x1002" => "AMD",
                "0x10de" => "NVIDIA",
                _ => "Unknown",
            })
            .unwrap_or("Unknown")
            .to_owned();
        Self { device, vendor }
    }
}

impl LiveCollector<GpuLive> for LinuxGpuLiveCollector {
    fn refresh_live(&mut self) -> anyhow::Result<GpuLive> {
        let Some(device) = self.device.as_ref() else {
            return Ok(GpuLive::default());
        };
        if self.vendor == "NVIDIA" {
            return Ok(nvidia_live().unwrap_or_default());
        }
        let usage_percent = read_number(device.join("gpu_busy_percent")).map(|v| v as f32);
        let temperature_c = hwmon_number(device, "temp1_input").map(|v| v as f32 / 1000.0);
        let frequency_mhz = hwmon_number(device, "freq1_input")
            .map(|v| v / 1_000_000)
            .filter(|v| *v > 0);
        Ok(GpuLive {
            usage_percent,
            temperature_c,
            frequency_mhz,
            vram_used_bytes: read_number(device.join("mem_info_vram_used")),
            vram_total_bytes: read_number(device.join("mem_info_vram_total")),
        })
    }
}
fn collect_gpu() -> GpuInfo {
    if let Some(device) = gpu_device() {
        let vendor_id = fs::read_to_string(device.join("vendor")).unwrap_or_default();
        let vendor = match vendor_id.trim() {
            "0x8086" => "Intel",
            "0x1002" => "AMD",
            "0x10de" => "NVIDIA",
            _ => "Unknown",
        }
        .to_owned();
        if vendor != "Unknown" {
            let device_id = fs::read_to_string(device.join("device")).unwrap_or_default();
            let mut gpu = GpuInfo {
                model: format!("{vendor} Graphics ({})", device_id.trim()),
                vendor,
                temperature_c: None,
                utilization_percent: None,
            };
            if gpu.vendor == "NVIDIA"
                && let Some(nvidia) = nvidia_smi()
            {
                gpu = nvidia;
            }
            return gpu;
        }
    }
    GpuInfo::default()
}

fn gpu_device() -> Option<PathBuf> {
    fs::read_dir("/sys/class/drm")
        .ok()?
        .flatten()
        .find_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.starts_with("card") || name.contains('-') {
                return None;
            }
            let device = entry.path().join("device");
            fs::read_to_string(device.join("vendor"))
                .ok()
                .filter(|vendor| matches!(vendor.trim(), "0x8086" | "0x1002" | "0x10de"))
                .map(|_| device)
        })
}

fn read_number(path: PathBuf) -> Option<u64> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}
fn hwmon_number(device: &Path, name: &str) -> Option<u64> {
    fs::read_dir(device.join("hwmon"))
        .ok()?
        .flatten()
        .find_map(|entry| read_number(entry.path().join(name)))
}

/// Optional NVIDIA enhancement. It runs only during a static collection/explicit
/// refresh and never makes NVIDIA software a runtime dependency.
fn nvidia_smi() -> Option<GpuInfo> {
    let output = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,utilization.gpu,temperature.gpu",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let line = String::from_utf8(output.stdout)
        .ok()?
        .lines()
        .next()?
        .to_owned();
    let fields: Vec<_> = line.split(',').map(str::trim).collect();
    let model = fields.first()?.to_string();
    Some(GpuInfo {
        model,
        vendor: "NVIDIA".into(),
        utilization_percent: fields.get(1).and_then(|v| v.parse().ok()),
        temperature_c: fields.get(2).and_then(|v| v.parse().ok()),
    })
}

fn nvidia_live() -> Option<GpuLive> {
    let output = Command::new("nvidia-smi").args(["--query-gpu=utilization.gpu,temperature.gpu,clocks.current.graphics,memory.used,memory.total", "--format=csv,noheader,nounits"]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let fields: Vec<_> = stdout.lines().next()?.split(',').map(str::trim).collect();
    Some(GpuLive {
        usage_percent: fields.first().and_then(|v| v.parse().ok()),
        temperature_c: fields.get(1).and_then(|v| v.parse().ok()),
        frequency_mhz: fields.get(2).and_then(|v| v.parse().ok()),
        vram_used_bytes: fields
            .get(3)
            .and_then(|v| v.parse::<u64>().ok())
            .map(|v| v * 1024 * 1024),
        vram_total_bytes: fields
            .get(4)
            .and_then(|v| v.parse::<u64>().ok())
            .map(|v| v * 1024 * 1024),
    })
}
