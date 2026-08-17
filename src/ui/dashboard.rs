use super::{help, widgets};
use crate::{
    app::App,
    ascii,
    ascii::builtin::distro_name,
    collectors::system::uptime_human,
    output::{format_bytes, format_rate},
};
use ratatui::{
    prelude::*,
    widgets::{Paragraph, Wrap},
};

const NETWORK_GRAPH_MAX: f64 = 64.0 * 1024.0 * 1024.0;

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
        " [T] Theme  [A] ASCII  [G] Graphs  [L] Logo  [C] Compact  [S] Snapshot  [R] Refresh  [Q] Quit  [?] Help"
    };
    frame.render_widget(
        Paragraph::new(Line::styled(text, Style::default().fg(app.theme().muted)))
            .style(widgets::surface_style(app)),
        area,
    );
}

fn large(frame: &mut Frame, app: &App, area: Rect) {
    let columns =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).split(area);
    let left = Layout::vertical([Constraint::Percentage(65), Constraint::Percentage(35)])
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
    gpu_monitor(frame, app, left[1]);

    let right = Layout::vertical([
        Constraint::Percentage(31),
        Constraint::Percentage(35),
        Constraint::Percentage(34),
    ])
    .split(columns[1]);
    system_panel(frame, app, right[0]);
    hardware_panel(frame, app, right[1]);
    performance_panel(frame, app, right[2]);
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
    if area.height >= 11 && app.settings.modules.cpu {
        let rows = Layout::vertical([Constraint::Length(6), Constraint::Min(3)]).split(area);
        system_grid(frame, app, rows[0]);
        widgets::core_grid(
            " CPU CORES ",
            &app.snapshot.live.cpu.per_core_percent,
            rows[1],
            frame,
            app,
        );
    } else {
        system_grid(frame, app, area);
    }
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
        .map(|d| format!("{} · {}", d.kind, format_bytes(d.capacity_bytes as f64)))
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
                "{}P / {}L",
                s.hardware.physical_cpus, s.hardware.logical_cpus
            ),
        ));
    }
    if modules.memory {
        rows.push((
            "Memory".into(),
            format_bytes(s.hardware.total_memory_bytes as f64),
        ));
    }
    if modules.gpu {
        rows.push(("GPU".into(), s.gpu.model.clone()));
    }
    if modules.disks {
        rows.push(("Storage".into(), disk));
    }
    if modules.display {
        rows.push((
            "Display".into(),
            format!("{} @ {}", s.display.resolution, s.display.refresh_rate),
        ));
    }
    if modules.battery {
        rows.push(("Battery".into(), widgets::battery_summary(app)));
    }
    if area.height >= 12 && app.settings.modules.disks {
        let panels = Layout::vertical([Constraint::Length(7), Constraint::Min(3)]).split(area);
        widgets::detail_grid(" HARDWARE ", rows, panels[0], frame, app);
        widgets::filesystem_card(
            app.snapshot.static_info.filesystems.first(),
            app.snapshot.live.disk.read_bytes_per_sec,
            app.snapshot.live.disk.write_bytes_per_sec,
            panels[1],
            frame,
            app,
        );
    } else {
        widgets::detail_grid(" HARDWARE ", rows, area, frame, app);
    }
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

fn gpu_monitor(frame: &mut Frame, app: &App, area: Rect) {
    let live = &app.snapshot.live.gpu;
    if !app.show_graphs || area.height < 8 {
        widgets::info_panel(
            " GPU MONITOR ",
            vec![
                ("Model".into(), app.snapshot.static_info.gpu.model.clone()),
                ("Live".into(), widgets::gpu_summary(app)),
            ],
            area,
            frame,
            app,
        );
        return;
    }
    let rows = Layout::vertical([Constraint::Length(3), Constraint::Min(4)]).split(area);
    let details = format!(
        "{}{}{}",
        live.frequency_mhz
            .map(|value| format!("{value} MHz"))
            .unwrap_or_else(|| "Sensor optional".into()),
        live.temperature_c
            .map(|value| format!(" · {value:.0}°C"))
            .unwrap_or_default(),
        match (live.vram_used_bytes, live.vram_total_bytes) {
            (Some(used), Some(total)) => format!(
                " · VRAM {} / {}",
                format_bytes(used as f64),
                format_bytes(total as f64)
            ),
            (Some(used), None) => format!(" · VRAM {}", format_bytes(used as f64)),
            _ => String::new(),
        }
    );
    widgets::usage_gauge(
        " GPU LOAD ",
        live.usage_percent.unwrap_or(0.0) as f64,
        details,
        rows[0],
        frame,
        app,
    );
    widgets::line_chart(
        widgets::LineChartSpec {
            title: format!(" GPU HISTORY  {} ", widgets::gpu_summary(app)),
            primary: &app.gpu_history,
            secondary: None,
            max: 100.0,
            middle_label: "50%".into(),
            upper_label: "100%".into(),
            color_index: 1,
            scale: widgets::VerticalScale::Linear,
        },
        rows[1],
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
    let cpu = &app.snapshot.live.cpu;
    widgets::line_chart(
        widgets::LineChartSpec {
            title: format!(
                " CPU + RAM  {:.0}% CPU · {:.0}% RAM · {} MHz{} ",
                cpu.usage_percent,
                app.snapshot.live.memory.percent(),
                cpu.frequency_mhz,
                cpu.temperature_c
                    .map(|value| format!(" · {value:.0}°C"))
                    .unwrap_or_default()
            ),
            primary: &app.cpu_history,
            secondary: Some(&app.memory_history),
            max: 100.0,
            middle_label: "50%".into(),
            upper_label: "100%".into(),
            color_index: 0,
            scale: widgets::VerticalScale::Linear,
        },
        charts[0],
        frame,
        app,
    );
    let net = &app.snapshot.live.network;
    widgets::line_chart(
        widgets::LineChartSpec {
            title: format!(
                " NETWORK  ↓ {} · ↑ {} ",
                format_rate(net.download_bytes_per_sec),
                format_rate(net.upload_bytes_per_sec)
            ),
            primary: &app.network_down_history,
            secondary: Some(&app.network_up_history),
            max: NETWORK_GRAPH_MAX,
            middle_label: "8 KiB/s".into(),
            upper_label: "64 MiB/s".into(),
            color_index: 2,
            scale: widgets::VerticalScale::Logarithmic,
        },
        charts[1],
        frame,
        app,
    );
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
