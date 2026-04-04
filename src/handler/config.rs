use crate::handler::error::{Error, Result};
use serde::{Deserialize, Serialize};
pub const DEFAULT_MAX_DOCX_SIZE: u64 = 50 * 1024 * 1024;
pub const DEFAULT_MAX_PLACEHOLDERS: usize = 10_000;
pub const DEFAULT_MAX_REPLACEMENT_SIZE: usize = 1024 * 1024;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PlaceholderSyntax {
    prefix: String,
    suffix: String,
    escape_char: char,
}
impl PlaceholderSyntax {
    #[inline]
    pub fn new(
        prefix: impl Into<String>,
        suffix: impl Into<String>,
        escape_char: char,
    ) -> Result<Self> {
        let prefix = prefix.into();
        let suffix = suffix.into();
        if prefix.is_empty() {
            return Err(Error::Config {
                message: "Placeholder prefix cannot be empty".into(),
            });
        }
        if suffix.is_empty() {
            return Err(Error::Config {
                message: "Placeholder suffix cannot be empty".into(),
            });
        }
        if prefix == suffix {
            return Err(Error::Config {
                message: "Prefix and suffix must differ to avoid ambiguous parsing".into(),
            });
        }
        if prefix.contains(&suffix) || suffix.contains(&prefix) {
            return Err(Error::Config {
                message: "Prefix must not contain suffix (and vice versa)".into(),
            });
        }
        Ok(Self {
            prefix,
            suffix,
            escape_char,
        })
    }
    #[inline]
    #[must_use]
    pub fn prefix(&self) -> &str {
        &self.prefix
    }
    #[inline]
    #[must_use]
    pub fn suffix(&self) -> &str {
        &self.suffix
    }
    #[inline]
    #[must_use]
    pub fn escape_char(&self) -> char {
        self.escape_char
    }
    #[inline]
    #[must_use]
    pub fn pattern(&self, key: &str) -> String {
        format!("{}{}{}", self.prefix, key, self.suffix)
    }
    #[inline]
    #[must_use]
    pub fn regex_pattern(&self, key: &str) -> String {
        let escape = |s: &str| {
            s.replace('.', "\\.")
                .replace('(', "\\(")
                .replace(')', "\\)")
                .replace('[', "\\[")
                .replace(']', "\\]")
                .replace('*', "\\*")
                .replace('+', "\\+")
                .replace('?', "\\?")
                .replace('{', "\\{")
                .replace('}', "\\}")
                .replace('^', "\\^")
                .replace('$', "\\$")
                .replace('|', "\\|")
        };
        format!(
            "{}{}{}",
            escape(&self.prefix),
            regex::escape(key),
            escape(&self.suffix)
        )
    }
    pub fn validate(&self) -> Result<()> {
        if self.prefix.is_empty() {
            return Err(Error::config("Placeholder prefix cannot be empty"));
        }
        if self.suffix.is_empty() {
            return Err(Error::config("Placeholder suffix cannot be empty"));
        }
        if self.prefix == self.suffix {
            return Err(Error::config("Prefix and suffix must differ"));
        }
        if self.prefix.contains(&self.suffix) || self.suffix.contains(&self.prefix) {
            return Err(Error::config(
                "Prefix must not contain suffix (and vice versa)",
            ));
        }
        Ok(())
    }
}
impl Default for PlaceholderSyntax {
    fn default() -> Self {
        Self {
            prefix: "[[%%".into(),
            suffix: "%%]]".into(),
            escape_char: '\\',
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DateFormat {
    input_format: String,
    output_format: String,
    indonesia_locale: bool,
}
impl DateFormat {
    #[inline]
    pub fn new(
        input: impl Into<String>,
        output: impl Into<String>,
        indonesia_locale: bool,
    ) -> Self {
        Self {
            input_format: input.into(),
            output_format: output.into(),
            indonesia_locale,
        }
    }
    #[inline]
    #[must_use]
    pub fn input_format(&self) -> &str {
        &self.input_format
    }
    #[inline]
    #[must_use]
    pub fn output_format(&self) -> &str {
        &self.output_format
    }
    #[inline]
    #[must_use]
    pub fn is_indonesia_locale(&self) -> bool {
        self.indonesia_locale
    }
    #[inline]
    pub fn validate(&self) -> Result<()> {
        if self.input_format.is_empty() {
            return Err(Error::Config {
                message: "Date input format cannot be empty".into(),
            });
        }
        if self.output_format.is_empty() {
            return Err(Error::Config {
                message: "Date output format cannot be empty".into(),
            });
        }
        Ok(())
    }
}
impl Default for DateFormat {
    fn default() -> Self {
        Self {
            input_format: "%d/%m/%Y".into(),
            output_format: "%d %B %Y".into(),
            indonesia_locale: true,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TimeFormat {
    separator: String,
    minute_separator: String,
    output_format: String,
    default_range: String,
    timezone: String,
}
impl TimeFormat {
    #[inline]
    pub fn new(
        separator: impl Into<String>,
        minute_sep: impl Into<String>,
        output: impl Into<String>,
        default_range: impl Into<String>,
        timezone: impl Into<String>,
    ) -> Self {
        Self {
            separator: separator.into(),
            minute_separator: minute_sep.into(),
            output_format: output.into(),
            default_range: default_range.into(),
            timezone: timezone.into(),
        }
    }
    #[inline]
    #[must_use]
    pub fn separator(&self) -> &str {
        &self.separator
    }
    #[inline]
    #[must_use]
    pub fn minute_separator(&self) -> &str {
        &self.minute_separator
    }
    #[inline]
    #[must_use]
    pub fn output_format(&self) -> &str {
        &self.output_format
    }
    #[inline]
    #[must_use]
    pub fn default_range(&self) -> &str {
        &self.default_range
    }
    #[inline]
    #[must_use]
    pub fn timezone(&self) -> &str {
        &self.timezone
    }
    #[inline]
    pub fn validate(&self) -> Result<()> {
        if self.separator.is_empty() {
            return Err(Error::Config {
                message: "Time separator cannot be empty".into(),
            });
        }
        if self.timezone.is_empty() {
            return Err(Error::Config {
                message: "Timezone cannot be empty".into(),
            });
        }
        if !self.output_format.contains("{from}") || !self.output_format.contains("{to}") {
            return Err(Error::Config {
                message: "Time output_format must contain {from} and {to} placeholders".into(),
            });
        }
        Ok(())
    }
}
impl Default for TimeFormat {
    fn default() -> Self {
        Self {
            separator: "-".into(),
            minute_separator: ".".into(),
            output_format: "{from} - {to} {tz}".into(),
            default_range: "13:00 - 15:00 WIB".into(),
            timezone: "WIB".into(),
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
impl ProcessingMode {
    #[inline]
    #[must_use]
    pub fn is_strict(&self) -> bool {
        matches!(self, Self::Strict)
    }
    #[inline]
    #[must_use]
    pub fn is_warn(&self) -> bool {
        matches!(self, Self::Warn)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    syntax: PlaceholderSyntax,
    date_format: DateFormat,
    time_format: TimeFormat,
    mode: ProcessingMode,
    logging: bool,
    max_file_size: u64,
    max_placeholders_per_file: usize,
    max_replacement_size: usize,
    security_checks: bool,
}
impl Config {
    #[inline]
    #[must_use]
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
    #[inline]
    #[must_use]
    pub fn syntax(&self) -> &PlaceholderSyntax {
        &self.syntax
    }
    #[inline]
    #[must_use]
    pub fn date_format(&self) -> &DateFormat {
        &self.date_format
    }
    #[inline]
    #[must_use]
    pub fn time_format(&self) -> &TimeFormat {
        &self.time_format
    }
    #[inline]
    #[must_use]
    pub fn mode(&self) -> ProcessingMode {
        self.mode
    }
    #[inline]
    #[must_use]
    pub fn is_logging_enabled(&self) -> bool {
        self.logging
    }
    #[inline]
    #[must_use]
    pub fn max_file_size(&self) -> u64 {
        self.max_file_size
    }
    #[inline]
    #[must_use]
    pub fn max_placeholders_per_file(&self) -> usize {
        self.max_placeholders_per_file
    }
    #[inline]
    #[must_use]
    pub fn max_replacement_size(&self) -> usize {
        self.max_replacement_size
    }
    #[inline]
    #[must_use]
    pub fn has_security_checks(&self) -> bool {
        self.security_checks
    }
    #[inline]
    pub fn validate(&self) -> Result<()> {
        self.syntax.validate()?;
        self.date_format.validate()?;
        self.time_format.validate()?;
        if self.max_file_size == 0 {
            return Err(Error::Config {
                message: "max_file_size cannot be zero".into(),
            });
        }
        if self.max_placeholders_per_file == 0 {
            return Err(Error::Config {
                message: "max_placeholders_per_file cannot be zero".into(),
            });
        }
        if self.max_replacement_size == 0 {
            return Err(Error::Config {
                message: "max_replacement_size cannot be zero".into(),
            });
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigBuilder {
    syntax: PlaceholderSyntax,
    date_format: DateFormat,
    time_format: TimeFormat,
    mode: ProcessingMode,
    logging: bool,
    max_file_size: u64,
    max_placeholders_per_file: usize,
    max_replacement_size: usize,
    security_checks: bool,
}
impl Default for ConfigBuilder {
    fn default() -> Self {
        Self {
            syntax: PlaceholderSyntax::default(),
            date_format: DateFormat::default(),
            time_format: TimeFormat::default(),
            mode: ProcessingMode::default(),
            logging: false,
            max_file_size: DEFAULT_MAX_DOCX_SIZE,
            max_placeholders_per_file: DEFAULT_MAX_PLACEHOLDERS,
            max_replacement_size: DEFAULT_MAX_REPLACEMENT_SIZE,
            security_checks: true,
        }
    }
}
impl ConfigBuilder {
    #[inline]
    #[must_use = "config builder must be built"]
    pub fn build(self) -> Result<Config> {
        let config = Config {
            syntax: self.syntax,
            date_format: self.date_format,
            time_format: self.time_format,
            mode: self.mode,
            logging: self.logging,
            max_file_size: self.max_file_size,
            max_placeholders_per_file: self.max_placeholders_per_file,
            max_replacement_size: self.max_replacement_size,
            security_checks: self.security_checks,
        };
        config.validate()?;
        Ok(config)
    }
    #[inline]
    #[must_use]
    pub fn syntax(mut self, syntax: PlaceholderSyntax) -> Self {
        self.syntax = syntax;
        self
    }
    #[inline]
    #[must_use]
    pub fn date_format(mut self, df: DateFormat) -> Self {
        self.date_format = df;
        self
    }
    #[inline]
    #[must_use]
    pub fn time_format(mut self, tf: TimeFormat) -> Self {
        self.time_format = tf;
        self
    }
    #[inline]
    #[must_use]
    pub fn mode(mut self, mode: ProcessingMode) -> Self {
        self.mode = mode;
        self
    }
    #[inline]
    #[must_use]
    pub fn logging(mut self, enabled: bool) -> Self {
        self.logging = enabled;
        self
    }
    #[inline]
    #[must_use]
    pub fn max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = size;
        self
    }
    #[inline]
    #[must_use]
    pub fn max_placeholders(mut self, max: usize) -> Self {
        self.max_placeholders_per_file = max;
        self
    }
    #[inline]
    #[must_use]
    pub fn max_replacement_size(mut self, size: usize) -> Self {
        self.max_replacement_size = size;
        self
    }
    #[inline]
    #[must_use]
    pub fn security_checks(mut self, enabled: bool) -> Self {
        self.security_checks = enabled;
        self
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_config_is_valid() {
        let config = Config::builder().build().unwrap();
        assert_eq!(config.mode(), ProcessingMode::Lenient);
        assert!(config.has_security_checks());
        assert_eq!(config.max_file_size(), DEFAULT_MAX_DOCX_SIZE);
    }
    #[test]
    fn builder_validation_catches_zero_limits() {
        let result = Config::builder().max_file_size(0).build();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be zero"));
    }
    #[test]
    fn placeholder_syntax_validation() {
        assert!(PlaceholderSyntax::new("{{", "}}", '\\').is_ok());
        assert!(PlaceholderSyntax::new("", "}}", '\\').is_err());
        assert!(PlaceholderSyntax::new("{{", "", '\\').is_err());
        assert!(PlaceholderSyntax::new("X", "X", '\\').is_err());
        assert!(PlaceholderSyntax::new("A{B", "B", '\\').is_err());
    }
    #[test]
    fn time_format_validation() {
        let valid = TimeFormat::new("-", ".", "{from} - {to} {tz}", "13:00", "WIB");
        assert!(valid.validate().is_ok());
        let missing_token = TimeFormat::new("-", ".", "invalid", "13:00", "WIB");
        assert!(missing_token.validate().is_err());
    }
    #[test]
    fn serde_roundtrip_with_validation() {
        let config = Config::builder().logging(true).build().unwrap();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();
        assert!(parsed.validate().is_ok());
        assert!(parsed.is_logging_enabled());
    }
    #[test]
    fn fluent_builder_chaining() {
        let config = Config::builder()
            .mode(ProcessingMode::Strict)
            .max_file_size(1024)
            .security_checks(false)
            .build()
            .unwrap();
        assert_eq!(config.mode(), ProcessingMode::Strict);
        assert_eq!(config.max_file_size(), 1024);
        assert!(!config.has_security_checks());
    }
}
