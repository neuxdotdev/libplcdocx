pub mod context;
pub mod hooks;
pub mod resolver;
pub use context::ProcessingContext;
pub use hooks::{CompositeHooks, DocxHooks, HookResult, NoopHooks};
pub use resolver::{MapResolver, NullResolver, PlaceholderResolver};
