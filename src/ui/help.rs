use crate::app::App;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
pub fn render(frame: &mut Frame, app: &App) {
    let area = centered(64, 18, frame.area());
    frame.render_widget(Clear, area);
    let text = vec![
        Line::styled(
            "RigGlow controls",
            Style::default().fg(app.theme().accent).bold(),
        ),
        Line::from(""),
        Line::from(" [Q] / [Esc]  Quit"),
        Line::from(" [T]          Cycle themes"),
        Line::from(" [A]          Cycle ASCII art"),
        Line::from(" [G]          Toggle graphs"),
        Line::from(" [L]          Toggle logo"),
        Line::from(" [C]          Compact dashboard"),
        Line::from(" [S]          Print static snapshot and exit"),
        Line::from(" [R]          Refresh static hardware information"),
        Line::from(" [?]          Close this help"),
    ];
    frame.render_widget(
        Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(app.theme().border))
                    .title(" HELP "),
            )
            .wrap(Wrap { trim: true }),
        area,
    );
}
fn centered(width: u16, height: u16, area: Rect) -> Rect {
    let horizontal = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Length(width.min(area.width)),
        Constraint::Percentage(50),
    ])
    .split(area);
    Layout::vertical([
        Constraint::Percentage(50),
        Constraint::Length(height.min(area.height)),
        Constraint::Percentage(50),
    ])
    .split(horizontal[1])[1]
}
