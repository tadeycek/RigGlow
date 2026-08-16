use crate::{
    ascii,
    ascii::builtin::distro_name,
    collectors::{Snapshot, system::uptime_human},
    output::{format_bytes, format_rate},
    theme::{Theme, color_hex},
};

pub fn render(snapshot: &Snapshot, art_source: &str, theme: &Theme, icons: bool) -> String {
    let s = &snapshot.static_info;
    let live = &snapshot.live;
    let label = |icon: &str, title: &str| {
        if icons {
            format!("{icon} {title}")
        } else {
            title.into()
        }
    };
    let rows = vec![
        (label("󰣇", "OS"), s.system.os.clone()),
        (label("󰌽", "Host"), s.system.hostname.clone()),
        (label("󰌹", "Kernel"), s.system.kernel.clone()),
        (label("󰔚", "Uptime"), uptime_human(s.system.uptime_seconds)),
        (label("󰟀", "Desktop"), s.system.desktop.clone()),
        (label("󰌽", "Terminal"), s.system.terminal.clone()),
        (
            label("󰘚", "Machine"),
            format!("{} {}", s.hardware.manufacturer, s.hardware.model),
        ),
        (label("󰌢", "Board"), s.hardware.board.clone()),
        (label("󰏖", "BIOS"), s.hardware.bios.clone()),
        (label("󰘚", "CPU"), s.hardware.cpu_model.clone()),
        (
            label("󰅪", "Cores"),
            format!(
                "{} physical / {} logical",
                s.hardware.physical_cpus, s.hardware.logical_cpus
            ),
        ),
        (
            label("󰍛", "Memory"),
            format!(
                "{} / {}",
                format_bytes(live.memory.used_bytes as f64),
                format_bytes(live.memory.total_bytes as f64)
            ),
        ),
        (label("󰢮", "GPU"), s.gpu.model.clone()),
        (
            label("󰋊", "Disk"),
            s.disks
                .first()
                .map(|d| format!("{} {}", d.model, format_bytes(d.capacity_bytes as f64)))
                .unwrap_or_else(|| "Unknown".into()),
        ),
        (
            label("󰖩", "Network"),
            format!("{} ({})", s.network.interface, s.network.local_ip),
        ),
        (
            label("󰁹", "Display"),
            format!("{} @ {}", s.display.resolution, s.display.refresh_rate),
        ),
        (
            label("󰁹", "Battery"),
            s.battery
                .percentage
                .map(|p| {
                    format!(
                        "{p:.0}% {}{}",
                        s.battery.status,
                        s.battery
                            .health_percent
                            .map(|h| format!(", health {h:.0}%"))
                            .unwrap_or_default()
                    )
                })
                .unwrap_or_else(|| s.battery.status.clone()),
        ),
        (
            label("󰖡", "Live"),
            format!(
                "CPU {:.0}%  ↓ {}  ↑ {}",
                live.cpu.usage_percent,
                format_rate(live.network.download_bytes_per_sec),
                format_rate(live.network.upload_bytes_per_sec)
            ),
        ),
    ];
    let art = ascii::load(art_source, &distro_name(), theme).unwrap_or_default();
    let art_plain: Vec<_> = art
        .into_iter()
        .map(|l| {
            l.spans
                .into_iter()
                .map(|p| p.content.into_owned())
                .collect::<String>()
        })
        .collect();
    let width = 28usize;
    let mut output = format!("\x1b[38;2;{}mRigGlow\x1b[0m\n", rgb(theme.primary));
    for index in 0..rows.len().max(art_plain.len()) {
        let left = art_plain.get(index).map(String::as_str).unwrap_or("");
        let right = rows
            .get(index)
            .map(|(k, v)| format!("\x1b[38;2;{}m{:<10}\x1b[0m {}", rgb(theme.secondary), k, v))
            .unwrap_or_default();
        output.push_str(&format!(
            "\x1b[38;2;{}m{left:<width$}\x1b[0m {right}\n",
            rgb(theme.primary),
            width = width
        ));
    }
    output
}
fn rgb(theme_color: ratatui::style::Color) -> String {
    color_hex(theme_color)
        .trim_start_matches('#')
        .to_string()
        .as_bytes()
        .chunks(2)
        .map(|c| {
            u8::from_str_radix(std::str::from_utf8(c).unwrap_or("00"), 16)
                .unwrap_or(0)
                .to_string()
        })
        .collect::<Vec<_>>()
        .join(";")
}
