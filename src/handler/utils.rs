use crate::handler::error::{Error, Result};
use chrono::{Datelike, Local, NaiveDate};
const INDONESIA_MONTHS: [&str; 12] = [
    "Januari",
    "Februari",
    "Maret",
    "April",
    "Mei",
    "Juni",
    "Juli",
    "Agustus",
    "September",
    "Oktober",
    "November",
    "Desember",
];
pub fn format_date_indonesia(date_str: &str) -> Result<String> {
    let trimmed = date_str.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    let date = NaiveDate::parse_from_str(trimmed, "%d/%m/%Y").map_err(|e| Error::Date {
        message: format!("Invalid date format or value '{}': {}", trimmed, e),
    })?;
    let day = date.day();
    let month_idx = date.month() as usize - 1;
    let year = date.year();
    let month_name = INDONESIA_MONTHS.get(month_idx).unwrap_or(&"Invalid");
    Ok(format!("{} {} {}", day, month_name, year))
}
pub fn format_time_range(time_str: &str, default: &str) -> Result<String> {
    let trimmed = time_str.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("default") {
        return Ok(default.to_string());
    }
    let parts: Vec<&str> = trimmed.split('-').collect();
    if parts.len() != 2 {
        return Err(Error::Time {
            message: format!(
                "Invalid time range format: '{}'. Expected format 'start-end'",
                trimmed
            ),
        });
    }
    let from = normalize_time_part(parts[0])?;
    let to = normalize_time_part(parts[1])?;
    Ok(format!("{} - {} WIB", from, to))
}
fn normalize_time_part(part: &str) -> Result<String> {
    let part = part.trim();
    let (h_str, m_str) = if part.contains('.') {
        let mut s = part.split('.');
        (s.next().unwrap_or(""), s.next().unwrap_or("0"))
    } else if part.contains(':') {
        let mut s = part.split(':');
        (s.next().unwrap_or(""), s.next().unwrap_or("0"))
    } else {
        (part, "0")
    };
    let hour = h_str.parse::<u32>().map_err(|_| Error::Time {
        message: format!("Invalid hour value: '{}'", h_str),
    })?;
    let minute = m_str.parse::<u32>().map_err(|_| Error::Time {
        message: format!("Invalid minute value: '{}'", m_str),
    })?;
    if hour > 23 {
        return Err(Error::Time {
            message: format!("Hour out of range (0-23): {}", hour),
        });
    }
    if minute > 59 {
        return Err(Error::Time {
            message: format!("Minute out of range (0-59): {}", minute),
        });
    }
    Ok(format!("{:02}:{:02}", hour, minute))
}
#[must_use]
pub fn escape_xml(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            _ => result.push(c),
        }
    }
    result
}
#[allow(dead_code)]
#[must_use]
pub fn unescape_xml(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}
#[must_use]
pub fn get_today() -> String {
    let now = Local::now();
    now.format("%d/%m/%Y").to_string()
}
pub fn get_today_indonesia() -> Result<String> {
    format_date_indonesia(&get_today())
}
#[must_use]
pub fn get_current_time() -> String {
    let now = Local::now();
    now.format("%H:%M").to_string()
}
pub fn get_current_datetime_indonesia() -> Result<String> {
    let date = get_today_indonesia()?;
    let time = get_current_time();
    Ok(format!("{} pukul {}", date, time))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn format_date_indonesia_valid() {
        assert_eq!(
            format_date_indonesia("26/03/2026").unwrap(),
            "26 Maret 2026"
        );
        assert_eq!(
            format_date_indonesia("01/12/2020").unwrap(),
            "01 Desember 2020"
        );
    }
    #[test]
    fn format_date_indonesia_invalid_day() {
        assert!(format_date_indonesia("31/04/2026").is_err());
        assert!(format_date_indonesia("29/02/2021").is_err());
    }
    #[test]
    fn format_date_indonesia_invalid_format() {
        assert!(format_date_indonesia("2026-03-26").is_err());
        assert!(format_date_indonesia("26-03-2026").is_err());
    }
    #[test]
    fn format_time_range_hours_only() {
        let result = format_time_range("10-14", "default").unwrap();
        assert_eq!(result, "10:00 - 14:00 WIB");
    }
    #[test]
    fn format_time_range_with_minutes() {
        let result = format_time_range("10.30-12.45", "default").unwrap();
        assert_eq!(result, "10:30 - 12:45 WIB");
    }
    #[test]
    fn format_time_range_with_colon() {
        let result = format_time_range("10:30-12:45", "default").unwrap();
        assert_eq!(result, "10:30 - 12:45 WIB");
    }
    #[test]
    fn format_time_range_default_fallback() {
        let result = format_time_range("default", "13:00 WIB").unwrap();
        assert_eq!(result, "13:00 WIB");
    }
    #[test]
    fn format_time_range_invalid_hour() {
        assert!(format_time_range("25-14", "").is_err());
    }
    #[test]
    fn escape_xml_basic() {
        let result = escape_xml("Tom & Jerry <cat>");
        assert_eq!(result, "Tom &amp; Jerry &lt;cat&gt;");
    }
    #[test]
    fn escape_xml_quotes() {
        let result = escape_xml("He said \"Hello\"");
        assert_eq!(result, "He said &quot;Hello&quot;");
    }
    #[test]
    fn roundtrip_xml() {
        let original = "Tom & Jerry <cat> \"dog\"";
        let escaped = escape_xml(original);
        let unescaped = unescape_xml(&escaped);
        assert_eq!(original, unescaped);
    }
    #[test]
    fn normalize_time_valid() {
        assert_eq!(normalize_time_part("9").unwrap(), "09:00");
        assert_eq!(normalize_time_part("09").unwrap(), "09:00");
        assert_eq!(normalize_time_part("9.5").unwrap(), "09:05");
        assert_eq!(normalize_time_part("9.30").unwrap(), "09:30");
        assert_eq!(normalize_time_part(" 09 : 30 ").unwrap(), "09:30");
    }
    #[test]
    fn normalize_time_invalid() {
        assert!(normalize_time_part("24").is_err());
        assert!(normalize_time_part("10.60").is_err());
        assert!(normalize_time_part("abc").is_err());
    }
}
