use crate::handler::config::Config;
use std::any::Any;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
pub struct ProcessingContext {
    config: Arc<Config>,
    pub template_path: PathBuf,
    pub output_path: PathBuf,
    current_file: String,
    user_data: Option<Box<dyn Any + Send + Sync>>,
}
impl ProcessingContext {
    #[inline]
    pub fn new(template_path: PathBuf, output_path: PathBuf, config: Arc<Config>) -> Self {
        Self {
            config,
            template_path,
            output_path,
            current_file: String::new(),
            user_data: None,
        }
    }
    #[must_use]
    pub fn with_current_file(mut self, file: impl Into<String>) -> Self {
        self.current_file = file.into();
        self
    }
    #[must_use]
    pub fn with_user_data(mut self, data: impl Any + Send + Sync + 'static) -> Self {
        self.user_data = Some(Box::new(data));
        self
    }
    #[inline]
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }
    #[inline]
    #[must_use]
    pub fn current_file(&self) -> &str {
        &self.current_file
    }
    #[inline]
    #[must_use]
    pub fn get_user_data<T: 'static>(&self) -> Option<&T> {
        self.user_data.as_ref()?.downcast_ref::<T>()
    }
}
impl fmt::Debug for ProcessingContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProcessingContext")
            .field("template_path", &self.template_path)
            .field("output_path", &self.output_path)
            .field("current_file", &self.current_file)
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}
