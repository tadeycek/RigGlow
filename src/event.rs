use anyhow::Result;
use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;
pub fn poll_key(timeout: Duration) -> Result<Option<KeyEvent>> {
    if event::poll(timeout)?
        && let Event::Key(key) = event::read()?
    {
        return Ok(Some(key));
    }
    Ok(None)
}
