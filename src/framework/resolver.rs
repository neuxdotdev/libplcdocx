use crate::framework::context::ProcessingContext;
use crate::handler::error::Result;
use std::collections::HashMap;
pub trait PlaceholderResolver: Send + Sync {
    fn resolve(&self, key: &str, context: Option<&ProcessingContext>) -> Result<Option<String>>;
    fn resolve_batch(
        &self,
        keys: &[String],
        context: Option<&ProcessingContext>,
    ) -> Result<HashMap<String, String>> {
        let mut map = HashMap::new();
        for key in keys {
            if let Some(value) = self.resolve(key, context)? {
                map.insert(key.clone(), value);
            }
        }
        Ok(map)
    }
}
impl PlaceholderResolver for Box<dyn PlaceholderResolver> {
    fn resolve(&self, key: &str, context: Option<&ProcessingContext>) -> Result<Option<String>> {
        (**self).resolve(key, context)
    }
}
