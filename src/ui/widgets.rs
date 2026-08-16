use crate::{
    app::App,
    output::{format_bytes, format_rate},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline, Wrap},
};

pub fn block(title: &str, app: &App) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme().border))
        .title(Line::styled(
            title.to_owned(),
            Style::default().fg(app.theme().primary).bold(),
        ))
}
pub fn info_panel(
    title: &str,
    rows: Vec<(String, String)>,
    area: Rect,
    frame: &mut Frame,
    app: &App,
) {
    let text = rows
        .into_iter()
        .map(|(k, v)| {
            Line::from(vec![
                Span::styled(
                    format!("{k:<10}"),
                    Style::default().fg(app.theme().secondary),
                ),
                Span::styled(v, Style::default().fg(app.theme().foreground)),
            ])
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(text)
            .block(block(title, app))
            .wrap(Wrap { trim: true }),
        area,
    );
}
pub fn usage_gauge(
    label: &str,
    percent: f64,
    detail: String,
    area: Rect,
    frame: &mut Frame,
    app: &App,
) {
    let value = percent.clamp(0.0, 100.0);
    frame.render_widget(
        Gauge::default()
            .block(block(label, app))
            .gauge_style(
                Style::default()
                    .fg(if value > 90.0 {
                        app.theme().critical
                    } else if value > 75.0 {
                        app.theme().warning
                    } else {
                        app.theme().good
                    })
                    .bg(app.theme().background),
            )
            .ratio(value / 100.0)
            .label(Span::styled(
                format!("{detail}  {value:.0}%"),
                Style::default().fg(app.theme().foreground),
            )),
        area,
    );
}
pub fn graph(
    label: &str,
    values: impl DoubleEndedIterator<Item = f64>,
    max: f64,
    area: Rect,
    frame: &mut Frame,
    app: &App,
    color_index: usize,
) {
    let width = area.width.saturating_sub(2) as usize;
    let data: Vec<u64> = values
        .rev()
        .take(width)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|v| ((v / max.max(1.0)) * 100.0).clamp(0.0, 100.0) as u64)
        .collect();
    frame.render_widget(
        Sparkline::default()
            .block(block(label, app))
            .data(&data)
            .max(100)
            .style(Style::default().fg(app.theme().graph[color_index % 3])),
        area,
    );
}
pub fn live_rows(app: &App) -> Vec<(String, String)> {
    let live = &app.snapshot.live;
    vec![
        (
            "CPU".into(),
            format!(
                "{:.0}%  {} MHz{}",
                live.cpu.usage_percent,
                live.cpu.frequency_mhz,
                live.cpu
                    .temperature_c
                    .map(|t| format!("  {t:.0}°C"))
                    .unwrap_or_default()
            ),
        ),
        (
            "Memory".into(),
            format!(
                "{} / {}",
                format_bytes(live.memory.used_bytes as f64),
                format_bytes(live.memory.total_bytes as f64)
            ),
        ),
        (
            "Disk".into(),
            format!(
                "↓ {}  ↑ {}",
                format_rate(live.disk.read_bytes_per_sec),
                format_rate(live.disk.write_bytes_per_sec)
            ),
        ),
        (
            "Network".into(),
            format!(
                "↓ {}  ↑ {}",
                format_rate(live.network.download_bytes_per_sec),
                format_rate(live.network.upload_bytes_per_sec)
            ),
        ),
        ("Battery".into(), format_battery(app)),
    ]
}
fn format_battery(app: &App) -> String {
    let b = &app.snapshot.live.battery;
    b.percentage
        .map(|p| format!("{p:.0}% {}", b.status))
        .unwrap_or_else(|| b.status.clone())
}
