use crate::handler::config::Config;
use std::path::PathBuf;
#[derive(Debug)]
pub struct ProcessingContext {
    pub template_path: PathBuf,
    pub output_path: PathBuf,
    pub config: Config,
    pub current_file: String,
    pub user_data: Option<Box<dyn std::any::Any + Send + Sync>>,
}
impl ProcessingContext {
    pub fn new(template_path: PathBuf, output_path: PathBuf, config: Config) -> Self {
        Self {
            template_path,
            output_path,
            config,
            current_file: String::new(),
            user_data: None,
        }
    }
    #[must_use]
    pub fn with_current_file(mut self, file: String) -> Self {
        self.current_file = file;
        self
    }
    #[must_use]
    pub fn with_user_data(mut self, data: impl std::any::Any + Send + Sync) -> Self {
        self.user_data = Some(Box::new(data));
        self
    }
}
