use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Invalid DOCX format: {0}")]
    InvalidDocx(String),
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[cfg(feature = "xml")]
    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("Placeholder not found: {0}")]
    PlaceholderNotFound(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Date error: {0}")]
    DateError(String),
    #[error("Time error: {0}")]
    TimeError(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("File too large: {size} bytes (max: {max} bytes)")]
    FileTooLarge { size: u64, max: u64 },
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    #[error("{0}")]
    Custom(String),
}
pub type Result<T> = std::result::Result<T, Error>;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn error_display_works() {
        let err = Error::FileNotFound("test.docx".into());
        assert_eq!(err.to_string(), "File not found: test.docx");
    }
}
