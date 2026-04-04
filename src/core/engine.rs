use crate::core::parser::PlaceholderParser;
use crate::framework::context::ProcessingContext;
use crate::framework::hooks::DocxHooks;
use crate::framework::resolver::{MapResolver, NullResolver, PlaceholderResolver};
use crate::handler::config::{Config, ProcessingMode};
use crate::handler::error::{Error, Result};
use crate::handler::types::PlaceholderMap;
use path_clean::PathClean;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info, warn};
use zip::CompressionMethod;
use zip::read::ZipArchive;
use zip::write::{FileOptions, ZipWriter};
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
pub struct Engine {
    config: Arc<Config>,
    parser: PlaceholderParser,
    resolver: Box<dyn PlaceholderResolver>,
    hooks: Box<dyn DocxHooks>,
}
impl std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine")
            .field("config", &self.config)
            .field("parser", &self.parser)
            .finish()
    }
}
impl Engine {
    fn new_internal(config: Arc<Config>) -> Result<Self> {
        let parser = PlaceholderParser::new((*config).clone())?;
        Ok(Self {
            config,
            parser,
            resolver: Box::new(NullResolver),
            hooks: Box::new(crate::framework::hooks::NoopHooks),
        })
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
    pub fn process_with_mappings(
        &self,
        template_path: &Path,
        output_path: &Path,
        mappings: &PlaceholderMap,
    ) -> Result<()> {
        let resolver = MapResolver::new(mappings.clone());
        let engine = Engine {
            config: self.config.clone(),
            parser: self.parser.clone(),
            resolver: Box::new(resolver),
            hooks: Box::new(crate::framework::hooks::NoopHooks),
        };
        engine.process_with_resolver(template_path, output_path)
    }
    pub fn process_with_resolver(&self, template_path: &Path, output_path: &Path) -> Result<()> {
        info!("Processing template: {:?}", template_path);
        if !template_path.exists() {
            return Err(Error::file_not_found(template_path));
        }
        if self.config.has_security_checks() {
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
            let raw_name = entry.name().to_string();
            let safe_path = Path::new(&raw_name).clean();
            if self.config.has_security_checks() {
                self.validate_zip_path(&safe_path)?;
            }
            let file_name = safe_path.to_str().unwrap_or(&raw_name).to_string();
            let options: FileOptions<()> =
                FileOptions::default().compression_method(CompressionMethod::Deflated);
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
                        self.config.clone(),
                    )
                    .with_current_file(&file_name);
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
            let hook_ctx = ProcessingContext::new(
                ctx.template_path.clone(),
                ctx.output_path.clone(),
                self.config.clone(),
            )
            .with_current_file(&file_name);
            self.hooks.on_after_file(&hook_ctx, modified)?;
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
        let mut full_map = PlaceholderMap::new();
        for key in &placeholders {
            if let Some(value) = resolved_map.get(key) {
                full_map.insert(key.clone(), value.clone());
            } else {
                match self.config.mode() {
                    ProcessingMode::Strict => {
                        return Err(Error::placeholder_not_found(
                            key.to_string(),
                            Some(ctx.current_file()),
                        ));
                    }
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
        let max = self.config.max_file_size();
        if size > max {
            return Err(Error::FileTooLarge { size, max });
        }
        if size == 0 {
            return Err(Error::InvalidDocx {
                reason: "File is empty".into(),
            });
        }
        Ok(())
    }
    fn validate_zip_path(&self, path: &Path) -> Result<()> {
        let path_str = path.to_str().unwrap_or("");
        if path
            .components()
            .any(|c| c == std::path::Component::ParentDir)
        {
            return Err(Error::security(format!(
                "Path traversal detected: {}",
                path_str
            )));
        }
        if path.is_absolute() {
            return Err(Error::security(format!(
                "Absolute path not allowed: {}",
                path_str
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
                return Err(Error::InvalidDocx {
                    reason: format!("Missing required file: {}", required),
                });
            }
        }
        debug!("DOCX structure validation passed");
        Ok(())
    }
    pub fn config(&self) -> &Config {
        &self.config
    }
    pub fn parser(&self) -> &PlaceholderParser {
        &self.parser
    }
}
#[derive(Default)]
pub struct EngineBuilder {
    config: Option<Config>,
    resolver: Option<Box<dyn PlaceholderResolver>>,
    hooks: Option<Box<dyn DocxHooks>>,
}
impl EngineBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }
    pub fn with_resolver(mut self, resolver: impl PlaceholderResolver + 'static) -> Self {
        self.resolver = Some(Box::new(resolver));
        self
    }
    pub fn with_hooks(mut self, hooks: impl DocxHooks + 'static) -> Self {
        self.hooks = Some(Box::new(hooks));
        self
    }
    pub fn build(self) -> Result<Engine> {
        let config = self
            .config
            .map(Ok)
            .unwrap_or_else(|| Config::builder().build())?;
        config.validate()?;
        let config_arc = Arc::new(config);
        let mut engine = Engine::new_internal(config_arc)?;
        if let Some(resolver) = self.resolver {
            engine = engine.with_resolver(resolver);
        }
        if let Some(hooks) = self.hooks {
            engine = engine.with_hooks(hooks);
        }
        Ok(engine)
    }
}
