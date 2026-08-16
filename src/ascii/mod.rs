pub mod builtin;
pub mod parser;

use crate::theme::Theme;
use anyhow::Result;
use parser::{TokenColor, parse};
use ratatui::{
    style::Style,
    text::{Line, Span},
};
use std::{fs, path::Path};

pub fn load(source: &str, distro: &str, theme: &Theme) -> Result<Vec<Line<'static>>> {
    let raw = if source == "auto" {
        builtin::auto(distro)
    } else if let Some(art) = builtin::named(source) {
        art
    } else if Path::new(source).exists() {
        fs::read_to_string(source)?
    } else {
        builtin::named("rigglow").unwrap_or_default()
    };
    let lines = parse(&raw)
        .into_iter()
        .map(|parts| {
            let spans = parts
                .into_iter()
                .map(|part| {
                    let color = match part.color {
                        TokenColor::Primary => theme.primary,
                        TokenColor::Secondary => theme.secondary,
                        TokenColor::Accent => theme.accent,
                        TokenColor::Foreground | TokenColor::Reset => theme.foreground,
                        TokenColor::Muted => theme.muted,
                    };
                    Span::styled(part.text, Style::default().fg(color))
                })
                .collect::<Vec<_>>();
            Line::from(spans)
        })
        .collect();
    Ok(lines)
}
