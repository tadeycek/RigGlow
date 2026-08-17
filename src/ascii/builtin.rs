use std::fs;

/// Every shipped logo occupies this exact terminal canvas. It matches the
/// large EndeavourOS treatment while keeping the left dashboard column stable
/// when art is switched at runtime.
pub const CANVAS_WIDTH: usize = 48;
pub const CANVAS_HEIGHT: usize = 15;

pub fn auto(distro: &str) -> String {
    let distro = distro.to_ascii_lowercase();
    let logo = if distro.contains("endeavour") {
        "endeavouros"
    } else if distro.contains("arch") {
        "arch"
    } else if distro.contains("ubuntu") {
        "ubuntu"
    } else if distro.contains("fedora") {
        "fedora"
    } else if distro.contains("debian") {
        "debian"
    } else if distro.contains("mint") {
        "mint"
    } else if distro.contains("manjaro") {
        "manjaro"
    } else if distro.contains("opensuse") || distro == "sles" {
        "opensuse"
    } else if distro.contains("pop") {
        "popos"
    } else if distro.contains("kali") {
        "kali"
    } else if distro.contains("nixos") {
        "nixos"
    } else if distro.contains("gentoo") {
        "gentoo"
    } else if distro.contains("rhel") || distro.contains("redhat") || distro.contains("red hat") {
        "redhat"
    } else if distro.contains("rocky") {
        "rocky"
    } else if distro.contains("alma") {
        "almalinux"
    } else if distro.contains("void") {
        "void"
    } else if distro.contains("solus") {
        "solus"
    } else if distro.contains("elementary") {
        "elementary"
    } else if distro.contains("zorin") {
        "zorin"
    } else if distro == "mx" || distro.contains("mx linux") {
        "mx"
    } else {
        "linux"
    };
    named(logo).unwrap_or_default()
}

pub fn named(name: &str) -> Option<String> {
    let art = match name.to_ascii_lowercase().as_str() {
        "arch" | "archlinux" => {
            "${primary}                   -`\n${primary}                  .o+`\n${secondary}                 `ooo/\n${secondary}                `+oooo:\n${accent}               `+oooooo:\n${accent}              -+oooooo+:\n${primary}            `/:-:++oooo+:\n${secondary}           `/++++/+++++++:\n${accent}          `/++++++++++++++:\n${primary}         `/+++ooooooooooooo/`\n${secondary}        ./ooosssso++osssssso+`\n${accent}       -osssssso.      :ssssssso."
        }
        "endeavouros" | "endeavour" => {
            "${primary}                     ./o.\n${primary}                   ./o${accent}sssso${primary}-\n${primary}                 `:${accent}osssssss${primary}+-\n${primary}               `:+${accent}sssssssssso${primary}/.\n${primary}             `-/o${accent}ssssssssssssso${primary}/.\n${primary}           `-/+${accent}sssssssssssssssso${primary}+:`\n${primary}         `-:/+${accent}sssssssssssssssssso${primary}+/.\n${primary}       `.://o${accent}sssssssssssssssssssso${primary}++-\n${primary}      .://+${accent}ssssssssssssssssssssssso${primary}++:\n${primary}    .:///o${accent}ssssssssssssssssssssssssso${primary}++:\n${primary}  `:////${accent}ssssssssssssssssssssssssssso${primary}+++.\n${primary}`-////+${accent}ssssssssssssssssssssssssssso${primary}++++-\n${primary}  `..-+oo${accent}ssssssssssssssssssssssso${primary}+++++/`\n${primary}   ./++++++++++++++++++++++++++++++/:.\n${muted}  `:::::::::::::::::::::::::------``"
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
        "manjaro" => {
            "${primary} ██████  ████\n${primary} ██████  ████\n${secondary} ██████  ████\n${secondary} ██████      \n${accent} ██████  ████\n${accent} ██████  ████"
        }
        "opensuse" | "suse" => {
            "${primary}        .;ldkO0000Okdl;.\n${secondary}    .;d00xl:^''''''^:ok00d;.\n${accent}  .d00l'                'o00d.\n${primary} .x00l'     .;:;;:.     'l00x.\n${secondary},x00l     ;OOOOOOO;     l00x,\n${accent}x00l      OOOOOOOO0      l00x"
        }
        "pop" | "popos" | "pop_os" => {
            "${primary} ████████████\n${secondary} ███      ███\n${accent} ███  ██  ███\n${primary} ███      ███\n${secondary} ████████████"
        }
        "kali" | "kalilinux" => {
            "${primary}        ..,,:;ccllll\n${primary}  ...,,+:;  cllllllllllllllllll\n${secondary}  ,ccll;  :ll  lllllllllllllllllll\n${accent}  :lcc,  ccl  llllllllllllllllllll\n${accent}  'lllc  .cc  lllllllllllllllllllll\n${primary}   ;lllll  'cc  .lllllllllllllllllll"
        }
        "nixos" => {
            "${primary}          ▗▄▄▄▄▄▄▖\n${secondary}       ▗▄██████████▄▖\n${accent}    ▗▄███▀▀  ▐▌  ▀▀███▄▖\n${primary}  ▟██▀   ▄██████▄   ▀██▙\n${secondary}  ▜██▄   ▀██████▀   ▄██▛\n${accent}    ▀▀███▄▄  ▐▌  ▄▄███▀▀"
        }
        "gentoo" => {
            "${primary}         -/oyddmdhs+:.\n${secondary}     -odNMMMMMMMMNNmhy+-`\n${accent}   -yNMMMMMMMMMMMNNNmmdhy+-\n${primary} `omMMMMMMMMMMMMNmdmmmmddhhy/`\n${secondary} omMMMMMMMMMMMNhhyyyohmdddhhhdo`\n${accent}.ydMMMMMMMMMMdhs++so+sdddhhhhdm+"
        }
        "redhat" | "rhel" => {
            "${primary}           .-====-.\n${secondary}        .-=========-.\n${accent}      .-=====-.  .-===-.\n${primary}      ======      ======\n${secondary}       '====-.  .-===='\n${accent}          '-======-'"
        }
        "rocky" | "rockylinux" => {
            "${primary}     ██████╗  ██████╗\n${secondary}     ██╔══██╗██╔═══██╗\n${accent}     ██████╔╝██║   ██║\n${primary}     ██╔══██╗██║   ██║\n${secondary}     ██║  ██║╚██████╔╝"
        }
        "alma" | "almalinux" => {
            "${primary}      ████  ████\n${secondary}    ██████████████\n${accent}   ████  ████  ████\n${primary}   ████  ████  ████\n${secondary}    ██████████████"
        }
        "void" | "voidlinux" => {
            "${primary}       _______\n${secondary}    _-        -_\n${accent}  _-   _-__-_   -_\n${primary} |   _-    -_   |\n${secondary}  -_   -__-   _-\n${accent}    -_        _-"
        }
        "solus" => {
            "${primary}        ╭──────╮\n${secondary}      ╭─╯  ╭╮  ╰─╮\n${accent}      │   ╰╯   │\n${primary}      ╰─╮  ╭╮  ╭─╯\n${secondary}        ╰──────╯"
        }
        "elementary" | "elementaryos" => {
            "${primary}        eeeeeee\n${secondary}     eeeeeeeeeeee\n${accent}   eeeeeeeeeeeeeeee\n${primary}  eeee    eeee    eeee\n${secondary}  eeeeeeeeeeeeeeeeee"
        }
        "zorin" | "zorinos" => {
            "${primary}      ██████████\n${secondary}         ████\n${accent}        ████\n${primary}       ████\n${secondary}      ██████████"
        }
        "mx" | "mxlinux" => {
            "${primary}          ▄▄▄▄\n${secondary}       ▄██████▄\n${accent}    ▄███▀  ▀███▄\n${primary}  ▄██▀      ▀██▄\n${secondary} ▀▀          ▀▀"
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
    Some(normalize(art))
}

/// Centres an artwork on RigGlow's standard canvas without changing its art.
/// Wider/taller user-supplied artwork is deliberately preserved instead of
/// being cropped, while normal artwork is always exactly 48×15 cells.
pub fn normalize(art: &str) -> String {
    let lines = art.lines().collect::<Vec<_>>();
    let input_height = lines.len();
    let width = lines
        .iter()
        .map(|line| visible_width(line))
        .max()
        .unwrap_or(0)
        .max(CANVAS_WIDTH);
    let height = input_height.max(CANVAS_HEIGHT);
    let top_padding = (height - input_height) / 2;
    let art_width = lines
        .iter()
        .map(|line| visible_width(line))
        .max()
        .unwrap_or(0);
    // Apply one common left margin to the whole artwork. Individual line
    // offsets are part of a logo's geometry and must never be re-centred.
    let left_padding = (width - art_width) / 2;
    let mut out = Vec::with_capacity(height);

    out.extend(std::iter::repeat_n(" ".repeat(width), top_padding));
    for line in lines {
        let line_width = visible_width(line);
        let right_padding = width.saturating_sub(line_width + left_padding);
        out.push(format!(
            "{}{}{}",
            " ".repeat(left_padding),
            line,
            " ".repeat(right_padding)
        ));
    }
    out.extend(std::iter::repeat_n(
        " ".repeat(width),
        height - top_padding - input_height,
    ));
    out.join("\n")
}

/// `${theme}` tokens occupy no terminal cells, so normalisation must ignore
/// them when calculating the visual canvas width.
fn visible_width(line: &str) -> usize {
    let mut width = 0;
    let mut rest = line;
    while let Some(token_start) = rest.find("${") {
        width += rest[..token_start].chars().count();
        let after = &rest[token_start + 2..];
        let Some(token_end) = after.find('}') else {
            return width + rest[token_start..].chars().count();
        };
        rest = &after[token_end + 1..];
    }
    width + rest.chars().count()
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
        for distro in [
            "Arch Linux",
            "Ubuntu",
            "Debian",
            "Linux Mint",
            "Manjaro",
            "openSUSE Tumbleweed",
            "Pop!_OS",
            "Kali Linux",
            "NixOS",
            "Gentoo",
            "Red Hat Enterprise Linux",
            "Rocky Linux",
            "AlmaLinux",
            "Void",
            "Solus",
            "elementary OS",
            "Zorin OS",
            "MX Linux",
        ] {
            assert_ne!(
                auto(distro),
                named("linux").unwrap(),
                "{distro} should select a dedicated logo"
            );
        }
        assert_eq!(auto("something else"), named("linux").unwrap());
    }

    #[test]
    fn every_builtin_logo_uses_the_same_canvas() {
        for name in [
            "arch",
            "endeavouros",
            "ubuntu",
            "fedora",
            "debian",
            "mint",
            "manjaro",
            "opensuse",
            "popos",
            "kali",
            "nixos",
            "gentoo",
            "redhat",
            "rocky",
            "almalinux",
            "void",
            "solus",
            "elementary",
            "zorin",
            "mx",
            "linux",
            "cat",
            "retro",
            "rigglow",
        ] {
            let art = named(name).expect("built-in art should exist");
            let lines = art.lines().collect::<Vec<_>>();
            assert_eq!(lines.len(), CANVAS_HEIGHT, "{name} height");
            assert!(
                lines.iter().all(|line| visible_width(line) == CANVAS_WIDTH),
                "{name} width"
            );
        }
    }
}
