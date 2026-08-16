use std::{fs, process::Command, time::Instant};

use serde::Serialize;

use super::{LiveCollector, StaticCollector};

#[derive(Debug, Clone, Serialize, Default)]
pub struct DiskInfo {
    pub model: String,
    pub kind: String,
    pub capacity_bytes: u64,
}
#[derive(Debug, Clone, Serialize, Default)]
pub struct FilesystemInfo {
    pub source: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
}
#[derive(Debug, Clone, Copy, Serialize, Default)]
pub struct IoRate {
    pub read_bytes_per_sec: f64,
    pub write_bytes_per_sec: f64,
}
#[derive(Debug, Clone, Copy, Default)]
pub struct ByteCounters {
    pub read: u64,
    pub written: u64,
}

pub struct LinuxDiskCollector {
    previous: Option<(ByteCounters, Instant)>,
}
impl LinuxDiskCollector {
    pub fn new() -> Self {
        Self { previous: None }
    }
}
impl StaticCollector<Vec<DiskInfo>> for LinuxDiskCollector {
    fn collect_static(&self) -> anyhow::Result<Vec<DiskInfo>> {
        let mut disks = Vec::new();
        for entry in fs::read_dir("/sys/block")?.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with("loop") || name.starts_with("ram") {
                continue;
            }
            let root = entry.path();
            let sectors = fs::read_to_string(root.join("size"))
                .ok()
                .and_then(|v| v.trim().parse::<u64>().ok())
                .unwrap_or(0);
            let model = fs::read_to_string(root.join("device/model"))
                .ok()
                .map(|v| v.trim().to_owned())
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| name.clone());
            let rotational = fs::read_to_string(root.join("queue/rotational"))
                .ok()
                .map(|v| v.trim() == "1")
                .unwrap_or(false);
            disks.push(DiskInfo {
                model,
                kind: if rotational {
                    "HDD".into()
                } else {
                    "SSD/NVMe".into()
                },
                capacity_bytes: sectors.saturating_mul(512),
            });
        }
        Ok(disks)
    }
}
impl LinuxDiskCollector {
    /// Filesystem capacity is static metadata; `df` is never called from the live refresh loop.
    pub fn collect_filesystems(&self) -> Vec<FilesystemInfo> {
        let Ok(output) = Command::new("df")
            .args([
                "-B1",
                "-x",
                "tmpfs",
                "-x",
                "devtmpfs",
                "--output=source,size,used,avail,target",
            ])
            .output()
        else {
            return Vec::new();
        };
        if !output.status.success() {
            return Vec::new();
        }
        String::from_utf8(output.stdout)
            .ok()
            .into_iter()
            .flat_map(|text| {
                text.lines()
                    .skip(1)
                    .filter_map(parse_filesystem)
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}

fn parse_filesystem(line: &str) -> Option<FilesystemInfo> {
    let fields: Vec<_> = line.split_whitespace().collect();
    if fields.len() < 5 {
        return None;
    }
    Some(FilesystemInfo {
        source: fields[0].into(),
        total_bytes: fields[1].parse().ok()?,
        used_bytes: fields[2].parse().ok()?,
        available_bytes: fields[3].parse().ok()?,
        mount_point: fields[4..].join(" "),
    })
}
impl LiveCollector<IoRate> for LinuxDiskCollector {
    fn refresh_live(&mut self) -> anyhow::Result<IoRate> {
        let now = Instant::now();
        let current = read_disk_counters()?;
        let rate = self
            .previous
            .map(|(old, then)| byte_rate(old, current, now.saturating_duration_since(then)))
            .unwrap_or_default();
        self.previous = Some((current, now));
        Ok(rate)
    }
}
fn read_disk_counters() -> anyhow::Result<ByteCounters> {
    let mut total = ByteCounters::default();
    for line in fs::read_to_string("/proc/diskstats")?.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() < 10 || ignored_disk(fields[2]) {
            continue;
        }
        total.read = total
            .read
            .saturating_add(fields[5].parse::<u64>().unwrap_or(0).saturating_mul(512));
        total.written = total
            .written
            .saturating_add(fields[9].parse::<u64>().unwrap_or(0).saturating_mul(512));
    }
    Ok(total)
}
fn ignored_disk(name: &str) -> bool {
    ["loop", "ram", "dm-", "md", "zram"]
        .iter()
        .any(|p| name.starts_with(p))
        || name.chars().last().is_some_and(|c| c.is_ascii_digit()) && !name.starts_with("nvme")
}
pub fn byte_rate(old: ByteCounters, new: ByteCounters, elapsed: std::time::Duration) -> IoRate {
    let seconds = elapsed.as_secs_f64();
    if seconds <= f64::EPSILON {
        return IoRate::default();
    }
    IoRate {
        read_bytes_per_sec: new.read.saturating_sub(old.read) as f64 / seconds,
        write_bytes_per_sec: new.written.saturating_sub(old.written) as f64 / seconds,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rates_are_counter_deltas() {
        let rate = byte_rate(
            ByteCounters {
                read: 100,
                written: 200,
            },
            ByteCounters {
                read: 600,
                written: 1_200,
            },
            std::time::Duration::from_secs(2),
        );
        assert_eq!(rate.read_bytes_per_sec, 250.0);
        assert_eq!(rate.write_bytes_per_sec, 500.0);
    }
}
