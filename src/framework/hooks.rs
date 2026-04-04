use crate::framework::context::ProcessingContext;
use crate::handler::error::Result;
pub type HookResult = Result<()>;
#[diagnostic::on_unimplemented(
    message = "Type `{Self}` does not implement `DocxHooks`",
    label = "missing `DocxHooks` implementation"
)]
pub trait DocxHooks: Send + Sync {
    fn on_before_process(&self, _ctx: &ProcessingContext) -> HookResult {
        Ok(())
    }
    fn on_after_file(&self, _ctx: &ProcessingContext, _modified: bool) -> HookResult {
        Ok(())
    }
    fn on_before_write(&self, _ctx: &ProcessingContext) -> HookResult {
        Ok(())
    }
    fn on_after_process(&self, _ctx: &ProcessingContext) -> HookResult {
        Ok(())
    }
}
#[derive(Default, Debug, Clone, Copy)]
pub struct NoopHooks;
impl DocxHooks for NoopHooks {}
impl DocxHooks for Box<dyn DocxHooks> {
    fn on_before_process(&self, ctx: &ProcessingContext) -> HookResult {
        (**self).on_before_process(ctx)
    }
    fn on_after_file(&self, ctx: &ProcessingContext, modified: bool) -> HookResult {
        (**self).on_after_file(ctx, modified)
    }
    fn on_before_write(&self, ctx: &ProcessingContext) -> HookResult {
        (**self).on_before_write(ctx)
    }
    fn on_after_process(&self, ctx: &ProcessingContext) -> HookResult {
        (**self).on_after_process(ctx)
    }
}
pub struct CompositeHooks {
    hooks: Vec<Box<dyn DocxHooks>>,
}
impl CompositeHooks {
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }
    pub fn with_hook(mut self, hook: impl DocxHooks + 'static) -> Self {
        self.hooks.push(Box::new(hook));
        self
    }
}
impl Default for CompositeHooks {
    fn default() -> Self {
        Self::new()
    }
}
impl DocxHooks for CompositeHooks {
    fn on_before_process(&self, ctx: &ProcessingContext) -> HookResult {
        for hook in &self.hooks {
            hook.on_before_process(ctx)?;
        }
        Ok(())
    }
    fn on_after_file(&self, ctx: &ProcessingContext, modified: bool) -> HookResult {
        for hook in &self.hooks {
            hook.on_after_file(ctx, modified)?;
        }
        Ok(())
    }
    fn on_before_write(&self, ctx: &ProcessingContext) -> HookResult {
        for hook in &self.hooks {
            hook.on_before_write(ctx)?;
        }
        Ok(())
    }
    fn on_after_process(&self, ctx: &ProcessingContext) -> HookResult {
        for hook in &self.hooks {
            hook.on_after_process(ctx)?;
        }
        Ok(())
    }
}
