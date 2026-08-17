use super::StaticCollector;
use serde::Serialize;
use std::fs;

#[derive(Debug, Clone, Serialize, Default)]
pub struct BatteryInfo {
    pub percentage: Option<f32>,
    pub health_percent: Option<f32>,
    pub status: String,
    pub time_remaining_seconds: Option<u64>,
    pub power_watts: Option<f32>,
}
pub struct LinuxBatteryCollector;
impl StaticCollector<BatteryInfo> for LinuxBatteryCollector {
    fn collect_static(&self) -> anyhow::Result<BatteryInfo> {
        collect_battery()
    }
}
impl LinuxBatteryCollector {
    pub fn collect(&self) -> BatteryInfo {
        collect_battery().unwrap_or_default()
    }
}
fn collect_battery() -> anyhow::Result<BatteryInfo> {
    let Some(root) = fs::read_dir("/sys/class/power_supply")?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("BAT"))
        })
    else {
        return Ok(BatteryInfo {
            status: "No battery".into(),
            ..BatteryInfo::default()
        });
    };
    let read = |name: &str| {
        fs::read_to_string(root.join(name))
            .ok()
            .map(|v| v.trim().to_owned())
    };
    let percentage = read("capacity").and_then(|v| v.parse().ok());
    let full = read("energy_full")
        .or_else(|| read("charge_full"))
        .and_then(|v| v.parse::<f32>().ok());
    let design = read("energy_full_design")
        .or_else(|| read("charge_full_design"))
        .and_then(|v| v.parse::<f32>().ok());
    let energy_now = read("energy_now")
        .or_else(|| read("charge_now"))
        .and_then(|v| v.parse::<f64>().ok());
    let power_now = read("power_now")
        .or_else(|| read("current_now"))
        .and_then(|v| v.parse::<f64>().ok())
        .filter(|value| *value > 0.0);
    let status = read("status").unwrap_or_else(|| "Unknown".into());
    let time_remaining_seconds = match (energy_now, full.map(f64::from), power_now) {
        (Some(now), Some(capacity), Some(rate)) if status.eq_ignore_ascii_case("charging") => {
            Some(((capacity - now).max(0.0) / rate * 3600.0) as u64)
        }
        (Some(now), _, Some(rate)) if status.eq_ignore_ascii_case("discharging") => {
            Some((now / rate * 3600.0) as u64)
        }
        _ => None,
    };
    Ok(BatteryInfo {
        percentage,
        health_percent: full
            .zip(design)
            .and_then(|(a, b)| (b > 0.0).then_some(a / b * 100.0)),
        status,
        time_remaining_seconds,
        power_watts: power_now.map(|microwatts| (microwatts / 1_000_000.0) as f32),
    })
}
