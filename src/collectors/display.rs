use super::StaticCollector;
use serde::Serialize;
use std::fs;

#[derive(Debug, Clone, Serialize, Default)]
pub struct DisplayInfo {
    pub resolution: String,
    pub refresh_rate: String,
}
pub struct LinuxDisplayCollector;
impl StaticCollector<DisplayInfo> for LinuxDisplayCollector {
    fn collect_static(&self) -> anyhow::Result<DisplayInfo> {
        for connector in fs::read_dir("/sys/class/drm")?.flatten() {
            let root = connector.path();
            if fs::read_to_string(root.join("status"))
                .ok()
                .is_some_and(|v| v.trim() == "connected")
            {
                let mode = fs::read_to_string(root.join("modes"))
                    .ok()
                    .and_then(|v| v.lines().next().map(str::to_owned))
                    .unwrap_or_else(|| "Unknown".into());
                return Ok(DisplayInfo {
                    resolution: mode,
                    refresh_rate: "Unknown".into(),
                });
            }
        }
        Ok(DisplayInfo::default())
    }
}
