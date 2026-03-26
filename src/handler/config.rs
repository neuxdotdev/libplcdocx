use crate::handler::error::{Error, Result};
use serde::{Deserialize, Serialize};
pub const MAX_DOCX_SIZE: u64 = 50 * 1024 * 1024;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlaceholderSyntax {
    pub prefix: String,
    pub suffix: String,
    pub escape: String,
}
impl Default for PlaceholderSyntax {
    fn default() -> Self {
        PlaceholderSyntax {
            prefix: "[[%%".to_string(),
            suffix: "%%]]".to_string(),
            escape: "\\".to_string(),
        }
    }
}
impl PlaceholderSyntax {
    pub fn validate(&self) -> Result<()> {
        if self.prefix.is_empty() {
            return Err(Error::ConfigError("Prefix cannot be empty".into()));
        }
        if self.suffix.is_empty() {
            return Err(Error::ConfigError("Suffix cannot be empty".into()));
        }
        if self.prefix.contains(char::is_alphanumeric) {
            return Err(Error::ConfigError(
                "Prefix should not contain alphanumeric characters".into(),
            ));
        }
        if self.suffix.contains(char::is_alphanumeric) {
            return Err(Error::ConfigError(
                "Suffix should not contain alphanumeric characters".into(),
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DateFormat {
    pub input_format: String,
    pub output_format: String,
    pub indonesia_locale: bool,
}
impl Default for DateFormat {
    fn default() -> Self {
        DateFormat {
            input_format: "%d/%m/%Y".to_string(),
            output_format: "%d %B %Y".to_string(),
            indonesia_locale: true,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimeFormat {
    pub separator: String,
    pub minute_separator: String,
    pub output_format: String,
    pub default_range: String,
    pub timezone: String,
}
impl Default for TimeFormat {
    fn default() -> Self {
        TimeFormat {
            separator: "-".to_string(),
            minute_separator: ".".to_string(),
            output_format: "{from} - {to} {tz}".to_string(),
            default_range: "13:00 - 15:00 WIB".to_string(),
            timezone: "WIB".to_string(),
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ProcessingMode {
    #[default]
    Lenient,
    Strict,
    Warn,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub syntax: PlaceholderSyntax,
    pub date_format: DateFormat,
    pub time_format: TimeFormat,
    pub mode: ProcessingMode,
    pub preserve_styles: bool,
    pub validate: bool,
    pub logging: bool,
    pub max_file_size: u64,
    pub security_checks: bool,
}
impl Default for Config {
    fn default() -> Self {
        Config {
            syntax: PlaceholderSyntax::default(),
            date_format: DateFormat::default(),
            time_format: TimeFormat::default(),
            mode: ProcessingMode::default(),
            preserve_styles: true,
            validate: true,
            logging: false,
            max_file_size: MAX_DOCX_SIZE,
            security_checks: true,
        }
    }
}
impl Config {
    #[must_use]
    pub fn new() -> Self {
        Config::default()
    }
    pub fn with_placeholder_prefix(mut self, prefix: &str) -> Result<Self> {
        if prefix.is_empty() {
            return Err(Error::ConfigError("Prefix cannot be empty".into()));
        }
        self.syntax.prefix = prefix.to_string();
        Ok(self)
    }
    pub fn with_placeholder_suffix(mut self, suffix: &str) -> Result<Self> {
        if suffix.is_empty() {
            return Err(Error::ConfigError("Suffix cannot be empty".into()));
        }
        self.syntax.suffix = suffix.to_string();
        Ok(self)
    }
    #[must_use]
    pub fn with_mode(mut self, mode: ProcessingMode) -> Self {
        self.mode = mode;
        self
    }
    #[must_use]
    pub fn with_logging(mut self, enabled: bool) -> Self {
        self.logging = enabled;
        self
    }
    #[must_use]
    pub fn with_preserve_styles(mut self, enabled: bool) -> Self {
        self.preserve_styles = enabled;
        self
    }
    #[must_use]
    pub fn with_max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = size;
        self
    }
    #[must_use]
    pub fn with_security_checks(mut self, enabled: bool) -> Self {
        self.security_checks = enabled;
        self
    }
    pub fn validate(&self) -> Result<()> {
        self.syntax.validate()?;
        if self.max_file_size == 0 {
            return Err(Error::ConfigError("Max file size cannot be zero".into()));
        }
        Ok(())
    }
    #[must_use]
    pub fn get_placeholder_pattern(&self, key: &str) -> String {
        format!("{}{}{}", self.syntax.prefix, key, self.syntax.suffix)
    }
    pub fn from_toml(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content).map_err(|e| Error::ConfigError(format!("TOML parse error: {}", e)))
    }
    pub fn from_json(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| Error::ConfigError(format!("JSON parse error: {}", e)))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_config_is_valid() {
        let cfg = Config::default();
        assert!(cfg.validate().is_ok());
    }
    #[test]
    fn builder_pattern_works() {
        let cfg = Config::default()
            .with_logging(true)
            .with_mode(ProcessingMode::Strict);
        assert!(cfg.logging);
        assert_eq!(cfg.mode, ProcessingMode::Strict);
    }
    #[test]
    fn invalid_prefix_rejected() {
        let res = Config::default().with_placeholder_prefix("");
        assert!(res.is_err());
    }
}
