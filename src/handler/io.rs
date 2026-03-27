
use crate::handler::error::{Error, Result};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use tempfile::{NamedTempFile, TempDir};
use tracing::{debug, trace};
pub fn create_temp_file(prefix: &str, suffix: &str) -> Result<NamedTempFile> {
    let dir = std::env::temp_dir();
    let mut builder = tempfile::Builder::new();
    if !prefix.is_empty() {
        builder.prefix(prefix);
    }
    if !suffix.is_empty() {
        builder.suffix(suffix);
    }
    let temp_file = builder.tempfile_in(dir)?;
    trace!("Created temporary file: {:?}", temp_file.path());
    Ok(temp_file)
}
pub fn ensure_extension(path: &Path, target_ext: &str) -> Result<PathBuf> {
    let ext = path.extension().and_then(|e| e.to_str());
    if ext == Some(target_ext) {
        return Ok(path.to_path_buf());
    }
    let new_path = path.with_extension(target_ext);
    fs::rename(path, &new_path)?;
    debug!("Renamed {:?} to {:?}", path, new_path);
    Ok(new_path)
}
pub fn read_file_to_string(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(Error::Io)
}
pub fn write_string_to_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    trace!("Written {} bytes to {:?}", content.len(), path);
    Ok(())
}
pub fn ensure_directory_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
        debug!("Created directory: {:?}", path);
    }
    Ok(())
}
pub fn safe_rename(src: &Path, dst: &Path) -> Result<()> {
    fs::rename(src, dst)?;
    debug!("Renamed {:?} -> {:?}", src, dst);
    Ok(())
}
pub fn remove_file_if_exists(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)?;
        trace!("Removed file: {:?}", path);
    }
    Ok(())
}
pub fn create_temp_dir(prefix: &str) -> Result<TempDir> {
    let dir = std::env::temp_dir();
    let builder = tempfile::Builder::new().prefix(prefix).tempdir_in(dir)?;
    debug!("Created temporary directory: {:?}", builder.path());
    Ok(builder)
}
pub fn copy_file(src: &Path, dst: &Path) -> Result<()> {
    fs::copy(src, dst)?;
    trace!("Copied {:?} -> {:?}", src, dst);
    Ok(())
}
pub fn read_lines(path: &Path) -> Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();
    for line in reader.lines() {
        lines.push(line?);
    }
    Ok(lines)
}
pub fn write_lines(path: &Path, lines: &[String]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    for line in lines {
        writeln!(writer, "{}", line)?;
    }
    writer.flush()?;
    trace!("Written {} lines to {:?}", lines.len(), path);
    Ok(())
}
pub fn is_file(path: &Path) -> bool {
    path.is_file()
}
pub fn is_directory(path: &Path) -> bool {
    path.is_dir()
}
pub fn file_size(path: &Path) -> Result<u64> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len())
}
pub fn is_empty_file(path: &Path) -> Result<bool> {
    Ok(file_size(path)? == 0)
}
pub fn append_to_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(content.as_bytes())?;
    trace!("Appended {} bytes to {:?}", content.len(), path);
    Ok(())
}
pub fn read_bytes(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(Error::Io)
}
pub fn write_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)?;
    trace!("Written {} bytes to {:?}", bytes.len(), path);
    Ok(())
}
#[cfg(unix)]
pub fn create_symlink(original: &Path, link: &Path) -> Result<()> {
    std::os::unix::fs::symlink(original, link)?;
    debug!("Created symlink {:?} -> {:?}", link, original);
    Ok(())
}
pub fn create_hardlink(original: &Path, link: &Path) -> Result<()> {
    fs::hard_link(original, link)?;
    debug!("Created hard link {:?} -> {:?}", link, original);
    Ok(())
}
pub fn canonicalize(path: &Path) -> Result<PathBuf> {
    fs::canonicalize(path).map_err(Error::Io)
}
pub fn ensure_file_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        File::create(path)?;
        trace!("Created empty file: {:?}", path);
    }
    Ok(())
}
pub fn remove_dir_all(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
        debug!("Removed directory tree: {:?}", path);
    }
    Ok(())
}