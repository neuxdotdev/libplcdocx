pub mod context;
pub mod hooks;
pub mod resolver;
pub use context::ProcessingContext;
pub use hooks::{DocxHooks, HookResult};
pub use resolver::PlaceholderResolver;
