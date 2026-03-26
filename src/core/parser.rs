use crate::handler::config::{Config, ProcessingMode};
use crate::handler::error::{Error, Result};
use crate::utils::escape_xml;
use std::collections::HashMap;
use tracing::{debug, warn};
#[derive(Debug, Clone)]
pub struct PlaceholderParser {
    config: Config,
}
impl PlaceholderParser {
    #[must_use]
    pub fn new(config: Config) -> Self {
        PlaceholderParser { config }
    }
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }
    #[must_use]
    pub fn find_all(&self, text: &str) -> Vec<String> {
        let mut placeholders = Vec::new();
        let mut start_idx = 0;
        while let Some(start) = text[start_idx..].find(&self.config.syntax.prefix) {
            let abs_start = start_idx + start;
            let search_from = abs_start + self.config.syntax.prefix.len();
            if let Some(end) = text[search_from..].find(&self.config.syntax.suffix) {
                let key_start = search_from;
                let key_end = search_from + end;
                let key = text[key_start..key_end].to_string();
                if !key.is_empty() && !placeholders.contains(&key) {
                    placeholders.push(key);
                }
                start_idx = key_end + self.config.syntax.suffix.len();
            } else {
                break;
            }
        }
        debug!("Found {} placeholders", placeholders.len());
        placeholders
    }
    pub fn replace_all(&self, text: &str, mappings: &HashMap<String, String>) -> Result<String> {
        let all_placeholders = self.find_all(text);
        self.validate_mappings(&all_placeholders, mappings)?;
        let mut result = text.to_string();
        let mut replaced_count = 0;
        for (key, value) in mappings {
            let placeholder = self.config.get_placeholder_pattern(key);
            if result.contains(&placeholder) {
                let escaped_value = escape_xml(value);
                result = result.replace(&placeholder, &escaped_value);
                replaced_count += 1;
                debug!("Replaced placeholder: {} -> {}", key, escaped_value);
            }
        }
        debug!("Total replacements: {}", replaced_count);
        Ok(result)
    }
    fn validate_mappings(
        &self,
        placeholders: &[String],
        mappings: &HashMap<String, String>,
    ) -> Result<()> {
        let missing: Vec<&String> = placeholders
            .iter()
            .filter(|ph| !mappings.contains_key(*ph))
            .collect();
        if missing.is_empty() {
            return Ok(());
        }
        match self.config.mode {
            ProcessingMode::Strict => {
                let missing_list = missing
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                Err(Error::PlaceholderNotFound(format!(
                    "Missing mappings for: {}",
                    missing_list
                )))
            }
            ProcessingMode::Warn => {
                for ph in missing {
                    warn!("Unmapped placeholder: {}", ph);
                }
                Ok(())
            }
            ProcessingMode::Lenient => Ok(()),
        }
    }
    #[must_use]
    pub fn has_placeholders(&self, text: &str) -> bool {
        text.contains(&self.config.syntax.prefix) && text.contains(&self.config.syntax.suffix)
    }
    #[must_use]
    pub fn count(&self, text: &str) -> usize {
        self.find_all(text).len()
    }
    #[must_use]
    pub fn find_unmapped(&self, text: &str, mappings: &HashMap<String, String>) -> Vec<String> {
        self.find_all(text)
            .into_iter()
            .filter(|ph| !mappings.contains_key(ph))
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn test_config() -> Config {
        Config::default().with_logging(true)
    }
    #[test]
    fn find_placeholders_works() {
        let parser = PlaceholderParser::new(test_config());
        let text = "Hello [[%%NAME%%]], today is [[%%DATE%%]]";
        let found = parser.find_all(text);
        assert_eq!(found.len(), 2);
        assert!(found.contains(&"NAME".to_string()));
        assert!(found.contains(&"DATE".to_string()));
    }
    #[test]
    fn replace_placeholders_works() {
        let parser = PlaceholderParser::new(test_config());
        let text = "Hello [[%%NAME%%]]!";
        let mut mappings = HashMap::new();
        mappings.insert("NAME".to_string(), "John".to_string());
        let result = parser.replace_all(text, &mappings).unwrap();
        assert_eq!(result, "Hello John!");
    }
    #[test]
    fn xml_escaping_applied() {
        let parser = PlaceholderParser::new(test_config());
        let text = "Value: [[%%CONTENT%%]]";
        let mut mappings = HashMap::new();
        mappings.insert("CONTENT".to_string(), "Tom & Jerry <cat>".to_string());
        let result = parser.replace_all(text, &mappings).unwrap();
        assert!(result.contains("&amp;"));
        assert!(result.contains("&lt;"));
    }
    #[test]
    fn strict_mode_reports_missing() {
        let config = Config::default().with_mode(ProcessingMode::Strict);
        let parser = PlaceholderParser::new(config);
        let text = "Hello [[%%NAME%%]] and [[%%MISSING%%]]";
        let mut mappings = HashMap::new();
        mappings.insert("NAME".to_string(), "John".to_string());
        let result = parser.replace_all(text, &mappings);
        assert!(result.is_err());
    }
}
