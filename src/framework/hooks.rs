use crate::framework::context::ProcessingContext;
use crate::handler::error::Result;
pub type HookResult = Result<()>;
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
#[derive(Default)]
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
