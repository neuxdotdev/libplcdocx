use crate::framework::context::ProcessingContext;
use crate::handler::error::Result;
use crate::handler::types::{PlaceholderKey, PlaceholderMap, ReplacementValue};
use std::sync::Arc;
#[diagnostic::on_unimplemented(
    message = "Type `{Self}` does not implement `PlaceholderResolver`",
    label = "missing `PlaceholderResolver` implementation",
    note = "ensure the type is Send + Sync and implements `resolve`"
)]
pub trait PlaceholderResolver: Send + Sync {
    fn resolve(
        &self,
        key: &PlaceholderKey,
        context: Option<&ProcessingContext>,
    ) -> Result<Option<ReplacementValue>>;
    fn resolve_batch(
        &self,
        keys: &[PlaceholderKey],
        context: Option<&ProcessingContext>,
    ) -> Result<PlaceholderMap> {
        let mut map = PlaceholderMap::new();
        for key in keys {
            if let Some(value) = self.resolve(key, context)? {
                map.insert(key.clone(), value);
            }
        }
        Ok(map)
    }
}
impl PlaceholderResolver for Box<dyn PlaceholderResolver> {
    fn resolve(
        &self,
        key: &PlaceholderKey,
        context: Option<&ProcessingContext>,
    ) -> Result<Option<ReplacementValue>> {
        (**self).resolve(key, context)
    }
}
impl PlaceholderResolver for Arc<dyn PlaceholderResolver> {
    fn resolve(
        &self,
        key: &PlaceholderKey,
        context: Option<&ProcessingContext>,
    ) -> Result<Option<ReplacementValue>> {
        (**self).resolve(key, context)
    }
}
impl<F> PlaceholderResolver for F
where
    F: Fn(&PlaceholderKey, Option<&ProcessingContext>) -> Result<Option<ReplacementValue>>
        + Send
        + Sync,
{
    fn resolve(
        &self,
        key: &PlaceholderKey,
        context: Option<&ProcessingContext>,
    ) -> Result<Option<ReplacementValue>> {
        self(key, context)
    }
}
pub struct MapResolver {
    map: PlaceholderMap,
}
impl MapResolver {
    pub fn new(map: PlaceholderMap) -> Self {
        Self { map }
    }
}
impl PlaceholderResolver for MapResolver {
    fn resolve(
        &self,
        key: &PlaceholderKey,
        _context: Option<&ProcessingContext>,
    ) -> Result<Option<ReplacementValue>> {
        Ok(self.map.get(key).cloned())
    }
}
pub struct NullResolver;
impl PlaceholderResolver for NullResolver {
    fn resolve(
        &self,
        _key: &PlaceholderKey,
        _context: Option<&ProcessingContext>,
    ) -> Result<Option<ReplacementValue>> {
        Ok(None)
    }
}
