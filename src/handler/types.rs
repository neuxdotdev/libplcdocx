use std::fmt;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlaceholderKey(String);
impl PlaceholderKey {
    #[must_use]
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.0.is_empty() {
            return Err("Placeholder key cannot be empty");
        }
        if self.0.contains(char::is_whitespace) {
            return Err("Placeholder key cannot contain whitespace");
        }
        Ok(())
    }
}
impl fmt::Display for PlaceholderKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<&str> for PlaceholderKey {
    fn from(s: &str) -> Self {
        PlaceholderKey::new(s)
    }
}
impl From<String> for PlaceholderKey {
    fn from(s: String) -> Self {
        PlaceholderKey::new(s)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplacementValue {
    content: String,
    escaped: bool,
}
impl ReplacementValue {
    #[must_use]
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            escaped: false,
        }
    }
    #[must_use]
    pub fn pre_escaped(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            escaped: true,
        }
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.content
    }
    #[must_use]
    pub fn is_escaped(&self) -> bool {
        self.escaped
    }
}
impl fmt::Display for ReplacementValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}
impl From<&str> for ReplacementValue {
    fn from(s: &str) -> Self {
        ReplacementValue::new(s)
    }
}
impl From<String> for ReplacementValue {
    fn from(s: String) -> Self {
        ReplacementValue::new(s)
    }
}
pub type PlaceholderMap = std::collections::HashMap<PlaceholderKey, ReplacementValue>;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn placeholder_key_validation() {
        let valid = PlaceholderKey::new("NAME");
        assert!(valid.validate().is_ok());
        let invalid = PlaceholderKey::new("NA ME");
        assert!(invalid.validate().is_err());
    }
    #[test]
    fn replacement_value_creation() {
        let val = ReplacementValue::new("foo");
        assert_eq!(val.as_str(), "foo");
        assert!(!val.is_escaped());
        let escaped = ReplacementValue::pre_escaped("bar");
        assert!(escaped.is_escaped());
    }
}
