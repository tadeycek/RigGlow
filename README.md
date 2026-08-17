# RigGlow

RigGlow is a Linux-first live hardware fetcher: Fastfetch-style machine information with a compact, colorful Ratatui dashboard. It deliberately stays out of process-management territory—there is no process table, only the hardware picture and a few lightweight live graphs.

```text
 RIGGLOW  LIVE HARDWARE FETCHER             ◈ emerald
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
- Live CPU, GPU, memory, disk-I/O and network rates with labelled CPU/RAM, GPU, and download/upload history charts.
- OS, host, kernel, uptime, desktop, terminal, DMI, CPU, GPU, disk, battery, display and network metadata when available.
- Seventeen built-in themes: Terminal (transparent/inherited), Emerald, Catppuccin Mocha, Dracula, Nord, Gruvbox, Tokyo Night, Synthwave, Matrix, Arch Blue, Monochrome, Rose Pine, One Dark, Kanagawa, Everforest, Ayu Dark, and Solarized Dark. Named themes use opaque dashboard surfaces.
- Fastfetch-style OS logos for 20 popular Linux distributions—Arch, EndeavourOS, Ubuntu, Fedora, Debian, Mint, Manjaro, openSUSE, Pop!_OS, Kali, NixOS, Gentoo, RHEL, Rocky, AlmaLinux, Void, Solus, elementary, Zorin, and MX—plus retro, cat, and RigGlow artwork.
- Responsive Ratatui layout with a safe minimal view for tiny terminals.
- Static, compact SSH, and JSON output modes.

Top Activity is deliberately a three-process overview, not a process manager. Its CPU percentage is per logical thread: `100%` means one fully occupied logical CPU, so a multithreaded process can exceed `100%`.

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
| `p` | Toggle Top Activity sorting between CPU and RAM |
| `l` | Toggle logo |
| `c` | Toggle compact layout |
| `s` | Leave the alternate screen and print a snapshot |
| `r` | Re-read static hardware data |
| `?` | Toggle help |

## Configuration

Optional configuration: `~/.config/rigglow/config.toml`. CLI values take precedence over this file, which takes precedence over built-in defaults.

```toml
refresh_rate_ms = 2000
theme = "emerald"
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

[memory]
# Show swap even when unused; memory bars turn yellow/red at these levels.
show_swap = false
warning_percent = 80
critical_percent = 90

[storage]
# Warn when free capacity drops below these percentages.
warning_free_percent = 15
critical_free_percent = 5
```

Custom ASCII art supports `${primary}`, `${secondary}`, `${accent}`, `${foreground}`, `${muted}`, and `${reset}`. Unknown tokens are preserved as text.

## Current Linux limitations

GPU activity, VRAM, power and temperatures depend on the driver and are optional. RigGlow asks KDE's `kscreen-doctor` for Wayland refresh rates when available, with `/sys` as a fallback. Disk capacity distinguishes physical-drive capacity from the mounted filesystem capacity. Network latency is a lightweight periodic ping of the active default gateway; it is optional and never requires root.

The collector traits isolate Linux-specific behavior. Windows and macOS collectors are planned, but not yet implemented.
