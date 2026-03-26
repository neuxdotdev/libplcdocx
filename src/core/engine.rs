use crate::core::parser::PlaceholderParser;
use crate::framework::context::ProcessingContext;
use crate::framework::hooks::{DocxHooks, NoopHooks};
use crate::framework::resolver::PlaceholderResolver;
use crate::handler::config::{Config, ProcessingMode};
use crate::handler::error::{Error, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use tracing::{debug, info, warn};
use zip::read::ZipArchive;
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;
const DOCX_TEXT_FILES: &[&str] = &[
    "word/document.xml",
    "word/header1.xml",
    "word/header2.xml",
    "word/header3.xml",
    "word/footer1.xml",
    "word/footer2.xml",
    "word/footer3.xml",
    "word/comments.xml",
    "word/footnotes.xml",
    "word/endnotes.xml",
];
struct HashMapResolver {
    map: HashMap<String, String>,
}
impl PlaceholderResolver for HashMapResolver {
    fn resolve(&self, key: &str, _context: Option<&ProcessingContext>) -> Result<Option<String>> {
        Ok(self.map.get(key).cloned())
    }
}
#[derive(Default)]
struct NullResolver;
impl PlaceholderResolver for NullResolver {
    fn resolve(&self, _key: &str, _context: Option<&ProcessingContext>) -> Result<Option<String>> {
        Ok(None)
    }
}
pub struct Engine {
    config: Config,
    parser: PlaceholderParser,
    resolver: Box<dyn PlaceholderResolver>,
    hooks: Box<dyn DocxHooks>,
}
impl std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine")
            .field("config", &self.config)
            .field("parser", &self.parser)
            .field("resolver", &"<dyn PlaceholderResolver>")
            .field("hooks", &"<dyn DocxHooks>")
            .finish()
    }
}
impl Engine {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self {
            config: config.clone(),
            parser: PlaceholderParser::new(config),
            resolver: Box::new(NullResolver::default()),
            hooks: Box::new(NoopHooks),
        }
    }
    #[must_use]
    pub fn with_resolver(mut self, resolver: impl PlaceholderResolver + 'static) -> Self {
        self.resolver = Box::new(resolver);
        self
    }
    #[must_use]
    pub fn with_hooks(mut self, hooks: impl DocxHooks + 'static) -> Self {
        self.hooks = Box::new(hooks);
        self
    }
    pub fn process(
        &self,
        template_path: &Path,
        output_path: &Path,
        mappings: &HashMap<String, String>,
    ) -> Result<()> {
        let resolver = HashMapResolver {
            map: mappings.clone(),
        };
        let engine = Engine {
            config: self.config.clone(),
            parser: self.parser.clone(),
            resolver: Box::new(resolver),
            hooks: Box::new(NoopHooks),
        };
        engine.process_with_resolver(template_path, output_path)
    }
    pub fn process_with_resolver(&self, template_path: &Path, output_path: &Path) -> Result<()> {
        info!("Processing template: {:?}", template_path);
        if !template_path.exists() {
            return Err(Error::FileNotFound(template_path.display().to_string()));
        }
        if self.config.security_checks {
            self.check_file_size(template_path)?;
            self.validate_docx(template_path)?;
        }
        let ctx = ProcessingContext::new(
            template_path.to_path_buf(),
            output_path.to_path_buf(),
            self.config.clone(),
        );
        self.hooks.on_before_process(&ctx)?;
        let input_file = File::open(template_path)?;
        let mut archive = ZipArchive::new(BufReader::new(input_file))?;
        let output_file = File::create(output_path)?;
        let mut writer = ZipWriter::new(BufWriter::new(output_file));
        let (processed_files, total_placeholders) =
            self.process_archive(&mut archive, &mut writer, &ctx)?;
        self.hooks.on_before_write(&ctx)?;
        writer.finish()?;
        self.hooks.on_after_process(&ctx)?;
        info!(
            "Output written to: {:?} ({} files processed, {} placeholders replaced)",
            output_path, processed_files, total_placeholders
        );
        Ok(())
    }
    fn process_archive(
        &self,
        archive: &mut ZipArchive<BufReader<File>>,
        writer: &mut ZipWriter<BufWriter<File>>,
        ctx: &ProcessingContext,
    ) -> Result<(usize, usize)> {
        let mut processed_files = 0;
        let mut total_placeholders = 0;
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            let file_name = entry.name().to_string();
            if self.config.security_checks {
                self.validate_zip_path(&file_name)?;
            }
            let options: FileOptions<'_, ()> = FileOptions::default().compression_method(CompressionMethod::Deflated);
            writer.start_file(&file_name, options)?;
            let mut modified = false;
            if self.should_process_file(&file_name) {
                let mut contents = String::new();
                entry.read_to_string(&mut contents)?;
                let placeholder_count = self.parser.count(&contents);
                if placeholder_count > 0 {
                    let file_ctx = ProcessingContext::new(
                        ctx.template_path.clone(),
                        ctx.output_path.clone(),
                        ctx.config.clone(),
                    ).with_current_file(file_name.clone());
                    let resolved = self.replace_placeholders_dynamic(&contents, &file_ctx)?;
                    writer.write_all(resolved.as_bytes())?;
                    total_placeholders += placeholder_count;
                    modified = true;
                    debug!(
                        "Processed {} ({} placeholders)",
                        file_name, placeholder_count
                    );
                } else {
                    writer.write_all(contents.as_bytes())?;
                }
                processed_files += 1;
            } else {
                let mut buffer = Vec::new();
                entry.read_to_end(&mut buffer)?;
                writer.write_all(&buffer)?;
            }
            let file_ctx = ProcessingContext::new(
                ctx.template_path.clone(),
                ctx.output_path.clone(),
                ctx.config.clone(),
            ).with_current_file(file_name);
            self.hooks.on_after_file(&file_ctx, modified)?;
        }
        Ok((processed_files, total_placeholders))
    }
    fn replace_placeholders_dynamic(
        &self,
        content: &str,
        ctx: &ProcessingContext,
    ) -> Result<String> {
        let placeholders = self.parser.find_all(content);
        if placeholders.is_empty() {
            return Ok(content.to_string());
        }
        let resolved_map = self.resolver.resolve_batch(&placeholders, Some(ctx))?;
        let mut full_map = HashMap::new();
        for key in &placeholders {
            if let Some(value) = resolved_map.get(key) {
                full_map.insert(key.clone(), value.clone());
            } else {
                match self.config.mode {
                    ProcessingMode::Strict => return Err(Error::PlaceholderNotFound(key.clone())),
                    ProcessingMode::Warn => warn!("Unmapped placeholder: {}", key),
                    ProcessingMode::Lenient => {}
                }
            }
        }
        self.parser.replace_all(content, &full_map)
    }
    fn should_process_file(&self, file_name: &str) -> bool {
        DOCX_TEXT_FILES.iter().any(|f| file_name.ends_with(f))
    }
    fn check_file_size(&self, path: &Path) -> Result<()> {
        let metadata = std::fs::metadata(path)?;
        let size = metadata.len();
        if size > self.config.max_file_size {
            return Err(Error::FileTooLarge {
                size,
                max: self.config.max_file_size,
            });
        }
        if size == 0 {
            return Err(Error::InvalidDocx("File is empty".into()));
        }
        Ok(())
    }
    fn validate_zip_path(&self, path: &str) -> Result<()> {
        if path.contains("..") {
            return Err(Error::SecurityViolation(format!(
                "Path traversal detected: {}",
                path
            )));
        }
        if path.starts_with('/') || path.starts_with('\\') {
            return Err(Error::SecurityViolation(format!(
                "Absolute path not allowed: {}",
                path
            )));
        }
        Ok(())
    }
    fn validate_docx(&self, path: &Path) -> Result<()> {
        let file = File::open(path)?;
        let mut archive = ZipArchive::new(BufReader::new(file))?;
        let required_files = ["word/document.xml", "[Content_Types].xml"];
        for required in required_files.iter() {
            if archive.by_name(required).is_err() {
                return Err(Error::InvalidDocx(format!(
                    "Missing required file: {}",
                    required
                )));
            }
        }
        debug!("DOCX structure validation passed");
        Ok(())
    }
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }
    #[must_use]
    pub fn parser(&self) -> &PlaceholderParser {
        &self.parser
    }
}
#[derive(Default)]
pub struct EngineBuilder {
    config: Config,
    resolver: Option<Box<dyn PlaceholderResolver>>,
    hooks: Option<Box<dyn DocxHooks>>,
}
impl std::fmt::Debug for EngineBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineBuilder")
            .field("config", &self.config)
            .field("resolver", &"<dyn PlaceholderResolver>")
            .field("hooks", &"<dyn DocxHooks>")
            .finish()
    }
}
impl EngineBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }
    #[must_use]
    pub fn with_resolver(mut self, resolver: impl PlaceholderResolver + 'static) -> Self {
        self.resolver = Some(Box::new(resolver));
        self
    }
    #[must_use]
    pub fn with_hooks(mut self, hooks: impl DocxHooks + 'static) -> Self {
        self.hooks = Some(Box::new(hooks));
        self
    }
    #[must_use]
    pub fn with_logging(mut self, enabled: bool) -> Self {
        self.config.logging = enabled;
        self
    }
    #[must_use]
    pub fn with_mode(mut self, mode: ProcessingMode) -> Self {
        self.config.mode = mode;
        self
    }
    #[must_use]
    pub fn with_security_checks(mut self, enabled: bool) -> Self {
        self.config.security_checks = enabled;
        self
    }
    #[must_use]
    pub fn with_max_file_size(mut self, size: u64) -> Self {
        self.config.max_file_size = size;
        self
    }
    #[must_use]
    pub fn build(self) -> Engine {
        let mut engine = Engine::new(self.config);
        if let Some(resolver) = self.resolver {
            engine = engine.with_resolver(resolver);
        }
        if let Some(hooks) = self.hooks {
            engine = engine.with_hooks(hooks);
        }
        engine
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn builder_works() {
        let engine = EngineBuilder::new()
            .with_logging(true)
            .with_security_checks(false)
            .build();
        assert!(engine.config().logging);
        assert!(!engine.config().security_checks);
    }
    #[test]
    fn validate_zip_path_rejects_traversal() {
        let engine = Engine::new(Config::default());
        assert!(engine.validate_zip_path("word/document.xml").is_ok());
        assert!(engine.validate_zip_path("../etc/passwd").is_err());
        assert!(engine.validate_zip_path("/absolute/path").is_err());
    }
}
