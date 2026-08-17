use super::{help, widgets};
use crate::{
    app::App,
    ascii,
    ascii::builtin::{CANVAS_HEIGHT, CANVAS_WIDTH, distro_name},
    collectors::system::uptime_human,
    output::format_bytes,
};
use ratatui::{
    prelude::*,
    widgets::{Paragraph, Wrap},
};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    frame.render_widget(
        ratatui::widgets::Block::default().style(widgets::surface_style(app)),
        area,
    );
    if area.width < 38 || area.height < 8 {
        fallback(frame, app);
        return;
    }
    let page = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(3),
        Constraint::Length(1),
    ])
    .split(area);
    title(frame, app, page[0]);
    status(frame, app, page[2]);
    if area.width >= 104 && area.height >= 25 && !app.compact {
        large(frame, app, page[1]);
    } else {
        small(frame, app, page[1]);
    }
    if app.show_help {
        help::render(frame, app);
    }
}

fn title(frame: &mut Frame, app: &App, area: Rect) {
    let glow = if app.settings.animation && app.animation_step.is_multiple_of(2) {
        app.theme().accent
    } else {
        app.theme().secondary
    };
    let title = Line::from(vec![
        Span::styled(" RIG", Style::default().fg(app.theme().primary).bold()),
        Span::styled("GLOW", Style::default().fg(glow).bold()),
        Span::styled("  HARDWARE COCKPIT", Style::default().fg(app.theme().muted)),
        Span::styled(
            format!("  ◈ {}", app.theme().name),
            Style::default().fg(app.theme().secondary),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(title).style(widgets::surface_style(app)),
        area,
    );
}

fn status(frame: &mut Frame, app: &App, area: Rect) {
    let text = if app.show_help {
        " [?] Close help"
    } else {
        " [T] Theme  [A] ASCII  [P] Activity sort  [G] Graphs  [L] Logo  [C] Compact  [S] Snapshot  [R] Refresh  [Q] Quit  [?] Help"
    };
    frame.render_widget(
        Paragraph::new(Line::styled(text, Style::default().fg(app.theme().muted)))
            .style(widgets::surface_style(app)),
        area,
    );
}

fn large(frame: &mut Frame, app: &App, area: Rect) {
    // The identity rail is the art canvas plus its two border cells. Keeping
    // it fixed avoids turning an OS logo into a large empty decorative box.
    let columns = Layout::horizontal([
        Constraint::Length((CANVAS_WIDTH + 2) as u16),
        Constraint::Length(1),
        Constraint::Min(1),
    ])
    .split(area);
    let left = Layout::vertical([
        Constraint::Length((CANVAS_HEIGHT + 2) as u16),
        Constraint::Length(1),
        Constraint::Length(6),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Min(8),
    ])
    .split(columns[0]);
    if app.show_logo {
        let lines = ascii::load(&app.settings.ascii.source, &distro_name(), app.theme())
            .unwrap_or_default();
        frame.render_widget(
            Paragraph::new(lines)
                .block(widgets::block(" OS IDENTITY ", app))
                .style(widgets::surface_style(app))
                .wrap(Wrap { trim: false }),
            left[0],
        );
    } else {
        edge_summary(frame, app, left[0]);
    }
    widgets::top_activity(app, left[2], frame);
    gpu_load_panel(frame, app, left[4]);
    gpu_history_panel(frame, app, left[6]);

    if area.height >= 42 {
        // Keep the identity cards together. Live inspection cards follow
        // beneath them in the same reading order as the hardware cockpit.
        let right = Layout::vertical([
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(7),
            Constraint::Length(1),
            Constraint::Length(7),
            Constraint::Length(1),
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Min(8),
        ])
        .split(columns[2]);
        system_panel(frame, app, right[0]);
        hardware_panel(frame, app, right[2]);
        widgets::core_grid(
            " CPU THREADS ",
            &app.snapshot.live.cpu.per_core_percent,
            right[4],
            frame,
            app,
        );
        disk_panel(frame, app, right[6]);
        performance_panel(frame, app, right[8]);
    } else {
        let right = Layout::vertical([
            Constraint::Percentage(31),
            Constraint::Percentage(35),
            Constraint::Percentage(34),
        ])
        .split(columns[2]);
        system_panel(frame, app, right[0]);
        hardware_panel(frame, app, right[1]);
        performance_panel(frame, app, right[2]);
    }
}

fn small(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Percentage(42),
        Constraint::Percentage(30),
        Constraint::Percentage(28),
    ])
    .split(area);
    system_panel(frame, app, chunks[0]);
    hardware_panel(frame, app, chunks[1]);
    if app.show_graphs && chunks[2].height >= 8 {
        performance_panel(frame, app, chunks[2]);
    } else {
        widgets::info_panel(" LIVE ", widgets::live_rows(app), chunks[2], frame, app);
    }
}

fn system_panel(frame: &mut Frame, app: &App, area: Rect) {
    if !app.settings.modules.system {
        widgets::info_panel(
            " SYSTEM ",
            vec![("Status".into(), "Hidden by configuration".into())],
            area,
            frame,
            app,
        );
        return;
    }
    system_grid(frame, app, area);
}

fn system_grid(frame: &mut Frame, app: &App, area: Rect) {
    let s = &app.snapshot.static_info;
    widgets::detail_grid(
        " SYSTEM ",
        vec![
            ("OS".into(), s.system.os.clone()),
            ("Kernel".into(), s.system.kernel.clone()),
            ("Host".into(), s.system.hostname.clone()),
            ("Uptime".into(), uptime_human(s.system.uptime_seconds)),
            ("Desktop".into(), s.system.desktop.clone()),
            ("Shell".into(), s.system.shell.clone()),
            ("Terminal".into(), s.system.terminal.clone()),
            (
                "LAN".into(),
                format!("{} · {}", s.network.interface, s.network.local_ip),
            ),
        ],
        area,
        frame,
        app,
    );
}

fn hardware_panel(frame: &mut Frame, app: &App, area: Rect) {
    let s = &app.snapshot.static_info;
    let disk = s
        .disks
        .first()
        .map(|d| drive_summary(d.capacity_bytes, &d.kind))
        .unwrap_or_else(|| "Unknown".into());
    let modules = &app.settings.modules;
    let mut rows = vec![
        (
            "Machine".into(),
            format!("{} {}", s.hardware.manufacturer, s.hardware.model),
        ),
        ("Board".into(), s.hardware.board.clone()),
        ("BIOS".into(), s.hardware.bios.clone()),
    ];
    if modules.cpu {
        rows.push(("CPU".into(), s.hardware.cpu_model.clone()));
        rows.push((
            "Cores".into(),
            format!(
                "{} cores / {} threads",
                s.hardware.physical_cpus, s.hardware.logical_cpus
            ),
        ));
    }
    if modules.memory {
        rows.push((
            "Memory".into(),
            format!(
                "{} / {} ({:.0}%)",
                format_bytes(app.snapshot.live.memory.used_bytes as f64),
                format_bytes(s.hardware.total_memory_bytes as f64),
                app.snapshot.live.memory.percent()
            ),
        ));
        let memory = &app.snapshot.live.memory;
        if app.settings.memory.show_swap || memory.swap_used_bytes > 0 {
            rows.push((
                "Swap".into(),
                format!(
                    "{} / {}",
                    format_bytes(memory.swap_used_bytes as f64),
                    format_bytes(memory.swap_total_bytes as f64)
                ),
            ));
        }
    }
    if modules.gpu {
        rows.push(("GPU".into(), s.gpu.model.clone()));
    }
    if modules.disks {
        rows.push(("Drive".into(), disk));
    }
    if modules.display {
        rows.push(("Display".into(), display_summary(s)));
    }
    if modules.battery {
        rows.push(("Battery".into(), widgets::battery_summary(app)));
    }
    widgets::detail_grid(" HARDWARE ", rows, area, frame, app);
}

fn disk_panel(frame: &mut Frame, app: &App, area: Rect) {
    if !app.settings.modules.disks {
        widgets::info_panel(
            " DISK / I-O ",
            vec![("Status".into(), "Hidden by configuration".into())],
            area,
            frame,
            app,
        );
        return;
    }
    widgets::filesystem_card(
        app.snapshot.static_info.filesystems.first(),
        app.snapshot.static_info.disks.first(),
        app.snapshot.live.disk.read_bytes_per_sec,
        app.snapshot.live.disk.write_bytes_per_sec,
        area,
        frame,
        app,
    );
}

fn edge_summary(frame: &mut Frame, app: &App, area: Rect) {
    let s = &app.snapshot.static_info;
    widgets::detail_grid(
        " AT A GLANCE ",
        vec![
            ("GPU".into(), s.gpu.model.clone()),
            ("Network".into(), s.network.interface.clone()),
            ("IP".into(), s.network.local_ip.clone()),
            ("Battery".into(), widgets::battery_summary(app)),
        ],
        area,
        frame,
        app,
    );
}

fn gpu_load_panel(frame: &mut Frame, app: &App, area: Rect) {
    let live = &app.snapshot.live.gpu;
    widgets::usage_gauge(
        " GPU LOAD ",
        live.usage_percent.unwrap_or(0.0) as f64,
        widgets::gpu_summary(app),
        area,
        frame,
        app,
    );
}

fn gpu_history_panel(frame: &mut Frame, app: &App, area: Rect) {
    if !app.show_graphs || area.height < 5 {
        return;
    }
    let live = &app.snapshot.live.gpu;
    widgets::single_history(
        widgets::SingleHistorySpec {
            title: format!(
                " GPU HISTORY · {} ",
                live.usage_percent
                    .map(|usage| format!("{usage:.0}%"))
                    .unwrap_or_else(|| "—".into())
            ),
            history: &app.gpu_history,
            max: 100.0,
            upper_label: "100%".into(),
            color: app.theme().graph[2],
        },
        area,
        frame,
        app,
    );
}

fn performance_panel(frame: &mut Frame, app: &App, area: Rect) {
    if !app.show_graphs || area.height < 7 {
        widgets::info_panel(" LIVE METRICS ", widgets::live_rows(app), area, frame, app);
        return;
    }
    let charts =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);
    widgets::paired_history(
        widgets::PairedHistorySpec {
            title: cpu_memory_title(app),
            // CPU grows upward and RAM downward from the shared zero rail,
            // matching the upload/download btop-style network panel.
            upper: &app.cpu_history,
            lower: &app.memory_history,
            max: 100.0,
            scale: widgets::HistoryScale::Linear,
            upper_label: "CPU 100%".into(),
            lower_label: "RAM 100%".into(),
            upper_color: app.theme().graph[0],
            lower_color: memory_history_color(app),
        },
        charts[0],
        frame,
        app,
    );
    let net = &app.snapshot.live.network;
    widgets::paired_history(
        widgets::PairedHistorySpec {
            title: Line::from(vec![Span::styled(
                format!(
                    " NETWORK  {} · ↓{} · ↑{}{} ",
                    app.snapshot.static_info.network.interface,
                    compact_network_rate(net.download_bytes_per_sec),
                    compact_network_rate(net.upload_bytes_per_sec),
                    net.latency_ms
                        .map(|ms| format!(" · {ms:.0}ms"))
                        .unwrap_or_default(),
                ),
                Style::default().fg(app.theme().primary).bold(),
            )]),
            upper: &app.network_up_history,
            lower: &app.network_down_history,
            max: 100.0,
            scale: widgets::HistoryScale::Linear,
            upper_label: format!("UP MAX {}", format_network_scale(app.network_scale)),
            lower_label: format!("DOWN MAX {}", format_network_scale(app.network_scale)),
            upper_color: app.theme().graph[0],
            lower_color: app.theme().graph[1],
        },
        charts[1],
        frame,
        app,
    );
}

fn format_gib(bytes: u64) -> String {
    format!("{:.1}", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
}

fn cpu_memory_title(app: &App) -> Line<'static> {
    let cpu = &app.snapshot.live.cpu;
    let cpu_color = threshold_color(cpu.usage_percent as f64, 75.0, 90.0, app);
    let memory_color = threshold_color(
        app.snapshot.live.memory.percent(),
        app.settings.memory.warning_percent,
        app.settings.memory.critical_percent,
        app,
    );
    Line::from(vec![
        Span::styled(" CPU ", Style::default().fg(app.theme().primary).bold()),
        Span::styled(
            format!("{:.0}%", cpu.usage_percent),
            Style::default().fg(cpu_color).bold(),
        ),
        Span::styled(
            format!(
                " · {} MHz{}{} │ RAM ",
                cpu.frequency_mhz,
                cpu.temperature_c
                    .map(|value| format!(" · {value:.0}°C"))
                    .unwrap_or_default(),
                cpu.package_power_watts
                    .map(|value| format!(" · {value:.0} W"))
                    .unwrap_or_default(),
            ),
            Style::default().fg(app.theme().foreground),
        ),
        Span::styled(
            format!(
                "{}/{} GiB · {:.0}% ",
                format_gib(app.snapshot.live.memory.used_bytes),
                format_gib(app.snapshot.live.memory.total_bytes),
                app.snapshot.live.memory.percent(),
            ),
            Style::default().fg(memory_color).bold(),
        ),
    ])
}

fn threshold_color(value: f64, warning: f64, critical: f64, app: &App) -> Color {
    if value >= critical {
        app.theme().critical
    } else if value >= warning {
        app.theme().warning
    } else {
        app.theme().foreground
    }
}

fn compact_network_rate(bytes_per_second: f64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * KIB;
    if bytes_per_second >= MIB {
        format!("{:.1}M/s", bytes_per_second / MIB)
    } else if bytes_per_second >= KIB {
        format!("{:.1}K/s", bytes_per_second / KIB)
    } else if bytes_per_second <= f64::EPSILON {
        "—".into()
    } else {
        format!("{bytes_per_second:.0}B/s")
    }
}

fn format_network_scale(bytes_per_second: f64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * KIB;
    if bytes_per_second >= MIB {
        format!("{:.0} MiB/s", bytes_per_second / MIB)
    } else {
        format!("{:.0} KiB/s", bytes_per_second / KIB)
    }
}

fn drive_summary(bytes: u64, kind: &str) -> String {
    let kind = if kind == "SSD/NVMe" { "NVMe" } else { kind };
    let decimal_tb = bytes as f64 / 1_000_000_000_000.0;
    if decimal_tb >= 0.95 {
        format!("{decimal_tb:.0} TB {kind}")
    } else {
        format!("{:.0} GB {kind}", bytes as f64 / 1_000_000_000.0)
    }
}

fn display_summary(snapshot: &crate::collectors::StaticSnapshot) -> String {
    if snapshot.display.displays.len() > 1 {
        return snapshot
            .display
            .displays
            .iter()
            .map(|display| {
                format!(
                    "{} {} @ {}",
                    display.name, display.resolution, display.refresh_rate
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
    }
    format!(
        "{} @ {}",
        snapshot.display.resolution, snapshot.display.refresh_rate
    )
}

fn memory_history_color(app: &App) -> Color {
    let percent = app.snapshot.live.memory.percent();
    if percent >= app.settings.memory.critical_percent {
        app.theme().critical
    } else if percent >= app.settings.memory.warning_percent {
        app.theme().warning
    } else {
        app.theme().graph[1]
    }
}

fn fallback(frame: &mut Frame, app: &App) {
    let live = &app.snapshot.live;
    let text = format!(
        " RIGGLOW\n\n CPU {:.0}% · GPU {} · RAM {:.0}%\n\nTerminal too small — resize for dashboard\n[Q] Quit",
        live.cpu.usage_percent,
        live.gpu
            .usage_percent
            .map(|value| format!("{value:.0}%"))
            .unwrap_or_else(|| "N/A".into()),
        live.memory.percent()
    );
    frame.render_widget(
        Paragraph::new(text)
            .style(widgets::surface_style(app).fg(app.theme().foreground))
            .wrap(Wrap { trim: true }),
        frame.area(),
    );
}
