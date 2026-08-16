use std::fs;

pub fn auto(distro: &str) -> String {
    let distro = distro.to_ascii_lowercase();
    if distro.contains("endeavour") {
        named("endeavouros")
    } else if distro.contains("arch") {
        named("arch")
    } else if distro.contains("ubuntu") {
        named("ubuntu")
    } else if distro.contains("fedora") {
        named("fedora")
    } else if distro.contains("debian") {
        named("debian")
    } else if distro.contains("mint") {
        named("mint")
    } else {
        named("linux")
    }
    .unwrap_or_default()
}

pub fn named(name: &str) -> Option<String> {
    let art = match name.to_ascii_lowercase().as_str() {
        "arch" | "archlinux" => {
            "${primary}        ▲\n${secondary}       ▲▲▲\n${accent}      ▲   ▲\n${primary}     ▲▲▲ ▲▲▲"
        }
        "endeavouros" | "endeavour" => {
            "${primary}      ◢█◣ ◢█◣\n${secondary}     ◢███████◣\n${accent}    ◢██◤   ◥██◣\n${primary}   ◢██◤     ◥██◣"
        }
        "ubuntu" => {
            "${accent}      ●●●●●\n${primary}   ●●       ●●\n${secondary}  ●●   ●●●   ●●\n${accent}   ●●       ●●"
        }
        "fedora" => {
            "${primary}      ▄████▄\n${secondary}    ███  ████\n${accent}    █████████\n${primary}      ██████"
        }
        "debian" => {
            "${primary}       ▄██▄\n${secondary}    ▄█▀  ▀██▄\n${accent}   ██      ██\n${primary}    ▀██▄▄██▀"
        }
        "mint" => {
            "${primary}   ┌─────────┐\n${secondary}   │ LM  LM  │\n${accent}   │  MINT   │\n${primary}   └─────────┘"
        }
        "retro" => {
            "${primary}   ┌────────┐\n${secondary}   │ ░░░░░░ │\n${accent}   │ ▓▓▓▓▓▓ │\n${muted}   └──┬──┬──┘"
        }
        "cat" => "${primary}  ╱╲___╱╲\n${accent} (  ◕ ◕  )\n${secondary}  ╲  ^  ╱",
        "pulse" | "rigglow" => {
            "${primary} ██████╗ ██╗ ██████╗\n${secondary} ██╔══██╗██║██╔════╝\n${accent} ██████╔╝██║██║  ███╗\n${primary} ██╔══██╗██║██║   ██║\n${secondary} ██║  ██║██║╚██████╔╝\n${accent} ╚═╝  ╚═╝╚═╝ ╚═════╝"
        }
        "linux" | "generic" => {
            "${primary}      .--.\n${secondary}     |o_o |\n${accent}     |:_/ |\n${primary}    //   | |\n${secondary}   (|     | )"
        }
        _ => return None,
    };
    Some(art.into())
}

pub fn distro_name() -> String {
    fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|v| {
            v.lines().find_map(|l| {
                l.strip_prefix("ID=")
                    .map(|v| v.trim_matches('"').to_owned())
            })
        })
        .unwrap_or_else(|| "linux".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn distro_chooses_expected_art() {
        assert_eq!(auto("EndeavourOS"), named("endeavouros").unwrap());
        assert_eq!(auto("Fedora Linux"), named("fedora").unwrap());
        assert_eq!(auto("something else"), named("linux").unwrap());
    }
}
