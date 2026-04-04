use crate::handler::config::{Config, ProcessingMode};
use crate::handler::error::{Error, Result};
use crate::handler::types::{PlaceholderKey, PlaceholderMap};
use crate::handler::utils::escape_xml;
use regex::Regex;
use std::collections::HashSet;
use tracing::{debug, warn};
const MAX_PATTERN_LENGTH: usize = 2048;
const MAX_INPUT_TEXT_LEN: usize = 50 * 1024 * 1024;
const MAX_OUTPUT_EXPANSION_FACTOR: usize = 10;
const MAX_PLACEHOLDER_KEY_LEN: usize = 256;
#[derive(Debug, Clone)]
pub struct PlaceholderParser {
    config: Config,
    regex: Regex,
}
impl PlaceholderParser {
    pub fn new(config: Config) -> Result<Self> {
        config.validate()?;
        let syntax = config.syntax();
        if syntax.prefix().is_empty() || syntax.suffix().is_empty() {
            return Err(Error::config(
                "Placeholder prefix and suffix cannot be empty",
            ));
        }
        let escaped_prefix = regex::escape(syntax.prefix());
        let escaped_suffix = regex::escape(syntax.suffix());
        let pattern = format!(r"{}(.*?){}", escaped_prefix, escaped_suffix);
        if pattern.len() > MAX_PATTERN_LENGTH {
            return Err(Error::config(format!(
                "Generated regex pattern length {} exceeds limit {}",
                pattern.len(),
                MAX_PATTERN_LENGTH
            )));
        }
        let regex = Regex::new(&pattern).map_err(|e| Error::invalid_regex(&pattern, e))?;
        Ok(Self { config, regex })
    }
    pub fn find_all(&self, text: &str) -> Vec<PlaceholderKey> {
        if text.is_empty() {
            return Vec::new();
        }
        if text.len() > MAX_INPUT_TEXT_LEN {
            warn!(
                "find_all called with very large text ({} bytes), performance may degrade",
                text.len()
            );
        }
        let mut keys_set = HashSet::new();
        for cap in self.regex.captures_iter(text) {
            if let Some(key_match) = cap.get(1) {
                let key_str = key_match.as_str();
                if key_str.is_empty() || key_str.len() > MAX_PLACEHOLDER_KEY_LEN {
                    continue;
                }
                match PlaceholderKey::new(key_str) {
                    Ok(key) => {
                        keys_set.insert(key);
                    }
                    Err(e) => {
                        warn!("Invalid placeholder key '{}': {}", key_str, e);
                    }
                }
            }
        }
        let keys: Vec<PlaceholderKey> = keys_set.into_iter().collect();
        debug!("Found {} unique valid placeholders", keys.len());
        keys
    }
    pub fn replace_all(&self, text: &str, mappings: &PlaceholderMap) -> Result<String> {
        if text.len() > MAX_INPUT_TEXT_LEN {
            return Err(Error::resource_limit(
                "input_text_size",
                text.len(),
                MAX_INPUT_TEXT_LEN,
            ));
        }
        let placeholders = self.find_all(text);
        if placeholders.is_empty() {
            return Ok(text.to_string());
        }
        let max_placeholders = self.config.max_placeholders_per_file();
        if placeholders.len() > max_placeholders {
            return Err(Error::resource_limit(
                "placeholder_count",
                placeholders.len(),
                max_placeholders,
            ));
        }
        self.validate_mappings(&placeholders, mappings)?;
        let mut result = String::with_capacity(text.len() * 2);
        let mut replaced_count = 0;
        result.push_str(text);
        let mut result_text = text.to_string();
        for key in &placeholders {
            let pattern = self.config.syntax().pattern(key.as_str());
            if let Some(value) = mappings.get(key) {
                let final_value = if value.is_pre_escaped() {
                    value.as_str().to_string()
                } else {
                    escape_xml(value.as_str())
                };
                let max_replacement = self.config.max_replacement_size();
                if final_value.len() > max_replacement {
                    return Err(Error::resource_limit(
                        "replacement_size",
                        final_value.len(),
                        max_replacement,
                    ));
                }
                if final_value.len() > text.len() * MAX_OUTPUT_EXPANSION_FACTOR {
                    warn!("Potential output expansion limit hit for key {}", key);
                }
                if result_text.contains(&pattern) {
                    result_text = result_text.replace(&pattern, &final_value);
                    replaced_count += 1;
                    debug!("Replaced '{}' -> '{}'", key, final_value);
                }
            }
        }
        if self.config.mode() == ProcessingMode::Strict && self.has_placeholders(&result_text) {
            warn!("After replacement, placeholders still remain in strict mode");
        }
        debug!("Total replacements performed: {}", replaced_count);
        Ok(result_text)
    }
    fn validate_mappings(
        &self,
        placeholders: &[PlaceholderKey],
        mappings: &PlaceholderMap,
    ) -> Result<()> {
        let missing: Vec<&PlaceholderKey> = placeholders
            .iter()
            .filter(|ph| !mappings.contains_key(*ph))
            .collect();
        if missing.is_empty() {
            return Ok(());
        }
        match self.config.mode() {
            ProcessingMode::Strict => {
                let missing_keys = missing
                    .iter()
                    .map(|k| k.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                Err(Error::placeholder_not_found(missing_keys, None::<String>))
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
    pub fn has_placeholders(&self, text: &str) -> bool {
        !text.is_empty() && self.regex.is_match(text)
    }
    pub fn count(&self, text: &str) -> usize {
        self.find_all(text).len()
    }
    pub fn get_pattern(&self) -> &str {
        self.regex.as_str()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::config::{PlaceholderSyntax, ProcessingMode};
    fn dummy_config() -> Config {
        Config::builder()
            .syntax(PlaceholderSyntax::new("{{", "}}", '\\').unwrap())
            .mode(ProcessingMode::Strict)
            .max_placeholders(100)
            .max_replacement_size(1024)
            .build()
            .unwrap()
    }
    #[test]
    fn test_new_valid() {
        let parser = PlaceholderParser::new(dummy_config());
        assert!(parser.is_ok());
    }
    #[test]
    fn test_new_empty_prefix() {
        let syntax = PlaceholderSyntax::new("", "}}", '\\').unwrap();
        let cfg_res = Config::builder().syntax(syntax).build();
        assert!(cfg_res.is_err());
    }
    #[test]
    fn test_find_all() {
        let parser = PlaceholderParser::new(dummy_config()).unwrap();
        let text = "Hello {{name}}, today is {{date}} and {{name}} again.";
        let keys = parser.find_all(text);
        assert_eq!(keys.len(), 2);
        assert!(keys.iter().any(|k| k.as_str() == "name"));
        assert!(keys.iter().any(|k| k.as_str() == "date"));
    }
    #[test]
    fn test_replace_all_strict() {
        let parser = PlaceholderParser::new(dummy_config()).unwrap();
        let text = "Hello {{name}}!";
        let mut mappings = PlaceholderMap::new();
        mappings.insert(PlaceholderKey::new("name").unwrap(), "Alice".into());
        let result = parser.replace_all(text, &mappings).unwrap();
        assert_eq!(result, "Hello Alice!");
    }
}
