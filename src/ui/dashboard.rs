use super::{help, widgets};
use crate::{app::App, ascii, ascii::builtin::distro_name, collectors::system::uptime_human};
use ratatui::{
    prelude::*,
    widgets::{Paragraph, Wrap},
};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    frame.render_widget(
        ratatui::widgets::Block::default().style(Style::default().bg(app.theme().background)),
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
        Span::styled(
            "  LIVE HARDWARE FETCHER",
            Style::default().fg(app.theme().muted),
        ),
        Span::styled(
            format!("  ◈ {}", app.theme().name),
            Style::default().fg(app.theme().secondary),
        ),
    ]);
    frame.render_widget(Paragraph::new(title), area);
}
fn status(frame: &mut Frame, app: &App, area: Rect) {
    let text = if app.show_help {
        " [?] Close help"
    } else {
        " [T] Theme  [A] ASCII  [G] Graphs  [L] Logo  [C] Compact  [S] Snapshot  [R] Refresh  [Q] Quit  [?] Help"
    };
    frame.render_widget(
        Paragraph::new(Line::styled(text, Style::default().fg(app.theme().muted))),
        area,
    );
}
fn large(frame: &mut Frame, app: &App, area: Rect) {
    let columns =
        Layout::horizontal([Constraint::Percentage(37), Constraint::Percentage(63)]).split(area);
    if app.show_logo {
        let lines = ascii::load(&app.settings.ascii.source, &distro_name(), app.theme())
            .unwrap_or_default();
        frame.render_widget(
            Paragraph::new(lines)
                .block(widgets::block(" IDENTITY ", app))
                .wrap(Wrap { trim: false }),
            columns[0],
        );
    }
    let rows = Layout::vertical([
        Constraint::Percentage(39),
        Constraint::Percentage(29),
        Constraint::Percentage(32),
    ])
    .split(columns[1]);
    system_panel(frame, app, rows[0]);
    hardware_panel(frame, app, rows[1]);
    live_panel(frame, app, rows[2]);
}
fn small(frame: &mut Frame, app: &App, area: Rect) {
    let mut constraints = Vec::new();
    if app.show_logo && area.height >= 17 {
        constraints.push(Constraint::Length(8));
    }
    constraints.push(Constraint::Percentage(55));
    constraints.push(Constraint::Percentage(45));
    let chunks = Layout::vertical(constraints).split(area);
    let mut at = 0;
    if app.show_logo && area.height >= 17 {
        let lines = ascii::load(&app.settings.ascii.source, &distro_name(), app.theme())
            .unwrap_or_default();
        frame.render_widget(
            Paragraph::new(lines).block(widgets::block(" RIGGLOW ", app)),
            chunks[0],
        );
        at = 1;
    }
    system_panel(frame, app, chunks[at]);
    if chunks.len() > at + 1 {
        if app.show_graphs && chunks[at + 1].height >= 8 {
            live_panel(frame, app, chunks[at + 1]);
        } else {
            widgets::info_panel(
                " LIVE ",
                widgets::live_rows(app),
                chunks[at + 1],
                frame,
                app,
            );
        }
    }
}
fn system_panel(frame: &mut Frame, app: &App, area: Rect) {
    let s = &app.snapshot.static_info;
    widgets::info_panel(
        " SYSTEM ",
        vec![
            ("OS".into(), s.system.os.clone()),
            ("Host".into(), s.system.hostname.clone()),
            ("Kernel".into(), s.system.kernel.clone()),
            ("Uptime".into(), uptime_human(s.system.uptime_seconds)),
            ("Desktop".into(), s.system.desktop.clone()),
            ("Shell".into(), s.system.shell.clone()),
        ],
        area,
        frame,
        app,
    );
}
fn hardware_panel(frame: &mut Frame, app: &App, area: Rect) {
    let s = &app.snapshot.static_info;
    let mut rows = Vec::new();
    if app.settings.modules.cpu {
        rows.push(("CPU".into(), s.hardware.cpu_model.clone()));
        rows.push((
            "Cores".into(),
            format!(
                "{} physical / {} logical",
                s.hardware.physical_cpus, s.hardware.logical_cpus
            ),
        ));
    }
    if app.settings.modules.gpu {
        rows.push(("GPU".into(), s.gpu.model.clone()));
    }
    if app.settings.modules.disks {
        rows.push((
            "Disk".into(),
            s.disks
                .first()
                .map(|d| d.model.clone())
                .unwrap_or_else(|| "Unknown".into()),
        ));
    }
    if app.settings.modules.display {
        rows.push(("Display".into(), s.display.resolution.clone()));
    }
    widgets::info_panel(" HARDWARE ", rows, area, frame, app);
}
fn live_panel(frame: &mut Frame, app: &App, area: Rect) {
    if !app.show_graphs || area.height < 10 {
        widgets::info_panel(" LIVE ", widgets::live_rows(app), area, frame, app);
        return;
    }
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(3),
    ])
    .split(area);
    let live = &app.snapshot.live;
    widgets::usage_gauge(
        " CPU ",
        live.cpu.usage_percent as f64,
        format!("{} MHz", live.cpu.frequency_mhz),
        chunks[0],
        frame,
        app,
    );
    widgets::usage_gauge(
        " MEMORY ",
        live.memory.percent(),
        format!(
            "{} / {}",
            crate::output::format_bytes(live.memory.used_bytes as f64),
            crate::output::format_bytes(live.memory.total_bytes as f64)
        ),
        chunks[1],
        frame,
        app,
    );
    let graphs = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);
    widgets::graph(
        " CPU HISTORY ",
        app.cpu_history.iter().copied(),
        100.0,
        graphs[0],
        frame,
        app,
        0,
    );
    widgets::graph(
        " NETWORK ",
        app.network_history.iter().copied(),
        app.network_history.iter().copied().fold(1.0, f64::max),
        graphs[1],
        frame,
        app,
        2,
    );
}
fn fallback(frame: &mut Frame, app: &App) {
    let live = &app.snapshot.live;
    let text = format!(
        " RIGGLOW\n\n CPU {:.0}% · RAM {:.0}%\n\nTerminal too small — resize for dashboard\n[Q] Quit",
        live.cpu.usage_percent,
        live.memory.percent()
    );
    frame.render_widget(
        Paragraph::new(text)
            .style(Style::default().fg(app.theme().foreground))
            .wrap(Wrap { trim: true }),
        frame.area(),
    );
}
