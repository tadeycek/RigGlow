use super::StaticCollector;
use serde::Serialize;
use std::{fs, process::Command};

#[derive(Debug, Clone, Serialize, Default)]
pub struct GpuInfo {
    pub model: String,
    pub vendor: String,
    pub temperature_c: Option<f32>,
    pub utilization_percent: Option<f32>,
}
pub struct LinuxGpuCollector;
impl StaticCollector<GpuInfo> for LinuxGpuCollector {
    fn collect_static(&self) -> anyhow::Result<GpuInfo> {
        Ok(collect_gpu())
    }
}
fn collect_gpu() -> GpuInfo {
    let Ok(entries) = fs::read_dir("/sys/class/drm") else {
        return GpuInfo::default();
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }
        let device = entry.path().join("device");
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
