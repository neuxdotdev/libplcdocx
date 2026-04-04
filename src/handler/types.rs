use std::collections::HashMap;
use std::fmt;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlaceholderKeyError {
    Empty,
    ContainsWhitespace,
    ContainsInvalidCharacter(char),
}
impl fmt::Display for PlaceholderKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "placeholder key cannot be empty"),
            Self::ContainsWhitespace => write!(f, "placeholder key cannot contain whitespace"),
            Self::ContainsInvalidCharacter(ch) => {
                write!(f, "placeholder key contains invalid character: {:?}", ch)
            }
        }
    }
}
impl std::error::Error for PlaceholderKeyError {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ReplacementValueError {
    ContainsPlaceholderSyntax,
}
impl fmt::Display for ReplacementValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContainsPlaceholderSyntax => {
                write!(
                    f,
                    "replacement value should not contain placeholder syntax {{{{...}}}}"
                )
            }
        }
    }
}
impl std::error::Error for ReplacementValueError {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct PlaceholderKey(String);
const _: () = {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    let _ = (assert_send::<PlaceholderKey>, assert_sync::<PlaceholderKey>);
};
impl PlaceholderKey {
    const FORBIDDEN_CHARS: &[char] = &[
        '{', '}', '[', ']', '(', ')', '<', '>', '&', '\'', '"', '\\', '|', '$', '`', '%', '#', '@',
        '!', '*',
    ];
    #[inline]
    #[must_use = "this Result must be handled"]
    pub fn new(key: impl Into<String>) -> Result<Self, PlaceholderKeyError> {
        let s = key.into();
        Self::validate(&s)?;
        Ok(Self(s))
    }
    #[inline]
    pub(crate) fn new_unchecked(key: String) -> Self {
        debug_assert!(
            Self::validate(&key).is_ok(),
            "PlaceholderKey::new_unchecked called with invalid key: {:?}",
            key
        );
        Self(key)
    }
    #[inline]
    fn validate(s: &str) -> Result<(), PlaceholderKeyError> {
        if s.is_empty() {
            return Err(PlaceholderKeyError::Empty);
        }
        if s.contains(char::is_whitespace) {
            return Err(PlaceholderKeyError::ContainsWhitespace);
        }
        if let Some(ch) = s.chars().find(|c| Self::FORBIDDEN_CHARS.contains(c)) {
            return Err(PlaceholderKeyError::ContainsInvalidCharacter(ch));
        }
        Ok(())
    }
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        debug_assert!(
            !self.0.is_empty(),
            "PlaceholderKey invariant violated: empty key"
        );
        self.0.is_empty()
    }
}
impl fmt::Display for PlaceholderKey {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl TryFrom<String> for PlaceholderKey {
    type Error = PlaceholderKeyError;
    #[inline]
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}
#[cfg(test)]
impl From<&str> for PlaceholderKey {
    #[inline]
    fn from(s: &str) -> Self {
        Self::new(s).expect("Test used invalid PlaceholderKey")
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplacementValue {
    content: String,
    pre_escaped: bool,
}
impl ReplacementValue {
    #[inline]
    #[must_use]
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            pre_escaped: false,
        }
    }
    #[inline]
    #[must_use]
    pub fn pre_escaped(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            pre_escaped: true,
        }
    }
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.content
    }
    #[inline]
    #[must_use]
    pub fn is_pre_escaped(&self) -> bool {
        self.pre_escaped
    }
    #[inline]
    #[must_use]
    pub fn validate(&self) -> Result<(), ReplacementValueError> {
        if self.content.contains("{{") && self.content.contains("}}") {
            let bytes = self.content.as_bytes();
            for i in 0..bytes.len().saturating_sub(3) {
                if bytes[i] == b'{' && bytes.get(i + 1) == Some(&b'{') {
                    if bytes[i..].windows(2).any(|w| w == b"}}") {
                        return Err(ReplacementValueError::ContainsPlaceholderSyntax);
                    }
                }
            }
        }
        Ok(())
    }
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.content.len()
    }
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}
impl fmt::Display for ReplacementValue {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.content)
    }
}
impl From<&str> for ReplacementValue {
    #[inline]
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}
impl From<String> for ReplacementValue {
    #[inline]
    fn from(s: String) -> Self {
        Self::new(s)
    }
}
pub type PlaceholderMap = HashMap<PlaceholderKey, ReplacementValue>;
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlaceholderRegistry {
    inner: PlaceholderMap,
}
impl PlaceholderRegistry {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
        }
    }
    #[inline]
    #[must_use]
    pub fn insert(
        &mut self,
        key: impl Into<String>,
        value: impl Into<ReplacementValue>,
    ) -> Result<Option<ReplacementValue>, PlaceholderKeyError> {
        let key = PlaceholderKey::new(key.into())?;
        Ok(self.inner.insert(key, value.into()))
    }
    #[inline]
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&ReplacementValue> {
        self.inner
            .get(&PlaceholderKey::new_unchecked(key.to_owned()))
    }
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> PlaceholderMap {
        self.inner
    }
}
impl FromIterator<(String, ReplacementValue)> for PlaceholderRegistry {
    fn from_iter<T: IntoIterator<Item = (String, ReplacementValue)>>(iter: T) -> Self {
        let mut registry = Self::new();
        for (key, value) in iter {
            registry
                .insert(key, value)
                .expect("Invalid key in iterator");
        }
        registry
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn placeholder_key_valid_cases() {
        assert!(PlaceholderKey::new("abc").is_ok());
        assert!(PlaceholderKey::new("user_name_123").is_ok());
        assert!(PlaceholderKey::new("a").is_ok());
        assert!(PlaceholderKey::new("KEY-123_test").is_ok());
    }
    #[test]
    fn placeholder_key_rejects_empty() {
        assert_eq!(PlaceholderKey::new(""), Err(PlaceholderKeyError::Empty));
    }
    #[test]
    fn placeholder_key_rejects_whitespace() {
        for ws in [' ', '\t', '\n', '\r', '\x0B', '\x0C'] {
            let input = format!("key{}name", ws);
            assert_eq!(
                PlaceholderKey::new(&input),
                Err(PlaceholderKeyError::ContainsWhitespace)
            );
        }
    }
    #[test]
    fn placeholder_key_rejects_forbidden_chars() {
        for ch in PlaceholderKey::FORBIDDEN_CHARS {
            let input = format!("key{}val", ch);
            assert_eq!(
                PlaceholderKey::new(&input),
                Err(PlaceholderKeyError::ContainsInvalidCharacter(*ch))
            );
        }
    }
    #[test]
    fn replacement_value_validation() {
        assert!(ReplacementValue::new("plain text").validate().is_ok());
        assert!(
            ReplacementValue::new("{{not_a_placeholder")
                .validate()
                .is_ok()
        );
        assert!(ReplacementValue::new("placeholder}}").validate().is_ok());
        assert!(ReplacementValue::new("}} {{").validate().is_ok());
        assert_eq!(
            ReplacementValue::new("Hello {{user}}").validate(),
            Err(ReplacementValueError::ContainsPlaceholderSyntax)
        );
        assert_eq!(
            ReplacementValue::new("{{foo}} and {{bar}}").validate(),
            Err(ReplacementValueError::ContainsPlaceholderSyntax)
        );
    }
    #[test]
    fn registry_safe_insert_and_lookup() {
        let mut reg = PlaceholderRegistry::new();
        assert!(reg.insert("name", "Alice").is_ok());
        assert_eq!(reg.get("name").map(|v| v.as_str()), Some("Alice"));
        assert!(
            reg.insert("html", ReplacementValue::pre_escaped("&lt;b&gt;"))
                .is_ok()
        );
        assert!(reg.get("html").unwrap().is_pre_escaped());
        assert!(matches!(
            reg.insert("bad key", "val"),
            Err(PlaceholderKeyError::ContainsWhitespace)
        ));
        assert_eq!(reg.get("missing"), None);
        assert_eq!(reg.get("bad key"), None);
    }
    #[test]
    fn display_and_conversion() {
        let key = PlaceholderKey::new("test").unwrap();
        assert_eq!(key.to_string(), "test");
        assert_eq!(key.as_str(), "test");
        let val: ReplacementValue = "hello".into();
        assert_eq!(val.to_string(), "hello");
        assert_eq!(val.as_str(), "hello");
    }
    #[test]
    fn registry_from_iterator() {
        let items = vec![
            ("key1".to_string(), ReplacementValue::new("val1")),
            ("key2".to_string(), ReplacementValue::new("val2")),
        ];
        let reg: PlaceholderRegistry = items.into_iter().collect();
        assert_eq!(reg.len(), 2);
        assert_eq!(reg.get("key1").map(|v| v.as_str()), Some("val1"));
        assert_eq!(reg.get("key2").map(|v| v.as_str()), Some("val2"));
    }
    #[test]
    #[should_panic(expected = "Invalid key in iterator")]
    fn registry_from_iterator_panics_on_invalid_key() {
        let items = vec![("bad key".to_string(), ReplacementValue::new("val"))];
        let _reg: PlaceholderRegistry = items.into_iter().collect();
    }
}
