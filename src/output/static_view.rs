use crate::{
    ascii,
    ascii::builtin::{CANVAS_WIDTH, distro_name},
    collectors::{Snapshot, system::uptime_human},
    output::{format_bytes, format_rate},
    theme::Theme,
};
use ratatui::style::Color;

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
                .map(|d| {
                    format!(
                        "{} {}",
                        d.model.replace("Sandisk", "SanDisk"),
                        format_bytes(d.capacity_bytes as f64)
                    )
                })
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
    let art_colored = art
        .into_iter()
        .map(|l| {
            let mut width = 0;
            let mut text = String::new();
            for span in l.spans {
                width += span.content.chars().count();
                text.push_str(&ansi_foreground(span.style.fg.unwrap_or(theme.primary)));
                text.push_str(&span.content);
            }
            text.push_str("\x1b[0m");
            (text, width)
        })
        .collect::<Vec<_>>();
    let art_width = art_colored
        .iter()
        .map(|(_, width)| *width)
        .max()
        .unwrap_or(CANVAS_WIDTH)
        .max(CANVAS_WIDTH);
    let mut output = format!(
        "{}RigGlow\x1b[0m  {}hardware snapshot\x1b[0m\n",
        ansi_foreground(theme.primary),
        ansi_foreground(theme.muted),
    );
    for index in 0..rows.len().max(art_colored.len()) {
        let (left, left_width) = art_colored
            .get(index)
            .map(|(text, width)| (text.as_str(), *width))
            .unwrap_or(("", 0));
        let right = rows
            .get(index)
            .map(|(k, v)| {
                format!(
                    "{}{k:<12}\x1b[0m {}{v}\x1b[0m",
                    ansi_foreground(theme.secondary),
                    ansi_foreground(theme.foreground),
                )
            })
            .unwrap_or_default();
        output.push_str(left);
        output.push_str(&" ".repeat(art_width.saturating_sub(left_width) + 4));
        output.push_str(&right);
        output.push('\n');
    }
    output
}

fn ansi_foreground(color: Color) -> String {
    match color {
        Color::Reset => "\x1b[39m".into(),
        Color::Indexed(index) => format!("\x1b[38;5;{index}m"),
        other => {
            let (red, green, blue) = rgb(other);
            format!("\x1b[38;2;{red};{green};{blue}m")
        }
    }
}

fn rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(red, green, blue) => (red, green, blue),
        Color::Black => (0, 0, 0),
        Color::Red => (255, 85, 85),
        Color::Green => (80, 250, 123),
        Color::Yellow => (241, 250, 140),
        Color::Blue => (139, 233, 253),
        Color::Magenta => (189, 147, 249),
        Color::Cyan => (139, 233, 253),
        Color::Gray => (205, 205, 205),
        Color::DarkGray => (112, 112, 112),
        Color::LightRed => (255, 121, 121),
        Color::LightGreen => (120, 255, 150),
        Color::LightYellow => (255, 244, 140),
        Color::LightBlue => (160, 210, 255),
        Color::LightMagenta => (220, 170, 255),
        Color::LightCyan => (140, 240, 255),
        Color::White => (255, 255, 255),
        Color::Reset | Color::Indexed(_) => unreachable!("handled by ansi_foreground"),
    }
}
