pub mod builtin;

use ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: &'static str,
    pub background: Color,
    pub foreground: Color,
    pub muted: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub good: Color,
    pub warning: Color,
    pub critical: Color,
    pub border: Color,
    pub graph: [Color; 3],
}

pub fn color_hex(color: Color) -> String {
    match color {
        Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        Color::Black => "#000000".into(),
        Color::White => "#ffffff".into(),
        _ => "#aaaaaa".into(),
    }
}
