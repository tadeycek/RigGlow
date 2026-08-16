pub mod compact;
pub mod json;
pub mod static_view;

pub fn format_bytes(bytes: f64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes.max(0.0);
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}
pub fn format_rate(bytes: f64) -> String {
    format!("{}/s", format_bytes(bytes))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn byte_formatting_is_human_readable() {
        assert_eq!(format_bytes(1023.0), "1023 B");
        assert_eq!(format_bytes(1024.0), "1.0 KiB");
        assert_eq!(format_bytes(1_572_864.0), "1.5 MiB");
    }
}
