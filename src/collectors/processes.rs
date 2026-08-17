//! Lightweight process activity collection for the dashboard's Top Activity card.
//! This intentionally exposes only three rows; RigGlow is not a process manager.

use serde::Serialize;
use sysinfo::{ProcessesToUpdate, System};

#[derive(Debug, Clone, Copy, Serialize, Default, PartialEq, Eq)]
pub enum ActivitySort {
    #[default]
    Cpu,
    Memory,
}

impl ActivitySort {
    pub fn toggle(&mut self) {
        *self = match self {
            Self::Cpu => Self::Memory,
            Self::Memory => Self::Cpu,
        };
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Cpu => "CPU",
            Self::Memory => "RAM",
        }
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct TopProcess {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
}

pub struct LinuxProcessCollector {
    system: System,
}

impl LinuxProcessCollector {
    pub fn new() -> Self {
        Self {
            system: System::new(),
        }
    }

    pub fn top(&mut self, sort: ActivitySort) -> Vec<TopProcess> {
        self.system.refresh_processes(ProcessesToUpdate::All, true);
        let own_pid = std::process::id();
        let mut processes = self
            .system
            .processes()
            .values()
            .filter_map(|process| {
                let name = process.name().to_string_lossy();
                let is_rigglow = name.eq_ignore_ascii_case("rigglow")
                    || name.to_ascii_lowercase().contains("rigglow");
                (!name.is_empty() && process.pid().as_u32() != own_pid && !is_rigglow).then(|| {
                    TopProcess {
                        pid: process.pid().as_u32(),
                        name: display_name(&name, process.exe().map(|path| path.to_string_lossy())),
                        cpu_percent: process.cpu_usage(),
                        memory_bytes: process.memory(),
                    }
                })
            })
            .collect::<Vec<_>>();
        processes.sort_by(|left, right| match sort {
            ActivitySort::Cpu => right
                .cpu_percent
                .total_cmp(&left.cpu_percent)
                .then_with(|| right.memory_bytes.cmp(&left.memory_bytes)),
            ActivitySort::Memory => right
                .memory_bytes
                .cmp(&left.memory_bytes)
                .then_with(|| right.cpu_percent.total_cmp(&left.cpu_percent)),
        });
        processes.truncate(3);
        processes
    }
}

fn display_name(name: &str, executable: Option<std::borrow::Cow<'_, str>>) -> String {
    let executable_is_firefox = executable
        .as_deref()
        .is_some_and(|path| path.contains("firefox"));
    if executable_is_firefox && name.starts_with("Isolated Web") {
        "Firefox (Web)".into()
    } else if executable_is_firefox && name.starts_with("WebExtensions") {
        "Firefox (Ext)".into()
    } else {
        name.into()
    }
}

#[cfg(test)]
mod tests {
    use super::display_name;

    #[test]
    fn firefox_child_processes_get_a_clear_alias() {
        assert_eq!(
            display_name(
                "Isolated Web Content",
                Some("/usr/lib/firefox/firefox".into())
            ),
            "Firefox (Web)"
        );
    }
}
