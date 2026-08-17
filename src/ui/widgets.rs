use std::collections::VecDeque;

use crate::{
    app::{App, HISTORY_LIMIT},
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

pub fn core_grid(title: &str, cores: &[f32], area: Rect, frame: &mut Frame, app: &App) {
    let outer = block(title, app);
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    // Two independent paragraphs deliberately reserve half the card for each
    // column. This avoids the visual drift that padding-based columns caused.
    let columns =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(inner);
    for (column, offset) in columns.iter().zip([0usize, 1]) {
        let meter_width = column.width.saturating_sub(9).clamp(3, 12) as usize;
        let lines = cores
            .iter()
            .skip(offset)
            .step_by(2)
            .enumerate()
            .map(|(row, usage)| {
                let index = row * 2 + offset + 1;
                Line::from(vec![
                    Span::styled(
                        format!("{index:02} "),
                        Style::default().fg(app.theme().secondary),
                    ),
                    Span::styled(
                        meter(*usage as f64, meter_width),
                        Style::default().fg(usage_color(*usage as f64, app)),
                    ),
                    Span::styled(
                        format!(" {usage:>3.0}%"),
                        Style::default().fg(app.theme().foreground),
                    ),
                ])
            })
            .collect::<Vec<_>>();
        frame.render_widget(Paragraph::new(lines).style(surface_style(app)), *column);
    }
}

pub fn filesystem_card(
    filesystem: Option<&crate::collectors::disks::FilesystemInfo>,
    read_rate: f64,
    write_rate: f64,
    area: Rect,
    frame: &mut Frame,
    app: &App,
) {
    let Some(fs) = filesystem else {
        info_panel(
            " DISK ",
            vec![("Status".into(), "No mounted filesystem data".into())],
            area,
            frame,
            app,
        );
        return;
    };
    let percent = if fs.total_bytes == 0 {
        0.0
    } else {
        fs.used_bytes as f64 / fs.total_bytes as f64 * 100.0
    };
    let meter_width = area.width.saturating_sub(16).clamp(8, 32) as usize;
    let lines = vec![
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                format!("{} ", clip(&fs.mount_point, 10)),
                Style::default().fg(app.theme().secondary),
            ),
            Span::styled(
                meter(percent, meter_width),
                Style::default().fg(usage_color(percent, app)),
            ),
            Span::styled(
                format!(" {percent:.0}%"),
                Style::default().fg(app.theme().foreground),
            ),
        ]),
        Line::from(vec![
            Span::styled("  USED ", Style::default().fg(app.theme().muted)),
            Span::styled(
                format!(
                    "{} / {}",
                    format_bytes(fs.used_bytes as f64),
                    format_bytes(fs.total_bytes as f64)
                ),
                Style::default().fg(app.theme().foreground),
            ),
            Span::styled("   READ ", Style::default().fg(app.theme().muted)),
            Span::styled(
                format_rate(read_rate),
                Style::default().fg(app.theme().graph[0]),
            ),
            Span::styled("   WRITE ", Style::default().fg(app.theme().muted)),
            Span::styled(
                format_rate(write_rate),
                Style::default().fg(app.theme().graph[1]),
            ),
        ]),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .block(block(" DISK / I-O ", app))
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
            // Braille uses an 2x4 sub-cell grid. It keeps the original large
            // chart canvas while making small changes in live hardware data
            // read as a detailed line rather than a staircase.
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(app.theme().graph[spec.color_index % 3]))
            .data(&primary_data),
    ];
    let secondary_data = spec.secondary.map(chart_points);
    if let Some(data) = secondary_data.as_ref() {
        datasets.push(
            Dataset::default()
                .name("secondary")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(app.theme().graph[(spec.color_index + 1) % 3]))
                .data(data),
        );
    }
    let x_max = HISTORY_LIMIT.saturating_sub(1) as f64;
    frame.render_widget(
        Chart::new(datasets)
            .block(block(&spec.title, app))
            .style(surface_style(app))
            .x_axis(Axis::default().bounds([0.0, x_max]).labels(vec![
                Span::styled("past", Style::default().fg(app.theme().muted)),
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

pub fn network_history(
    title: String,
    download: &VecDeque<f64>,
    upload: &VecDeque<f64>,
    area: Rect,
    frame: &mut Frame,
    app: &App,
) {
    let outer = block(&title, app);
    let inner = outer.inner(area);
    frame.render_widget(outer, area);
    if inner.width < 12 || inner.height < 4 {
        return;
    }

    let columns = Layout::horizontal([Constraint::Length(10), Constraint::Min(1)]).split(inner);
    let graph_rows = columns[1].height.saturating_sub(1) as usize;
    let width = columns[1].width as usize;
    let zero_row = graph_rows / 2;
    let samples = visible_network_samples(download, upload, width);
    let label_lines = (0..graph_rows)
        .map(|row| {
            let label = if row == 0 {
                "UP 64 MiB"
            } else if row == zero_row {
                "0"
            } else if row + 1 == graph_rows {
                "DN 64 MiB"
            } else {
                ""
            };
            Line::styled(label, Style::default().fg(app.theme().muted))
        })
        .chain(std::iter::once(Line::default()))
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(label_lines).style(surface_style(app)),
        columns[0],
    );

    let mut lines = Vec::with_capacity(graph_rows + 1);
    for row in 0..graph_rows {
        let spans = samples
            .iter()
            .map(|sample| {
                if row < zero_row {
                    let dots = dots_from_zero(
                        network_dots(sample.upload, zero_row * 4),
                        zero_row - row - 1,
                    );
                    Span::styled(
                        block_from_bottom(dots).to_string(),
                        Style::default().fg(app.theme().graph[0]),
                    )
                } else if row == zero_row {
                    Span::styled("─", Style::default().fg(app.theme().muted))
                } else {
                    let dots = dots_from_zero(
                        network_dots(sample.download, (graph_rows - zero_row - 1) * 4),
                        row - zero_row - 1,
                    );
                    Span::styled(
                        block_from_top(dots).to_string(),
                        Style::default().fg(app.theme().graph[1]),
                    )
                }
            })
            .collect::<Vec<_>>();
        lines.push(Line::from(spans));
    }
    lines.push(Line::from(vec![
        Span::styled("past", Style::default().fg(app.theme().muted)),
        Span::styled(
            " ".repeat(width.saturating_sub(7)),
            Style::default().fg(app.theme().muted),
        ),
        Span::styled("now", Style::default().fg(app.theme().muted)),
    ]));
    frame.render_widget(Paragraph::new(lines).style(surface_style(app)), columns[1]);
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
    if let (Some(used), Some(total)) = (gpu.vram_used_bytes, gpu.vram_total_bytes) {
        parts.push(format!(
            "VRAM {} / {}",
            format_bytes(used as f64),
            format_bytes(total as f64)
        ));
    } else if let Some(used) = gpu.vram_used_bytes {
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
    let offset = HISTORY_LIMIT.saturating_sub(history.len()) as f64;
    history
        .iter()
        .enumerate()
        .map(|(index, value)| (offset + index as f64, *value))
        .collect()
}

#[derive(Clone, Copy, Default)]
struct NetworkSample {
    download: f64,
    upload: f64,
}

fn visible_network_samples(
    download: &VecDeque<f64>,
    upload: &VecDeque<f64>,
    width: usize,
) -> Vec<NetworkSample> {
    let count = download.len().min(upload.len()).min(width);
    let padding = width.saturating_sub(count);
    let mut columns = vec![NetworkSample::default(); padding];
    for (down, up) in download
        .iter()
        .skip(download.len().saturating_sub(count))
        .zip(upload.iter().skip(upload.len().saturating_sub(count)))
    {
        columns.push(NetworkSample {
            download: *down,
            upload: *up,
        });
    }
    columns
}

fn network_dots(value: f64, max_dots: usize) -> usize {
    // The fixed logarithmic mapping keeps all retained samples at the same
    // coordinates while still showing low-rate traffic above the zero line.
    const NETWORK_MAX: f64 = 64.0 * 1024.0 * 1024.0;
    let dots = ((value.max(0.0).ln_1p() / NETWORK_MAX.ln_1p()) * max_dots as f64).round() as usize;
    // Keep a one-dot trace at zero so every history column stays connected to
    // the zero reference instead of disappearing between non-zero towers.
    dots.clamp(1, max_dots.max(1))
}

fn dots_from_zero(level: usize, cell_from_zero: usize) -> usize {
    level.saturating_sub(cell_from_zero * 4).clamp(0, 4)
}

fn block_from_bottom(dots: usize) -> char {
    match dots {
        0 => ' ',
        1 => '▂',
        2 => '▄',
        3 => '▆',
        _ => '█',
    }
}

fn block_from_top(dots: usize) -> char {
    match dots {
        0 => ' ',
        1 => '▔',
        2 => '▀',
        3 => '█',
        _ => '█',
    }
}

fn meter(percent: f64, width: usize) -> String {
    let filled = ((percent.clamp(0.0, 100.0) / 100.0) * width as f64).round() as usize;
    format!(
        "{}{}",
        "█".repeat(filled),
        "░".repeat(width.saturating_sub(filled))
    )
}
fn usage_color(value: f64, app: &App) -> Color {
    if value > 90.0 {
        app.theme().critical
    } else if value > 75.0 {
        app.theme().warning
    } else {
        app.theme().good
    }
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

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::{network_dots, visible_network_samples};

    #[test]
    fn network_columns_shift_left_and_append_on_the_right() {
        let first = visible_network_samples(
            &VecDeque::from([100.0, 1_000.0]),
            &VecDeque::from([0.0, 0.0]),
            4,
        );
        let second = visible_network_samples(
            &VecDeque::from([100.0, 1_000.0, 10_000.0]),
            &VecDeque::from([0.0, 0.0, 0.0]),
            4,
        );
        assert_eq!(first[2].download, second[1].download);
        assert_eq!(first[3].download, second[2].download);
        assert!(second[3].download > second[2].download);
    }

    #[test]
    fn zero_network_rate_keeps_a_single_dot() {
        assert_eq!(network_dots(0.0, 16), 1);
    }
}
