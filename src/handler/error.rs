use std::fmt;
use std::path::PathBuf;
use thiserror::Error;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    NotFound,
    InvalidFormat,
    Security,
    Io,
    Archive,
    Xml,
    Regex,
    DateTime,
    Configuration,
    ResourceLimit,
    Placeholder,
    Custom,
}
impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::NotFound => "not found",
            Self::InvalidFormat => "invalid format",
            Self::Security => "security violation",
            Self::Io => "I/O error",
            Self::Archive => "archive error",
            Self::Xml => "XML error",
            Self::Regex => "regex error",
            Self::DateTime => "date/time error",
            Self::Configuration => "configuration error",
            Self::ResourceLimit => "resource limit exceeded",
            Self::Placeholder => "placeholder error",
            Self::Custom => "custom error",
        };
        f.write_str(s)
    }
}
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum Error {
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },
    #[error("I/O error: {message}")]
    Io {
        message: String,
        #[source]
        source: std::io::Error,
    },
    #[error("File too large: {size} bytes (maximum allowed: {max} bytes)")]
    FileTooLarge { size: u64, max: u64 },
    #[error("ZIP archive error: {message}")]
    Zip {
        message: String,
        #[source]
        source: zip::result::ZipError,
    },
    #[error("Invalid DOCX format: {reason}")]
    InvalidDocx { reason: String },
    #[error("XML processing error: {details}")]
    Xml {
        details: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    },
    #[error("Placeholder not found: key='{key}' in file '{}'", file.as_deref().unwrap_or("unknown"))]
    PlaceholderNotFound { key: String, file: Option<String> },
    #[error("Placeholder substitution failed for key='{key}': {reason}")]
    PlaceholderSubstitution { key: String, reason: String },
    #[error("Configuration error: {message}")]
    Config { message: String },
    #[error("Invalid regex pattern '{pattern}': {source}")]
    InvalidRegex {
        pattern: String,
        #[source]
        source: regex::Error,
    },
    #[error("Date processing error: {message}")]
    Date { message: String },
    #[error("Time processing error: {message}")]
    Time { message: String },
    #[error("Security violation: {details}")]
    Security { details: String },
    #[error("Resource limit exceeded: {limit_type} (value: {value}, limit: {limit})")]
    ResourceLimit {
        limit_type: String,
        value: String,
        limit: String,
    },
    #[error("{message}")]
    Custom { message: String },
}
impl Error {
    #[inline]
    #[must_use]
    pub fn file_not_found(path: impl Into<PathBuf>) -> Self {
        Self::FileNotFound { path: path.into() }
    }
    #[inline]
    #[must_use]
    pub fn io(source: std::io::Error, message: impl Into<String>) -> Self {
        Self::Io {
            message: message.into(),
            source,
        }
    }
    #[inline]
    #[must_use]
    pub fn zip(source: zip::result::ZipError, message: impl Into<String>) -> Self {
        Self::Zip {
            message: message.into(),
            source,
        }
    }
    #[inline]
    #[must_use]
    pub fn xml(
        details: impl Into<String>,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::Xml {
            details: details.into(),
            source,
        }
    }
    #[inline]
    #[must_use]
    pub fn placeholder_not_found(key: impl Into<String>, file: Option<impl Into<PathBuf>>) -> Self {
        Self::PlaceholderNotFound {
            key: key.into(),
            file: file.map(|p| p.into().to_string_lossy().into_owned()),
        }
    }
    #[inline]
    #[must_use]
    pub fn placeholder_substitution(key: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::PlaceholderSubstitution {
            key: key.into(),
            reason: reason.into(),
        }
    }
    #[inline]
    #[must_use]
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }
    #[inline]
    #[must_use]
    pub fn invalid_regex(pattern: impl Into<String>, source: regex::Error) -> Self {
        Self::InvalidRegex {
            pattern: pattern.into(),
            source,
        }
    }
    #[inline]
    #[must_use]
    pub fn date(message: impl Into<String>) -> Self {
        Self::Date {
            message: message.into(),
        }
    }
    #[inline]
    #[must_use]
    pub fn time(message: impl Into<String>) -> Self {
        Self::Time {
            message: message.into(),
        }
    }
    #[inline]
    #[must_use]
    pub fn security(details: impl Into<String>) -> Self {
        Self::Security {
            details: details.into(),
        }
    }
    #[inline]
    #[must_use]
    pub fn resource_limit(
        limit_type: impl Into<String>,
        value: impl fmt::Display,
        limit: impl fmt::Display,
    ) -> Self {
        Self::ResourceLimit {
            limit_type: limit_type.into(),
            value: value.to_string(),
            limit: limit.to_string(),
        }
    }
    #[inline]
    #[must_use]
    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom {
            message: message.into(),
        }
    }
    #[inline]
    #[must_use]
    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::FileNotFound { .. } => ErrorKind::NotFound,
            Self::Io { .. } => ErrorKind::Io,
            Self::FileTooLarge { .. } => ErrorKind::ResourceLimit,
            Self::Zip { .. } => ErrorKind::Archive,
            Self::InvalidDocx { .. } => ErrorKind::InvalidFormat,
            Self::Xml { .. } => ErrorKind::Xml,
            Self::PlaceholderNotFound { .. } | Self::PlaceholderSubstitution { .. } => {
                ErrorKind::Placeholder
            }
            Self::Config { .. } => ErrorKind::Configuration,
            Self::InvalidRegex { .. } => ErrorKind::Regex,
            Self::Date { .. } | Self::Time { .. } => ErrorKind::DateTime,
            Self::Security { .. } => ErrorKind::Security,
            Self::ResourceLimit { .. } => ErrorKind::ResourceLimit,
            Self::Custom { .. } => ErrorKind::Custom,
        }
    }
    #[inline]
    #[must_use]
    pub fn is_not_found(&self) -> bool {
        matches!(self.kind(), ErrorKind::NotFound)
    }
    #[inline]
    #[must_use]
    pub fn is_invalid_format(&self) -> bool {
        matches!(
            self.kind(),
            ErrorKind::InvalidFormat | ErrorKind::Xml | ErrorKind::Regex
        )
    }
    #[inline]
    #[must_use]
    pub fn is_security(&self) -> bool {
        matches!(self.kind(), ErrorKind::Security)
    }
    #[inline]
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self.kind(),
            ErrorKind::Io | ErrorKind::Archive | ErrorKind::ResourceLimit
        )
    }
}
impl From<std::io::Error> for Error {
    #[inline]
    fn from(source: std::io::Error) -> Self {
        let message = match source.kind() {
            std::io::ErrorKind::NotFound => "File or path not found".to_string(),
            std::io::ErrorKind::PermissionDenied => "Permission denied".to_string(),
            std::io::ErrorKind::AlreadyExists => "Path already exists".to_string(),
            std::io::ErrorKind::UnexpectedEof => "Unexpected end of file".to_string(),
            _ => "I/O operation failed".to_string(),
        };
        Self::io(source, message)
    }
}
impl From<zip::result::ZipError> for Error {
    #[inline]
    fn from(source: zip::result::ZipError) -> Self {
        Self::zip(source, "ZIP archive operation failed")
    }
}
impl From<regex::Error> for Error {
    #[inline]
    fn from(source: regex::Error) -> Self {
        let pattern = source.to_string();
        Self::invalid_regex(pattern, source)
    }
}
pub trait Context<T, E> {
    fn context<C: Into<String>>(self, context: C) -> std::result::Result<T, Error>;
    fn with_context<C, F>(self, f: F) -> std::result::Result<T, Error>
    where
        C: Into<String>,
        F: FnOnce() -> C;
}
impl<T, E: Into<Error>> Context<T, E> for std::result::Result<T, E> {
    #[inline]
    fn context<C: Into<String>>(self, context: C) -> std::result::Result<T, Error> {
        self.map_err(|e| {
            let mut err = e.into();
            match &mut err {
                Error::Io { message, .. } => {
                    *message = format!("{}: {}", context.into(), message);
                }
                Error::Zip { message, .. } => {
                    *message = format!("{}: {}", context.into(), message);
                }
                Error::Xml { details, .. } => {
                    *details = format!("{}: {}", context.into(), details);
                }
                _ => {
                    return Error::custom(format!("{}: {}", context.into(), err));
                }
            }
            err
        })
    }
    #[inline]
    fn with_context<C, F>(self, f: F) -> std::result::Result<T, Error>
    where
        C: Into<String>,
        F: FnOnce() -> C,
    {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(err).context(f()),
        }
    }
}
impl<T> Context<T, Error> for Option<T> {
    #[inline]
    fn context<C: Into<String>>(self, context: C) -> std::result::Result<T, Error> {
        self.ok_or_else(|| Error::custom(context.into()))
    }
    #[inline]
    fn with_context<C, F>(self, f: F) -> std::result::Result<T, Error>
    where
        C: Into<String>,
        F: FnOnce() -> C,
    {
        self.ok_or_else(|| Error::custom(f().into()))
    }
}
pub type Result<T> = std::result::Result<T, Error>;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn error_kind_mapping() {
        assert_eq!(Error::file_not_found("x.txt").kind(), ErrorKind::NotFound);
        assert_eq!(
            Error::io(std::io::Error::last_os_error(), "read").kind(),
            ErrorKind::Io
        );
        assert_eq!(
            Error::config("missing field").kind(),
            ErrorKind::Configuration
        );
        assert_eq!(
            Error::security("path traversal attempt").kind(),
            ErrorKind::Security
        );
    }
    #[test]
    fn error_predicates() {
        let not_found = Error::file_not_found("missing.xml");
        assert!(not_found.is_not_found());
        assert!(!not_found.is_security());
        assert!(!not_found.is_retryable());
        let io_err = Error::io(
            std::io::Error::new(std::io::ErrorKind::Interrupted, "retryable"),
            "read",
        );
        assert!(io_err.is_retryable());
    }
    #[test]
    fn context_extension_result() {
        let io_err: std::result::Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "os error",
        ));
        let wrapped = io_err.context("Failed to open config");
        assert!(wrapped.is_err());
        let err = wrapped.unwrap_err();
        assert!(err.to_string().contains("Failed to open config"));
        assert!(matches!(err.kind(), ErrorKind::Io));
    }
    #[test]
    fn context_extension_option() {
        let none_err: std::result::Result<&str, Error> =
            None::<&str>.context("Expected value but got None");
        assert!(none_err.is_err());
        assert!(none_err.unwrap_err().to_string().contains("Expected value"));
    }
    #[test]
    #[allow(clippy::invalid_regex)]
    fn from_conversions() {
        let io: std::io::Error =
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let err: Error = io.into();
        assert!(matches!(err, Error::Io { .. }));
        let zip_err = zip::result::ZipError::InvalidArchive("bad zip");
        let err: Error = zip_err.into();
        assert!(matches!(err, Error::Zip { .. }));
        let regex_err = regex::Regex::new(r"[invalid").unwrap_err();
        let err: Error = regex_err.into();
        assert!(matches!(err, Error::InvalidRegex { .. }));
    }
    #[test]
    fn error_display_formatting() {
        let err = Error::FileNotFound {
            path: "/tmp/missing.docx".into(),
        };
        assert_eq!(err.to_string(), "File not found: /tmp/missing.docx");
        let err = Error::FileTooLarge {
            size: 104857600,
            max: 52428800,
        };
        assert!(err.to_string().contains("104857600"));
        assert!(err.to_string().contains("52428800"));
    }
}
