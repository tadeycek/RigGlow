use crate::collectors::Snapshot;
use anyhow::Result;
pub fn render(snapshot: &Snapshot) -> Result<String> {
    Ok(serde_json::to_string_pretty(snapshot)?)
}
