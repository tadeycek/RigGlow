use std::{collections::HashMap, fs, net::UdpSocket, time::Instant};

use serde::Serialize;

use super::LiveCollector;

#[derive(Debug, Clone, Serialize, Default)]
pub struct NetworkInfo {
    pub interface: String,
    pub local_ip: String,
}
#[derive(Debug, Clone, Copy, Serialize, Default)]
pub struct NetworkRate {
    pub download_bytes_per_sec: f64,
    pub upload_bytes_per_sec: f64,
}
#[derive(Debug, Clone, Copy, Default)]
struct Counters {
    received: u64,
    transmitted: u64,
}
pub struct LinuxNetworkCollector {
    previous: Option<(Counters, Instant)>,
    active: String,
}
impl LinuxNetworkCollector {
    pub fn new() -> Self {
        let active = active_interface().unwrap_or_else(|| "Unknown".into());
        Self {
            previous: None,
            active,
        }
    }
    pub fn static_info(&self) -> NetworkInfo {
        NetworkInfo {
            interface: self.active.clone(),
            local_ip: local_ip().unwrap_or_else(|| "Unknown".into()),
        }
    }
}
impl LiveCollector<NetworkRate> for LinuxNetworkCollector {
    fn refresh_live(&mut self) -> anyhow::Result<NetworkRate> {
        let now = Instant::now();
        let all = counters()?;
        let current = all
            .get(&self.active)
            .copied()
            .or_else(|| all.values().copied().max_by_key(|x| x.received))
            .unwrap_or_default();
        let rate = self
            .previous
            .map(|(old, then)| rate(old, current, now.saturating_duration_since(then)))
            .unwrap_or_default();
        self.previous = Some((current, now));
        Ok(rate)
    }
}
fn counters() -> anyhow::Result<HashMap<String, Counters>> {
    let mut out = HashMap::new();
    for line in fs::read_to_string("/proc/net/dev")?.lines().skip(2) {
        let Some((name, values)) = line.split_once(':') else {
            continue;
        };
        let fields: Vec<_> = values.split_whitespace().collect();
        if fields.len() >= 9 {
            out.insert(
                name.trim().into(),
                Counters {
                    received: fields[0].parse().unwrap_or(0),
                    transmitted: fields[8].parse().unwrap_or(0),
                },
            );
        }
    }
    Ok(out)
}
fn active_interface() -> Option<String> {
    let routes = fs::read_to_string("/proc/net/route").ok()?;
    routes.lines().skip(1).find_map(|line| {
        let p: Vec<_> = line.split_whitespace().collect();
        (p.len() > 2 && p[1] == "00000000").then(|| p[0].to_owned())
    })
}
fn local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("192.0.2.1:80").ok()?;
    Some(socket.local_addr().ok()?.ip().to_string())
}
fn rate(old: Counters, new: Counters, elapsed: std::time::Duration) -> NetworkRate {
    let secs = elapsed.as_secs_f64();
    if secs <= f64::EPSILON {
        return NetworkRate::default();
    }
    NetworkRate {
        download_bytes_per_sec: new.received.saturating_sub(old.received) as f64 / secs,
        upload_bytes_per_sec: new.transmitted.saturating_sub(old.transmitted) as f64 / secs,
    }
}
