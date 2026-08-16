# RigGlow

RigGlow is a Linux-first live hardware fetcher: Fastfetch-style machine information with a compact, colorful Ratatui dashboard. It deliberately stays out of process-management territory—there is no process table, only the hardware picture and a few lightweight live graphs.

```text
 RIGGLOW  LIVE HARDWARE FETCHER             ◈ synthwave
╭─ IDENTITY ───────────────╮ ╭─ SYSTEM ────────────────────────╮
│       /\      /\         │ │ OS       EndeavourOS            │
│      /  \____/  \        │ │ Host     ASUS Zenbook S16       │
│     /            \       │ │ Kernel   Linux 6.x              │
╰──────────────────────────╯ ╰─────────────────────────────────╯
                             ╭─ LIVE ──────────────────────────╮
                             │ CPU     ██████░░░░  38%          │
                             │ MEMORY  █████░░░░░  18.4 GiB     │
                             │ CPU HISTORY       ▂▄▆█▆▃▅▇       │
                             ╰─────────────────────────────────╯
 [T] Theme  [A] ASCII  [G] Graphs  [S] Snapshot  [Q] Quit
```

## Features

- Linux `/proc` and `/sys` collectors with no root requirement.
- Live CPU, memory, disk-I/O and network rates with compact histories.
- OS, host, kernel, uptime, desktop, terminal, DMI, CPU, GPU, disk, battery, display and network metadata when available.
- Nine built-in themes: Catppuccin Mocha, Dracula, Nord, Gruvbox, Tokyo Night, Synthwave, Matrix, Arch Blue, and Monochrome.
- Fastfetch-style OS logos for 20 popular Linux distributions—Arch, EndeavourOS, Ubuntu, Fedora, Debian, Mint, Manjaro, openSUSE, Pop!_OS, Kali, NixOS, Gentoo, RHEL, Rocky, AlmaLinux, Void, Solus, elementary, Zorin, and MX—plus retro, cat, and RigGlow artwork.
- Responsive Ratatui layout with a safe minimal view for tiny terminals.
- Static, compact SSH, and JSON output modes.

## Build

```bash
cargo build --release
./target/release/rigglow
```

## Run

```bash
rigglow
rigglow --static
rigglow --compact
rigglow --json
rigglow --theme synthwave --ascii cat
rigglow --ascii ~/.config/rigglow/custom.txt --no-animation
rigglow --refresh-rate 500
```

## Keybindings

| Key | Action |
| --- | --- |
| `q`, `Esc`, `Ctrl-C` | Quit safely |
| `t` | Cycle themes |
| `a` | Cycle ASCII art |
| `g` | Toggle graphs |
| `l` | Toggle logo |
| `c` | Toggle compact layout |
| `s` | Leave the alternate screen and print a snapshot |
| `r` | Re-read static hardware data |
| `?` | Toggle help |

## Configuration

Optional configuration: `~/.config/rigglow/config.toml`. CLI values take precedence over this file, which takes precedence over built-in defaults.

```toml
refresh_rate_ms = 1000
theme = "synthwave"
icons = true
graphs = true
animation = true
compact = false

[ascii]
source = "auto"
position = "left"
gradient = true

[modules]
system = true
hardware = true
cpu = true
gpu = true
memory = true
disks = true
network = true
battery = true
display = true
```

Custom ASCII art supports `${primary}`, `${secondary}`, `${accent}`, `${foreground}`, `${muted}`, and `${reset}`. Unknown tokens are preserved as text.

## Current Linux limitations

This MVP reports generic Intel/AMD/NVIDIA GPU information from sysfs; live GPU utilization, VRAM, and temperatures are intentionally optional. Display refresh rate is often unavailable under Wayland, and disk capacity is derived from Linux block devices. No shell command runs on the live refresh path.

The collector traits isolate Linux-specific behavior. Windows and macOS collectors are planned, but not yet implemented.
