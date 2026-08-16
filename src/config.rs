use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::cli::Cli;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ConfigFile {
    pub refresh_rate_ms: u64,
    pub theme: String,
    pub icons: bool,
    pub graphs: bool,
    pub animation: bool,
    pub compact: bool,
    pub ascii: AsciiConfig,
    pub modules: ModuleConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct AsciiConfig {
    pub source: String,
    pub position: String,
    pub gradient: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ModuleConfig {
    pub system: bool,
    pub hardware: bool,
    pub cpu: bool,
    pub gpu: bool,
    pub memory: bool,
    pub disks: bool,
    pub network: bool,
    pub battery: bool,
    pub display: bool,
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub refresh_rate_ms: u64,
    pub theme: String,
    pub icons: bool,
    pub graphs: bool,
    pub animation: bool,
    pub compact: bool,
    pub ascii: AsciiConfig,
    pub modules: ModuleConfig,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            refresh_rate_ms: 1000,
            theme: "catppuccin-mocha".into(),
            icons: true,
            graphs: true,
            animation: true,
            compact: false,
            ascii: AsciiConfig::default(),
            modules: ModuleConfig::default(),
        }
    }
}

impl Default for AsciiConfig {
    fn default() -> Self {
        Self {
            source: "auto".into(),
            position: "left".into(),
            gradient: true,
        }
    }
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            system: true,
            hardware: true,
            cpu: true,
            gpu: true,
            memory: true,
            disks: true,
            network: true,
            battery: true,
            display: true,
        }
    }
}

impl Settings {
    pub fn load(cli: &Cli) -> Result<Self> {
        let file = load_config()?;
        let refresh_rate_ms = cli.refresh_rate.unwrap_or(file.refresh_rate_ms).max(100);
        Ok(Self {
            refresh_rate_ms,
            theme: cli.theme.clone().unwrap_or(file.theme),
            icons: if cli.no_icons { false } else { file.icons },
            graphs: file.graphs,
            animation: if cli.no_animation {
                false
            } else {
                file.animation
            },
            compact: cli.compact || file.compact,
            ascii: AsciiConfig {
                source: cli.ascii.clone().unwrap_or(file.ascii.source),
                ..file.ascii
            },
            modules: file.modules,
        })
    }
}

fn config_path() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|dir| dir.join("rigglow/config.toml"))
}

fn load_config() -> Result<ConfigFile> {
    let Some(path) = config_path() else {
        return Ok(ConfigFile::default());
    };
    if !path.exists() {
        return Ok(ConfigFile::default());
    }
    let content =
        fs::read_to_string(&path).with_context(|| format!("could not read {}", path.display()))?;
    toml::from_str(&content).with_context(|| format!("invalid configuration in {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_values_override_file_values() {
        let cli = Cli {
            r#static: false,
            compact: false,
            json: false,
            theme: Some("nord".into()),
            ascii: Some("cat".into()),
            no_icons: true,
            no_animation: true,
            refresh_rate: Some(250),
        };
        let file = ConfigFile {
            theme: "dracula".into(),
            icons: true,
            animation: true,
            refresh_rate_ms: 1000,
            ..ConfigFile::default()
        };
        let settings = Settings {
            refresh_rate_ms: cli.refresh_rate.unwrap_or(file.refresh_rate_ms),
            theme: cli.theme.unwrap_or(file.theme),
            icons: if cli.no_icons { false } else { file.icons },
            graphs: file.graphs,
            animation: if cli.no_animation {
                false
            } else {
                file.animation
            },
            compact: cli.compact || file.compact,
            ascii: AsciiConfig {
                source: cli.ascii.unwrap_or(file.ascii.source),
                ..file.ascii
            },
            modules: file.modules,
        };
        assert_eq!(settings.theme, "nord");
        assert_eq!(settings.ascii.source, "cat");
        assert!(!settings.icons && !settings.animation);
        assert_eq!(settings.refresh_rate_ms, 250);
    }
}
