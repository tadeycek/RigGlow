use std::collections::VecDeque;

use crate::{
    app::App,
    output::{format_bytes, format_rate},
    theme,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
};
use std::collections::HashMap;

pub fn block(title: &str, app: &App) -> Block<'static> {
    block_line(
        Line::styled(
            title.to_owned(),
            Style::default().fg(app.theme().primary).bold(),
        ),
        app,
    )
}

pub fn block_line(title: Line<'static>, app: &App) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .style(surface_style(app))
        .border_style(Style::default().fg(app.theme().border))
        .title(title)
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

    let group_size = cores.len().div_ceil(4);
    let columns = Layout::horizontal([
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
    ])
    .split(inner);
    for (column, index) in columns.iter().zip(0..4) {
        let offset = index * group_size;
        let end = (offset + group_size).min(cores.len());
        let group = cores.get(offset..end).unwrap_or_default();
        // Derive one common meter width so every percentage column begins at
        // the same offset, even when terminal cell rounding differs by a cell.
        let meter_width = (inner.width / 4).saturating_sub(10).clamp(3, 18) as usize;
        let lines = group
            .iter()
            .enumerate()
            .map(|(row, usage)| {
                let index = row + offset + 1;
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
    disk: Option<&crate::collectors::disks::DiskInfo>,
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
    let meter_width = area.width.saturating_sub(16).clamp(8, 42) as usize;
    let free_percent = 100.0 - percent;
    let device = match disk {
        Some(disk) => format!(
            "{} · {} · {}{}",
            clip(&fs.source, 22),
            disk_kind_label(&disk.kind),
            clip(&normalize_disk_model(&disk.model), 42),
            disk.temperature_c
                .map(|temperature| format!(" · {temperature:.0}°C"))
                .unwrap_or_default(),
        ),
        None => format!("{} · filesystem", clip(&fs.source, 24)),
    };
    let lines = vec![
        Line::from(vec![
            Span::styled("  DEVICE ", Style::default().fg(app.theme().muted)),
            Span::styled(device, Style::default().fg(app.theme().secondary)),
            Span::styled("  FS ", Style::default().fg(app.theme().muted)),
            Span::styled(
                fs.filesystem_type.clone(),
                Style::default().fg(app.theme().foreground),
            ),
            Span::styled("   MOUNT ", Style::default().fg(app.theme().muted)),
            Span::styled(
                clip(&fs.mount_point, 14),
                Style::default().fg(app.theme().foreground),
            ),
        ]),
        Line::from(vec![
            Span::styled("  FS USAGE ", Style::default().fg(app.theme().muted)),
            Span::styled(
                meter(percent, meter_width),
                Style::default().fg(storage_usage_color(free_percent, app)),
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
            Span::styled("   FREE ", Style::default().fg(app.theme().muted)),
            Span::styled(
                format!(
                    "{} ({free_percent:.0}%)",
                    format_bytes(fs.available_bytes as f64)
                ),
                Style::default().fg(storage_usage_color(free_percent, app)),
            ),
        ]),
        Line::from(vec![
            Span::styled("  READ ", Style::default().fg(app.theme().muted)),
            Span::styled(
                rate_or_dash(read_rate),
                rate_style(read_rate, app.theme().graph[0], app),
            ),
            Span::styled("   WRITE ", Style::default().fg(app.theme().muted)),
            Span::styled(
                rate_or_dash(write_rate),
                rate_style(write_rate, app.theme().graph[1], app),
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

pub fn top_activity(app: &App, area: Rect, frame: &mut Frame) {
    let inner_width = area.width.saturating_sub(2) as usize;
    let cpu_width = 6;
    let ram_width = 9;
    let column_gap = 2;
    let name_width = inner_width.saturating_sub(cpu_width + ram_width + (column_gap * 2) + 1);
    let name_counts = app.snapshot.live.top_processes.iter().fold(
        HashMap::<&str, usize>::new(),
        |mut counts, process| {
            *counts.entry(process.name.as_str()).or_default() += 1;
            counts
        },
    );
    let mut lines = vec![Line::from(vec![
        Span::styled(
            format!(" {:<name_width$}", "PROCESS"),
            Style::default().fg(app.theme().muted).bold(),
        ),
        Span::styled(
            format!("{:column_gap$}{:>cpu_width$}", "", "CPU"),
            Style::default().fg(app.theme().muted).bold(),
        ),
        Span::styled(
            format!("{:column_gap$}{:>ram_width$}", "", "RAM"),
            Style::default().fg(app.theme().muted).bold(),
        ),
    ])];
    lines.extend(
        app.snapshot
            .live
            .top_processes
            .iter()
            .map(|process| {
                let process_name =
                    if name_counts.get(process.name.as_str()).copied().unwrap_or(0) > 1 {
                        format!("{} #{}", process.name, process.pid)
                    } else {
                        process.name.clone()
                    };
                Line::from(vec![
                    Span::styled(
                        format!(" {:<name_width$}", clip(&process_name, name_width)),
                        Style::default().fg(app.theme().foreground),
                    ),
                    Span::styled(
                        format!("{:column_gap$}{:>cpu_width$.0}%", "", process.cpu_percent),
                        Style::default().fg(app.theme().graph[0]),
                    ),
                    Span::styled(
                        format!(
                            "{:column_gap$}{:>ram_width$}",
                            "",
                            format_bytes(process.memory_bytes as f64)
                        ),
                        Style::default().fg(app.theme().graph[1]),
                    ),
                ])
            })
            .collect::<Vec<_>>(),
    );
    while lines.len() < 4 {
        lines.push(Line::styled(" —", Style::default().fg(app.theme().muted)));
    }
    frame.render_widget(
        Paragraph::new(lines)
            .block(block(
                &format!(" TOP ACTIVITY · {} ↓ ", app.activity_sort.label()),
                app,
            ))
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
                detail,
                Style::default().fg(app.theme().foreground),
            )),
        area,
    );
}

#[derive(Clone, Copy)]
pub enum HistoryScale {
    Linear,
}

pub struct PairedHistorySpec<'a> {
    pub title: Line<'static>,
    pub upper: &'a VecDeque<f64>,
    pub lower: &'a VecDeque<f64>,
    pub max: f64,
    pub scale: HistoryScale,
    pub upper_label: String,
    pub lower_label: String,
    pub upper_color: Color,
    pub lower_color: Color,
}

/// Renders btop-like history: every stored sample owns exactly one terminal
/// column and is never rescaled or redrawn when later samples arrive.
pub fn paired_history(spec: PairedHistorySpec<'_>, area: Rect, frame: &mut Frame, app: &App) {
    let outer = block_line(spec.title, app);
    let inner = outer.inner(area);
    frame.render_widget(outer, area);
    if inner.width < 12 || inner.height < 4 {
        return;
    }

    let axis_width = spec
        .upper_label
        .len()
        .max(spec.lower_label.len())
        .clamp(10, 18) as u16;
    let columns =
        Layout::horizontal([Constraint::Length(axis_width), Constraint::Min(1)]).split(inner);
    let graph_rows = columns[1].height as usize;
    let width = columns[1].width as usize;
    let zero_row = graph_rows / 2;
    let samples = visible_paired_samples(spec.upper, spec.lower, width);
    let label_lines = (0..graph_rows)
        .map(|row| {
            let label = if row == 0 {
                spec.upper_label.as_str()
            } else if row == zero_row {
                "0"
            } else if row + 1 == graph_rows {
                spec.lower_label.as_str()
            } else {
                ""
            };
            Line::styled(label, Style::default().fg(app.theme().muted))
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(label_lines).style(surface_style(app)),
        columns[0],
    );

    let mut lines = Vec::with_capacity(graph_rows);
    for row in 0..graph_rows {
        let spans = samples
            .iter()
            .map(|sample| {
                if row < zero_row {
                    let dots = dots_from_zero(
                        history_dots(sample.upper, spec.max, zero_row * 4, spec.scale),
                        zero_row - row - 1,
                    );
                    Span::styled(
                        dense_braille_from_bottom(dots).to_string(),
                        Style::default().fg(if sample.upper <= f64::EPSILON {
                            app.theme().muted
                        } else {
                            spec.upper_color
                        }),
                    )
                } else if row == zero_row {
                    // Two filled Braille columns: dotted texture, but no blank
                    // character cells between the data stems and zero rail.
                    Span::styled("⠶", Style::default().fg(app.theme().muted))
                } else {
                    let dots = dots_from_zero(
                        history_dots(
                            sample.lower,
                            spec.max,
                            (graph_rows - zero_row - 1) * 4,
                            spec.scale,
                        ),
                        row - zero_row - 1,
                    );
                    Span::styled(
                        dense_braille_from_top(dots).to_string(),
                        Style::default().fg(if sample.lower <= f64::EPSILON {
                            app.theme().muted
                        } else {
                            spec.lower_color
                        }),
                    )
                }
            })
            .collect::<Vec<_>>();
        lines.push(Line::from(spans));
    }
    frame.render_widget(Paragraph::new(lines).style(surface_style(app)), columns[1]);
}

pub struct SingleHistorySpec<'a> {
    pub title: String,
    pub history: &'a VecDeque<f64>,
    pub max: f64,
    pub upper_label: String,
    pub color: Color,
}

/// A single upward dense-dot history. A zero sample still has one dot, so the
/// history is a continuous bundled trace rather than a sequence of gaps.
pub fn single_history(spec: SingleHistorySpec<'_>, area: Rect, frame: &mut Frame, app: &App) {
    let outer = block(&spec.title, app);
    let inner = outer.inner(area);
    frame.render_widget(outer, area);
    if inner.width < 12 || inner.height < 4 {
        return;
    }

    let axis_width = spec.upper_label.len().clamp(7, 16) as u16;
    let columns =
        Layout::horizontal([Constraint::Length(axis_width), Constraint::Min(1)]).split(inner);
    let graph_rows = columns[1].height as usize;
    let width = columns[1].width as usize;
    let samples = visible_single_samples(spec.history, width);
    let label_lines = (0..graph_rows)
        .map(|row| {
            let label = if row == 0 {
                spec.upper_label.as_str()
            } else if row + 1 == graph_rows {
                "0"
            } else {
                ""
            };
            Line::styled(label, Style::default().fg(app.theme().muted))
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(label_lines).style(surface_style(app)),
        columns[0],
    );

    let mut lines = Vec::with_capacity(graph_rows);
    for row in 0..graph_rows {
        let spans = samples
            .iter()
            .map(|value| {
                let dots = dots_from_zero(
                    history_dots(*value, spec.max, graph_rows * 4, HistoryScale::Linear),
                    graph_rows - row - 1,
                );
                Span::styled(
                    dense_braille_from_bottom(dots).to_string(),
                    Style::default().fg(if *value <= f64::EPSILON {
                        app.theme().muted
                    } else {
                        spec.color
                    }),
                )
            })
            .collect::<Vec<_>>();
        lines.push(Line::from(spans));
    }
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
    let mut parts = gpu
        .usage_percent
        .map(|value| vec![format!("{value:.0}%")])
        .unwrap_or_default();
    if let Some(value) = gpu.temperature_c {
        parts.push(format!("{value:.0}°C"));
    }
    if let Some(value) = gpu.frequency_mhz {
        parts.push(format!("{value} MHz"));
    }
    if let (Some(used), Some(total)) = (gpu.vram_used_bytes, gpu.vram_total_bytes) {
        parts.push(format_vram_pair(used, total));
    } else if let Some(used) = gpu.vram_used_bytes {
        parts.push(format!("VRAM {}", format_bytes(used as f64)));
    }
    if let Some(power) = gpu.power_watts {
        parts.push(format!("{power:.0} W"));
    }
    parts.join(" · ")
}

fn format_vram_pair(used: u64, total: u64) -> String {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    if total as f64 >= GIB {
        format!("{:.1}/{:.1} GiB", used as f64 / GIB, total as f64 / GIB)
    } else {
        format!(
            "{}/{}",
            format_bytes(used as f64),
            format_bytes(total as f64)
        )
    }
}

pub fn battery_summary(app: &App) -> String {
    let battery = &app.snapshot.live.battery;
    battery
        .percentage
        .map(|value| {
            let mut parts = vec![format!("{value:.0}%"), battery.status.clone()];
            if let Some(seconds) = battery.time_remaining_seconds {
                parts.push(format_duration(seconds));
            }
            if let Some(health) = battery.health_percent {
                parts.push(format!("{health:.0}% health"));
            }
            if let Some(power) = battery.power_watts {
                parts.push(format!("{power:.0} W"));
            }
            parts.join(" · ")
        })
        .unwrap_or_else(|| battery.status.clone())
}

fn format_duration(seconds: u64) -> String {
    format!("{}h {:02}m", seconds / 3600, (seconds % 3600) / 60)
}

fn rate_style(rate: f64, color: Color, app: &App) -> Style {
    Style::default().fg(if rate <= f64::EPSILON {
        app.theme().muted
    } else {
        color
    })
}

fn rate_or_dash(rate: f64) -> String {
    if rate <= f64::EPSILON {
        "—".into()
    } else {
        format_rate(rate)
    }
}

fn storage_usage_color(free_percent: f64, app: &App) -> Color {
    if free_percent <= app.settings.storage.critical_free_percent {
        app.theme().critical
    } else if free_percent <= app.settings.storage.warning_free_percent {
        app.theme().warning
    } else {
        app.theme().good
    }
}

fn disk_kind_label(kind: &str) -> &str {
    if kind.eq_ignore_ascii_case("SSD/NVMe") {
        "NVMe"
    } else {
        kind
    }
}

fn normalize_disk_model(model: &str) -> String {
    model.replace("Sandisk", "SanDisk")
}
#[derive(Clone, Copy, Default)]
struct PairedSample {
    upper: f64,
    lower: f64,
}

fn visible_paired_samples(
    upper: &VecDeque<f64>,
    lower: &VecDeque<f64>,
    width: usize,
) -> Vec<PairedSample> {
    let count = upper.len().min(lower.len()).min(width);
    let padding = width.saturating_sub(count);
    let mut columns = vec![PairedSample::default(); padding];
    for (upper, lower) in upper
        .iter()
        .skip(upper.len().saturating_sub(count))
        .zip(lower.iter().skip(lower.len().saturating_sub(count)))
    {
        columns.push(PairedSample {
            upper: *upper,
            lower: *lower,
        });
    }
    columns
}

fn visible_single_samples(history: &VecDeque<f64>, width: usize) -> Vec<f64> {
    let count = history.len().min(width);
    let mut columns = vec![0.0; width.saturating_sub(count)];
    columns.extend(
        history
            .iter()
            .skip(history.len().saturating_sub(count))
            .copied(),
    );
    columns
}

fn history_dots(value: f64, max: f64, max_dots: usize, scale: HistoryScale) -> usize {
    let value = value.max(0.0);
    let max = max.max(1.0);
    let ratio = match scale {
        HistoryScale::Linear => value / max,
    };
    let dots = (ratio * max_dots as f64).round() as usize;
    // Keep a one-dot trace at zero so every history column stays connected to
    // the zero reference instead of disappearing between non-zero towers.
    dots.clamp(1, max_dots.max(1))
}

fn dots_from_zero(level: usize, cell_from_zero: usize) -> usize {
    level.saturating_sub(cell_from_zero * 4).clamp(0, 4)
}

fn dense_braille_from_bottom(dots: usize) -> char {
    match dots {
        0 => ' ',
        1 => '⣀',
        2 => '⣤',
        3 => '⣶',
        _ => '⣿',
    }
}

fn dense_braille_from_top(dots: usize) -> char {
    match dots {
        0 => ' ',
        1 => '⠉',
        2 => '⠛',
        3 => '⠿',
        _ => '⣿',
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
pub fn usage_color(value: f64, app: &App) -> Color {
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

    use super::{HistoryScale, history_dots, visible_paired_samples};

    #[test]
    fn network_columns_shift_left_and_append_on_the_right() {
        let first = visible_paired_samples(
            &VecDeque::from([100.0, 1_000.0]),
            &VecDeque::from([0.0, 0.0]),
            4,
        );
        let second = visible_paired_samples(
            &VecDeque::from([100.0, 1_000.0, 10_000.0]),
            &VecDeque::from([0.0, 0.0, 0.0]),
            4,
        );
        assert_eq!(first[2].upper, second[1].upper);
        assert_eq!(first[3].upper, second[2].upper);
        assert!(second[3].upper > second[2].upper);
    }

    #[test]
    fn zero_network_rate_keeps_a_single_dot() {
        assert_eq!(
            history_dots(0.0, 64.0 * 1024.0 * 1024.0, 16, HistoryScale::Linear),
            1
        );
    }
}
