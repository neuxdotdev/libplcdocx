use crate::handler::error::{Error, Result};
use chrono::Local;
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
    if date_str.is_empty() {
        return Ok(String::new());
    }
    let parts: Vec<&str> = date_str.split('/').collect();
    if parts.len() != 3 {
        return Err(Error::DateError(format!(
            "Invalid date format: {}. Expected dd/mm/yyyy",
            date_str
        )));
    }
    let day = parts[0]
        .parse::<u32>()
        .map_err(|e| Error::DateError(format!("Invalid day: {}", e)))?;
    let month = parts[1]
        .parse::<usize>()
        .map_err(|e| Error::DateError(format!("Invalid month: {}", e)))?;
    let year = parts[2]
        .parse::<i32>()
        .map_err(|e| Error::DateError(format!("Invalid year: {}", e)))?;
    if month < 1 || month > 12 {
        return Err(Error::DateError(format!("Month out of range: {}", month)));
    }
    if day < 1 || day > 31 {
        return Err(Error::DateError(format!("Day out of range: {}", day)));
    }
    Ok(format!("{} {} {}", day, INDONESIA_MONTHS[month - 1], year))
}
pub fn format_time_range(time_str: &str, default: &str) -> Result<String> {
    if time_str.is_empty() || time_str == "default" {
        return Ok(default.to_string());
    }
    let parts: Vec<&str> = time_str.split('-').collect();
    if parts.len() != 2 {
        return Err(Error::TimeError(format!(
            "Invalid time format: {}. Expected from-to",
            time_str
        )));
    }
    let from = normalize_time_part(parts[0], ".")?;
    let to = normalize_time_part(parts[1], ".")?;
    Ok(format!("{} - {} WIB", from, to))
}
fn normalize_time_part(part: &str, minute_sep: &str) -> Result<String> {
    let part = part.trim();
    if part.contains(minute_sep) {
        let sub: Vec<&str> = part.split(minute_sep).collect();
        if sub.len() != 2 {
            return Err(Error::TimeError(format!("Invalid time part: {}", part)));
        }
        let hour = sub[0]
            .parse::<u32>()
            .map_err(|e| Error::TimeError(format!("Invalid hour: {}", e)))?;
        let minute = sub[1]
            .parse::<u32>()
            .map_err(|e| Error::TimeError(format!("Invalid minute: {}", e)))?;
        if hour > 23 || minute > 59 {
            return Err(Error::TimeError(format!(
                "Invalid time value: {}:{:02}",
                hour, minute
            )));
        }
        Ok(format!("{:02}:{:02}", hour, minute))
    } else {
        let hour = part
            .parse::<u32>()
            .map_err(|e| Error::TimeError(format!("Invalid hour: {}", e)))?;
        if hour > 23 {
            return Err(Error::TimeError(format!("Hour out of range: {}", hour)));
        }
        Ok(format!("{:02}:00", hour))
    }
}
#[must_use]
pub fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
#[allow(dead_code)]
#[must_use]
pub fn unescape_xml(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
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
    fn format_date_indonesia_works() {
        let result = format_date_indonesia("26/03/2026").unwrap();
        assert_eq!(result, "26 Maret 2026");
    }
    #[test]
    fn format_time_range_works() {
        let result = format_time_range("10-14", "13:00 - 15:00 WIB").unwrap();
        assert_eq!(result, "10:00 - 14:00 WIB");
    }
    #[test]
    fn escape_xml_works() {
        let result = escape_xml("Tom & Jerry <cat>");
        assert_eq!(result, "Tom &amp; Jerry &lt;cat&gt;");
    }
    #[test]
    fn roundtrip_xml() {
        let original = "Tom & Jerry <cat> \"dog\"";
        let escaped = escape_xml(original);
        let unescaped = unescape_xml(&escaped);
        assert_eq!(original, unescaped);
    }
}
