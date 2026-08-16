use crate::{collectors::Snapshot, output::format_bytes};
pub fn render(snapshot: &Snapshot) -> String {
    let s = &snapshot.static_info;
    let live = &snapshot.live;
    format!(
        "RigGlow  | {} | {}\nCPU      | {} ({:.0}%)\nMemory   | {} / {}\nNetwork  | {}\n",
        s.system.os,
        s.system.hostname,
        s.hardware.cpu_model,
        live.cpu.usage_percent,
        format_bytes(live.memory.used_bytes as f64),
        format_bytes(live.memory.total_bytes as f64),
        s.network.local_ip
    )
}
