mod app;
mod ascii;
mod cli;
mod collectors;
mod config;
mod event;
mod output;
mod theme;
mod ui;

use std::{
    io,
    time::{Duration, Instant},
};

use anyhow::{Result, bail};
use clap::Parser;
use crossterm::{
    event::{KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{app::App, cli::Cli, config::Settings};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let settings = Settings::load(&cli)?;
    if theme::builtin::get(&settings.theme).is_none() {
        bail!(
            "unknown theme '{}'; available: {}",
            settings.theme,
            theme::builtin::all()
                .iter()
                .map(|t| t.name)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    let mut app = App::new(settings);
    if cli.json {
        println!("{}", output::json::render(&app.snapshot)?);
        return Ok(());
    }
    if cli.r#static {
        print!(
            "{}",
            output::static_view::render(
                &app.snapshot,
                &app.settings.ascii.source,
                app.theme(),
                app.settings.icons
            )
        );
        return Ok(());
    }
    if cli.compact {
        print!("{}", output::compact::render(&app.snapshot));
        return Ok(());
    }
    let snapshot = run_tui(&mut app)?;
    if app.theme_changed() {
        config::persist_theme(&app.settings.theme)?;
    }
    if snapshot {
        print!(
            "{}",
            output::static_view::render(
                &app.snapshot,
                &app.settings.ascii.source,
                app.theme(),
                app.settings.icons
            )
        );
    }
    Ok(())
}

fn run_tui(app: &mut App) -> Result<bool> {
    app.prime_history();
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = event_loop(&mut terminal, app);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<bool> {
    let mut next_refresh = Instant::now() + Duration::from_millis(app.settings.refresh_rate_ms);
    loop {
        terminal.draw(|frame| ui::dashboard::render(frame, app))?;
        let now = Instant::now();
        if now >= next_refresh {
            app.refresh_live();
            next_refresh = now + Duration::from_millis(app.settings.refresh_rate_ms);
        }
        let timeout = next_refresh
            .saturating_duration_since(Instant::now())
            .min(Duration::from_millis(100));
        if let Some(key) = event::poll_key(timeout)? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(false),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(false);
                }
                KeyCode::Char('t') => app.cycle_theme(),
                KeyCode::Char('a') => app.cycle_ascii(),
                KeyCode::Char('g') => app.show_graphs = !app.show_graphs,
                KeyCode::Char('l') => app.show_logo = !app.show_logo,
                KeyCode::Char('c') => app.compact = !app.compact,
                KeyCode::Char('s') => return Ok(true),
                KeyCode::Char('r') => app.refresh_static(),
                KeyCode::Char('p') => app.toggle_activity_sort(),
                KeyCode::Char('?') => app.show_help = !app.show_help,
                _ => {}
            }
        }
    }
}
