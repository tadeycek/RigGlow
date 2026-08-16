use super::StaticCollector;
use serde::Serialize;
use std::fs;

#[derive(Debug, Clone, Serialize, Default)]
pub struct BatteryInfo {
    pub percentage: Option<f32>,
    pub health_percent: Option<f32>,
    pub status: String,
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
    Ok(BatteryInfo {
        percentage,
        health_percent: full
            .zip(design)
            .and_then(|(a, b)| (b > 0.0).then_some(a / b * 100.0)),
        status: read("status").unwrap_or_else(|| "Unknown".into()),
    })
}
