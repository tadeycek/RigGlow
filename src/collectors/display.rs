use super::StaticCollector;
use serde::Serialize;
use std::{fs, process::Command};

#[derive(Debug, Clone, Serialize, Default)]
pub struct DisplayOutput {
    pub name: String,
    pub resolution: String,
    pub refresh_rate: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct DisplayInfo {
    pub resolution: String,
    pub refresh_rate: String,
    pub displays: Vec<DisplayOutput>,
}
pub struct LinuxDisplayCollector;
impl StaticCollector<DisplayInfo> for LinuxDisplayCollector {
    fn collect_static(&self) -> anyhow::Result<DisplayInfo> {
        let outputs = kscreen_outputs();
        if let Some(primary) = outputs.first() {
            return Ok(DisplayInfo {
                resolution: primary.resolution.clone(),
                refresh_rate: primary.refresh_rate.clone(),
                displays: outputs,
            });
        }
        let mut displays = Vec::new();
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
                displays.push(DisplayOutput {
                    name: root
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                    resolution: mode,
                    refresh_rate: "Unknown".into(),
                });
            }
        }
        let Some(primary) = displays.first().cloned() else {
            return Ok(DisplayInfo::default());
        };
        Ok(DisplayInfo {
            resolution: primary.resolution,
            refresh_rate: primary.refresh_rate,
            displays,
        })
    }
}

fn kscreen_outputs() -> Vec<DisplayOutput> {
    let Ok(output) = Command::new("kscreen-doctor").arg("-o").output() else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    let text = strip_ansi(&String::from_utf8_lossy(&output.stdout));
    let mut current = None;
    let mut displays = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Output: ") {
            current = rest.split_whitespace().nth(1).map(str::to_owned);
            continue;
        }
        if let (Some(name), Some(mode)) = (current.as_ref(), trimmed.strip_prefix("Modes: ")) {
            let Some(active) = mode.split_whitespace().find(|part| part.contains('*')) else {
                continue;
            };
            let active = active.split('*').next().unwrap_or(active);
            let active = active
                .split_once(':')
                .filter(|(index, _)| index.chars().all(|character| character.is_ascii_digit()))
                .map(|(_, mode)| mode)
                .unwrap_or(active);
            let (resolution, refresh_rate) = active
                .split_once('@')
                .map(|(resolution, hz)| (resolution.to_owned(), format_refresh_rate(hz)))
                .unwrap_or_else(|| (active.to_owned(), "Unknown".into()));
            displays.push(DisplayOutput {
                name: name.clone(),
                resolution,
                refresh_rate,
            });
            current = None;
        }
    }
    displays
}

fn format_refresh_rate(rate: &str) -> String {
    rate.trim()
        .parse::<f64>()
        .map(|value| format!("{value:.0} Hz"))
        .unwrap_or_else(|_| format!("{rate} Hz"))
}

fn strip_ansi(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\u{1b}' && chars.next_if_eq(&'[').is_some() {
            for control in chars.by_ref() {
                if ('@'..='~').contains(&control) {
                    break;
                }
            }
        } else {
            output.push(character);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::format_refresh_rate;

    #[test]
    fn rounds_display_refresh_rates_for_readability() {
        assert_eq!(format_refresh_rate("120.00"), "120 Hz");
        assert_eq!(format_refresh_rate("59.94"), "60 Hz");
    }
}
