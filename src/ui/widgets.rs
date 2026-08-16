use std::collections::VecDeque;

use crate::{
    app::App,
    output::{format_bytes, format_rate},
    theme,
};
use ratatui::{
    prelude::*,
    symbols,
    widgets::{Axis, Block, Borders, Chart, Dataset, Gauge, GraphType, Paragraph, Wrap},
};

pub fn block(title: &str, app: &App) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .style(surface_style(app))
        .border_style(Style::default().fg(app.theme().border))
        .title(Line::styled(
            title.to_owned(),
            Style::default().fg(app.theme().primary).bold(),
        ))
}

pub fn surface_style(app: &App) -> Style {
    Style::default().bg(theme::surface(app.theme().background))
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
        .map(|(key, value)| {
            Line::from(vec![
                Span::styled(
                    format!("{key:<10}"),
                    Style::default().fg(app.theme().secondary),
                ),
                Span::styled(value, Style::default().fg(app.theme().foreground)),
            ])
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(text)
            .block(block(title, app))
            .style(surface_style(app))
            .wrap(Wrap { trim: true }),
        area,
    );
}

pub fn detail_grid(
    title: &str,
    rows: Vec<(String, String)>,
    area: Rect,
    frame: &mut Frame,
    app: &App,
) {
    let half = area.width.saturating_sub(4) as usize / 2;
    let mut lines = Vec::new();
    for pair in rows.chunks(2) {
        let cell = |(key, value): &(String, String)| {
            let value_width = half.saturating_sub(key.len() + 2);
            let clipped = clip(value, value_width);
            format!("{key}: {clipped}")
        };
        let left = cell(&pair[0]);
        let right = pair.get(1).map(cell).unwrap_or_default();
        lines.push(Line::from(vec![
            Span::styled(
                format!("{left:<half$}"),
                Style::default().fg(app.theme().foreground),
            ),
            Span::styled(right, Style::default().fg(app.theme().foreground)),
        ]));
    }
    frame.render_widget(
        Paragraph::new(lines)
            .block(block(title, app))
            .style(surface_style(app))
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
    let color = if value > 90.0 {
        app.theme().critical
    } else if value > 75.0 {
        app.theme().warning
    } else {
        app.theme().good
    };
    frame.render_widget(
        Gauge::default()
            .block(block(label, app))
            .gauge_style(
                Style::default()
                    .fg(color)
                    .bg(theme::surface(app.theme().background)),
            )
            .ratio(value / 100.0)
            .label(Span::styled(
                format!("{detail}  {value:.0}%"),
                Style::default().fg(app.theme().foreground),
            )),
        area,
    );
}

pub struct LineChartSpec<'a> {
    pub title: String,
    pub primary: &'a VecDeque<f64>,
    pub secondary: Option<&'a VecDeque<f64>>,
    pub max: f64,
    pub middle_label: String,
    pub upper_label: String,
    pub color_index: usize,
}

pub fn line_chart(spec: LineChartSpec<'_>, area: Rect, frame: &mut Frame, app: &App) {
    let primary_data = chart_points(spec.primary);
    let mut datasets = vec![
        Dataset::default()
            .name("primary")
            .marker(symbols::Marker::HalfBlock)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(app.theme().graph[spec.color_index % 3]))
            .data(&primary_data),
    ];
    let secondary_data = spec.secondary.map(chart_points);
    if let Some(data) = secondary_data.as_ref() {
        datasets.push(
            Dataset::default()
                .name("secondary")
                .marker(symbols::Marker::HalfBlock)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(app.theme().graph[(spec.color_index + 1) % 3]))
                .data(data),
        );
    }
    let x_max = spec.primary.len().max(2) as f64 - 1.0;
    frame.render_widget(
        Chart::new(datasets)
            .block(block(&spec.title, app))
            .style(surface_style(app))
            .x_axis(Axis::default().bounds([0.0, x_max]).labels(vec![
                Span::styled("older", Style::default().fg(app.theme().muted)),
                Span::styled("now", Style::default().fg(app.theme().muted)),
            ]))
            .y_axis(
                Axis::default()
                    .bounds([0.0, spec.max.max(1.0)])
                    .labels(vec![
                        Span::styled("0", Style::default().fg(app.theme().muted)),
                        Span::styled(spec.middle_label, Style::default().fg(app.theme().muted)),
                        Span::styled(spec.upper_label, Style::default().fg(app.theme().muted)),
                    ]),
            ),
        area,
    );
}

pub fn live_rows(app: &App) -> Vec<(String, String)> {
    let live = &app.snapshot.live;
    vec![
        (
            "CPU".into(),
            format!(
                "{:.0}% · {} MHz{}",
                live.cpu.usage_percent,
                live.cpu.frequency_mhz,
                live.cpu
                    .temperature_c
                    .map(|value| format!(" · {value:.0}°C"))
                    .unwrap_or_default()
            ),
        ),
        ("GPU".into(), gpu_summary(app)),
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
                "↓ {} · ↑ {}",
                format_rate(live.disk.read_bytes_per_sec),
                format_rate(live.disk.write_bytes_per_sec)
            ),
        ),
        (
            "Network".into(),
            format!(
                "↓ {} · ↑ {}",
                format_rate(live.network.download_bytes_per_sec),
                format_rate(live.network.upload_bytes_per_sec)
            ),
        ),
        ("Battery".into(), battery_summary(app)),
    ]
}

pub fn gpu_summary(app: &App) -> String {
    let gpu = &app.snapshot.live.gpu;
    let mut parts = vec![
        gpu.usage_percent
            .map(|value| format!("{value:.0}%"))
            .unwrap_or_else(|| "No usage sensor".into()),
    ];
    if let Some(value) = gpu.temperature_c {
        parts.push(format!("{value:.0}°C"));
    }
    if let Some(value) = gpu.frequency_mhz {
        parts.push(format!("{value} MHz"));
    }
    if let Some(used) = gpu.vram_used_bytes {
        parts.push(format!("VRAM {}", format_bytes(used as f64)));
    }
    parts.join(" · ")
}

pub fn battery_summary(app: &App) -> String {
    let battery = &app.snapshot.live.battery;
    battery
        .percentage
        .map(|value| format!("{value:.0}% {}", battery.status))
        .unwrap_or_else(|| battery.status.clone())
}
fn chart_points(history: &VecDeque<f64>) -> Vec<(f64, f64)> {
    history
        .iter()
        .enumerate()
        .map(|(index, value)| (index as f64, *value))
        .collect()
}
fn clip(value: &str, width: usize) -> String {
    if width <= 1 {
        return String::new();
    }
    if value.chars().count() > width {
        format!("{}…", value.chars().take(width - 1).collect::<String>())
    } else {
        value.into()
    }
}
