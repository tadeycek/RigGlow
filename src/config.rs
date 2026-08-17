use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
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
    pub memory: MemoryConfig,
    pub storage: StorageConfig,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct MemoryConfig {
    pub show_swap: bool,
    pub warning_percent: f64,
    pub critical_percent: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct StorageConfig {
    pub warning_free_percent: f64,
    pub critical_free_percent: f64,
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
    pub memory: MemoryConfig,
    pub storage: StorageConfig,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            refresh_rate_ms: 2000,
            theme: "emerald".into(),
            icons: true,
            graphs: true,
            animation: true,
            compact: false,
            ascii: AsciiConfig::default(),
            modules: ModuleConfig::default(),
            memory: MemoryConfig::default(),
            storage: StorageConfig::default(),
        }
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            show_swap: false,
            warning_percent: 80.0,
            critical_percent: 90.0,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            warning_free_percent: 15.0,
            critical_free_percent: 5.0,
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
        validate_thresholds(&file)?;
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
            memory: file.memory,
            storage: file.storage,
        })
    }
}

fn validate_thresholds(file: &ConfigFile) -> Result<()> {
    let memory = &file.memory;
    let storage = &file.storage;
    if !(0.0..=100.0).contains(&memory.warning_percent)
        || !(0.0..=100.0).contains(&memory.critical_percent)
        || memory.warning_percent >= memory.critical_percent
    {
        bail!(
            "invalid [memory] thresholds: warning_percent must be below critical_percent, both between 0 and 100"
        );
    }
    if !(0.0..=100.0).contains(&storage.warning_free_percent)
        || !(0.0..=100.0).contains(&storage.critical_free_percent)
        || storage.warning_free_percent <= storage.critical_free_percent
    {
        bail!(
            "invalid [storage] thresholds: warning_free_percent must be above critical_free_percent, both between 0 and 100"
        );
    }
    Ok(())
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

/// Saves an interactively selected theme without discarding the user's other
/// RigGlow settings. CLI `--theme` remains temporary unless the user cycles a
/// theme from inside the dashboard.
pub fn persist_theme(theme: &str) -> Result<()> {
    let Some(path) = config_path() else {
        return Ok(());
    };
    persist_theme_at(&path, theme)
}

fn persist_theme_at(path: &Path, theme: &str) -> Result<()> {
    let mut file = if path.exists() {
        let content = fs::read_to_string(path)
            .with_context(|| format!("could not read {}", path.display()))?;
        toml::from_str(&content)
            .with_context(|| format!("invalid configuration in {}", path.display()))?
    } else {
        ConfigFile::default()
    };
    file.theme = theme.to_owned();
    let content = toml::to_string_pretty(&file).context("could not serialize configuration")?;
    let parent = path
        .parent()
        .context("configuration path has no parent directory")?;
    fs::create_dir_all(parent).with_context(|| format!("could not create {}", parent.display()))?;
    let temporary = path.with_extension(format!("toml.{}.tmp", std::process::id()));
    fs::write(&temporary, content)
        .with_context(|| format!("could not write {}", temporary.display()))?;
    fs::rename(&temporary, path).with_context(|| format!("could not save {}", path.display()))
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
            memory: file.memory,
            storage: file.storage,
        };
        assert_eq!(settings.theme, "nord");
        assert_eq!(settings.ascii.source, "cat");
        assert!(!settings.icons && !settings.animation);
        assert_eq!(settings.refresh_rate_ms, 250);
    }

    #[test]
    fn invalid_thresholds_are_explained() {
        let mut file = ConfigFile::default();
        file.storage.warning_free_percent = 4.0;
        assert!(validate_thresholds(&file).is_err());
    }

    #[test]
    fn persisted_theme_preserves_other_configuration() {
        let mut file = ConfigFile::default();
        file.refresh_rate_ms = 750;
        file.ascii.source = "cat".into();
        let root = std::env::temp_dir().join(format!("rigglow-config-{}", std::process::id()));
        let path = root.join("config.toml");
        fs::create_dir_all(&root).expect("test directory");
        fs::write(&path, toml::to_string(&file).expect("test config")).expect("test config write");

        persist_theme_at(&path, "emerald").expect("theme persists");
        let updated: ConfigFile =
            toml::from_str(&fs::read_to_string(&path).expect("read config")).expect("parse config");
        assert_eq!(updated.theme, "emerald");
        assert_eq!(updated.refresh_rate_ms, 750);
        assert_eq!(updated.ascii.source, "cat");

        fs::remove_dir_all(root).expect("remove test directory");
    }
}
