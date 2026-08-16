#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenColor {
    Primary,
    Secondary,
    Accent,
    Foreground,
    Muted,
    Reset,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    pub text: String,
    pub color: TokenColor,
}
pub fn parse(input: &str) -> Vec<Vec<Part>> {
    input.lines().map(parse_line).collect()
}
fn parse_line(line: &str) -> Vec<Part> {
    let mut out = Vec::new();
    let mut rest = line;
    let mut color = TokenColor::Primary;
    while let Some(pos) = rest.find("${") {
        if pos > 0 {
            out.push(Part {
                text: rest[..pos].into(),
                color,
            });
        }
        let after = &rest[pos..];
        let Some(end) = after.find('}') else {
            out.push(Part {
                text: after.into(),
                color,
            });
            return out;
        };
        let token = &after[2..end];
        if let Some(next) = match token {
            "primary" => Some(TokenColor::Primary),
            "secondary" => Some(TokenColor::Secondary),
            "accent" => Some(TokenColor::Accent),
            "foreground" => Some(TokenColor::Foreground),
            "muted" => Some(TokenColor::Muted),
            "reset" => Some(TokenColor::Reset),
            _ => None,
        } {
            color = next;
        } else {
            out.push(Part {
                text: after[..=end].into(),
                color,
            });
        }
        rest = &after[end + 1..];
    }
    if !rest.is_empty() {
        out.push(Part {
            text: rest.into(),
            color,
        });
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn color_tokens_split_and_unknown_tokens_survive() {
        let p = parse("${primary}A${accent}B${wat}C");
        assert_eq!(
            p[0][0],
            Part {
                text: "A".into(),
                color: TokenColor::Primary
            }
        );
        assert_eq!(
            p[0][1],
            Part {
                text: "B".into(),
                color: TokenColor::Accent
            }
        );
        assert_eq!(p[0][2].text, "${wat}");
        assert_eq!(
            p[0][3],
            Part {
                text: "C".into(),
                color: TokenColor::Accent
            }
        );
    }
}
