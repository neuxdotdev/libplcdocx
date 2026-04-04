use crate::handler::error::{Context, Error, Result};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use tempfile::{NamedTempFile, TempDir};
use tracing::{debug, trace};
pub const DEFAULT_MAX_READ_SIZE: u64 = 50 * 1024 * 1024;
pub const DEFAULT_MAX_WRITE_SIZE: u64 = 100 * 1024 * 1024;
#[inline]
#[must_use]
pub fn create_temp_file(prefix: &str, suffix: &str) -> Result<NamedTempFile> {
    let dir = std::env::temp_dir();
    let mut builder = tempfile::Builder::new();
    if !prefix.is_empty() {
        builder.prefix(prefix);
    }
    if !suffix.is_empty() {
        builder.suffix(suffix);
    }
    builder
        .tempfile_in(&dir)
        .context("Failed to create temporary file in system temp directory")
}
#[inline]
#[must_use]
pub fn create_temp_dir(prefix: &str) -> Result<TempDir> {
    let dir = std::env::temp_dir();
    tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(&dir)
        .context("Failed to create temporary directory in system temp directory")
}
#[inline]
#[must_use]
pub fn ensure_extension(path: &Path, target_ext: &str) -> Result<PathBuf> {
    if path.extension().and_then(|e| e.to_str()) == Some(target_ext) {
        return Ok(path.to_path_buf());
    }
    let new_path = path.with_extension(target_ext);
    if fs::rename(path, &new_path).is_err() {
        fs::copy(path, &new_path).context(format!(
            "Failed to copy file during extension change from {:?} to {:?}",
            path, new_path
        ))?;
        fs::remove_file(path).context(format!(
            "Failed to remove original file {:?} after extension change copy",
            path
        ))?;
    }
    debug!("Ensured extension for {:?} -> {:?}", path, new_path);
    Ok(new_path)
}
#[inline]
#[must_use]
pub fn canonicalize(path: &Path) -> Result<PathBuf> {
    fs::canonicalize(path).context(format!("Failed to canonicalize path: {:?}", path))
}
#[inline]
#[must_use]
pub fn read_file_to_string(path: &Path, max_size: u64) -> Result<String> {
    let file = File::open(path).context(format!("Failed to open file for reading: {:?}", path))?;
    let meta = file
        .metadata()
        .context(format!("Failed to read metadata for file: {:?}", path))?;
    if meta.len() > max_size {
        return Err(Error::resource_limit("file size", meta.len(), max_size));
    }
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    reader
        .read_to_string(&mut content)
        .context(format!("Failed to read string content from {:?}", path))?;
    Ok(content)
}
#[inline]
#[must_use]
pub fn read_bytes(path: &Path, max_size: u64) -> Result<Vec<u8>> {
    let file =
        File::open(path).context(format!("Failed to open file for reading bytes: {:?}", path))?;
    let meta = file
        .metadata()
        .context(format!("Failed to read metadata for file: {:?}", path))?;
    if meta.len() > max_size {
        return Err(Error::resource_limit("file size", meta.len(), max_size));
    }
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::with_capacity(meta.len() as usize);
    reader
        .read_to_end(&mut buffer)
        .context(format!("Failed to read bytes from {:?}", path))?;
    Ok(buffer)
}
#[inline]
#[must_use]
pub fn read_lines(path: &Path, max_size: u64) -> Result<Vec<String>> {
    let file =
        File::open(path).context(format!("Failed to open file for reading lines: {:?}", path))?;
    let meta = file.metadata().context("Failed to read file metadata")?;
    if meta.len() > max_size {
        return Err(Error::resource_limit("file size", meta.len(), max_size));
    }
    let reader = BufReader::new(file);
    reader
        .lines()
        .collect::<io::Result<Vec<_>>>()
        .context("Failed to read lines from file")
}
#[inline]
pub fn write_string_to_file_atomic(path: &Path, content: &str) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .context(format!("Failed to create parent directory: {:?}", parent))?;
    let mut tmp = NamedTempFile::new_in(parent)
        .context("Failed to create temporary file for atomic write")?;
    tmp.write_all(content.as_bytes())
        .context("Failed to write to temporary file")?;
    tmp.persist(path).map_err(|e| {
        Error::io(
            e.error,
            format!("Failed to atomically persist file to {:?}", path),
        )
    })?;
    trace!("Atomically written {} bytes to {:?}", content.len(), path);
    Ok(())
}
#[inline]
pub fn write_bytes_to_file_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .context(format!("Failed to create parent directory: {:?}", parent))?;
    let mut tmp = NamedTempFile::new_in(parent)
        .context("Failed to create temporary file for atomic write")?;
    tmp.write_all(bytes)
        .context("Failed to write bytes to temporary file")?;
    tmp.persist(path).map_err(|e| {
        Error::io(
            e.error,
            format!("Failed to atomically persist file to {:?}", path),
        )
    })?;
    trace!("Atomically written {} bytes to {:?}", bytes.len(), path);
    Ok(())
}
#[inline]
pub fn write_string_to_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .context(format!("Failed to create parent directory: {:?}", parent))?;
    }
    let mut file = File::create(path).context(format!("Failed to create file: {:?}", path))?;
    file.write_all(content.as_bytes())
        .context(format!("Failed to write content to {:?}", path))?;
    trace!("Written {} bytes to {:?}", content.len(), path);
    Ok(())
}
#[inline]
pub fn append_to_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .context(format!("Failed to create parent directory: {:?}", parent))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .context(format!("Failed to open file for appending: {:?}", path))?;
    file.write_all(content.as_bytes())
        .context(format!("Failed to append content to {:?}", path))?;
    trace!("Appended {} bytes to {:?}", content.len(), path);
    Ok(())
}
#[inline]
pub fn write_lines(path: &Path, lines: &[String]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .context(format!("Failed to create parent directory: {:?}", parent))?;
    }
    let file = File::create(path).context(format!(
        "Failed to create file for writing lines: {:?}",
        path
    ))?;
    let mut writer = BufWriter::new(file);
    for line in lines {
        writeln!(writer, "{}", line).context("Failed to write line to buffer")?;
    }
    writer.flush().context("Failed to flush buffered writer")?;
    trace!("Written {} lines to {:?}", lines.len(), path);
    Ok(())
}
#[inline]
pub fn ensure_directory_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path).context(format!("Failed to create directory: {:?}", path))?;
        debug!("Created directory: {:?}", path);
    }
    Ok(())
}
#[inline]
pub fn ensure_file_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context(format!(
                "Failed to create parent dir for file: {:?}",
                parent
            ))?;
        }
        File::create(path).context(format!("Failed to create empty file: {:?}", path))?;
        trace!("Created empty file: {:?}", path);
    }
    Ok(())
}
#[inline]
pub fn safe_rename(src: &Path, dst: &Path) -> Result<()> {
    if fs::rename(src, dst).is_err() {
        debug!(
            "Rename failed (cross-device?), falling back to copy + delete for {:?} -> {:?}",
            src, dst
        );
        fs::copy(src, dst).context(format!(
            "Fallback copy failed during rename {:?} -> {:?}",
            src, dst
        ))?;
        fs::remove_file(src).context(format!(
            "Fallback remove original failed during rename {:?}",
            src
        ))?;
    }
    debug!("Renamed {:?} -> {:?}", src, dst);
    Ok(())
}
#[inline]
pub fn copy_file(src: &Path, dst: &Path) -> Result<()> {
    if dst.exists() {
        return Err(Error::custom(format!(
            "Destination already exists: {:?}",
            dst
        )));
    }
    fs::copy(src, dst).context(format!("Failed to copy {:?} -> {:?}", src, dst))?;
    trace!("Copied {:?} -> {:?}", src, dst);
    Ok(())
}
#[inline]
pub fn remove_file_if_exists(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path).context(format!("Failed to remove file: {:?}", path))?;
        trace!("Removed file: {:?}", path);
    }
    Ok(())
}
#[inline]
pub fn remove_dir_all(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path).context(format!("Failed to remove directory tree: {:?}", path))?;
        debug!("Removed directory tree: {:?}", path);
    }
    Ok(())
}
#[inline]
#[must_use]
pub fn file_size(path: &Path) -> Result<u64> {
    fs::metadata(path)
        .context(format!("Failed to read metadata: {:?}", path))
        .map(|m| m.len())
}
#[inline]
#[must_use]
pub fn is_empty_file(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    file_size(path).map(|size| size == 0)
}
#[inline]
#[must_use]
pub fn is_file(path: &Path) -> bool {
    path.is_file()
}
#[inline]
#[must_use]
pub fn is_directory(path: &Path) -> bool {
    path.is_dir()
}
#[cfg(unix)]
#[inline]
pub fn create_symlink(original: &Path, link: &Path) -> Result<()> {
    std::os::unix::fs::symlink(original, link).context(format!(
        "Failed to create symlink {:?} -> {:?}",
        link, original
    ))?;
    debug!("Created symlink {:?} -> {:?}", link, original);
    Ok(())
}
#[inline]
pub fn create_hardlink(original: &Path, link: &Path) -> Result<()> {
    fs::hard_link(original, link).context(format!(
        "Failed to create hardlink {:?} -> {:?}",
        link, original
    ))?;
    debug!("Created hard link {:?} -> {:?}", link, original);
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn temp_file_creation_and_cleanup() {
        let tmp = create_temp_file("test_", ".tmp").unwrap();
        assert!(tmp.path().exists());
        let path = tmp.path().to_path_buf();
        drop(tmp);
        assert!(!path.exists());
    }
    #[test]
    fn atomic_write_prevents_partial_state() {
        let dir = create_temp_dir("atomic_test_").unwrap();
        let target = dir.path().join("config.json");
        write_string_to_file_atomic(&target, r#"{"valid": true}"#).unwrap();
        assert!(target.exists());
        let content = std::fs::read_to_string(&target).unwrap();
        assert_eq!(content, r#"{"valid": true}"#);
    }
    #[test]
    fn read_with_size_limit_blocks_oversized() {
        let dir = create_temp_dir("size_test_").unwrap();
        let file = dir.path().join("big.txt");
        std::fs::write(&file, "A".repeat(102_400)).unwrap();
        assert!(read_file_to_string(&file, 50 * 1024).is_err());
        assert!(read_file_to_string(&file, 200 * 1024).is_ok());
    }
    #[test]
    fn ensure_extension_cross_device_safe() {
        let dir = create_temp_dir("ext_test_").unwrap();
        let old = dir.path().join("data.txt");
        std::fs::write(&old, "data").unwrap();
        let new = ensure_extension(&old, "bak").unwrap();
        assert!(new.ends_with("data.bak"));
        assert!(!old.exists());
        assert_eq!(std::fs::read_to_string(&new).unwrap(), "data");
    }
    #[test]
    fn safe_copy_fails_on_existing_destination() {
        let dir = create_temp_dir("copy_test_").unwrap();
        let src = dir.path().join("src.txt");
        let dst = dir.path().join("dst.txt");
        std::fs::write(&src, "source").unwrap();
        std::fs::write(&dst, "existing").unwrap();
        assert!(copy_file(&src, &dst).is_err());
        assert_eq!(std::fs::read_to_string(&dst).unwrap(), "existing");
    }
}
