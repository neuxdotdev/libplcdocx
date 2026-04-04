# libplcdocx

> **A robust, extensible Rust framework for processing DOCX templates via placeholder replacement.**

<p align="center">
  <a href="https://crates.io/crates/libplcdocx"><img src="https://img.shields.io/crates/v/libplcdocx.svg?style=flat-square&logo=rust" alt="Crates.io"></a>
  <a href="https://docs.rs/libplcdocx"><img src="https://img.shields.io/docsrs/libplcdocx?style=flat-square&logo=docs.rs" alt="Docs.rs"></a>
  <a href="https://github.com/neuxdotdev/libplcdocx/blob/main/license"><img src="https://img.shields.io/github/license/neuxdotdev/libplcdocx?style=flat-square" alt="License"></a>
  <a href="https://github.com/neuxdotdev/libplcdocx/actions"><img src="https://img.shields.io/github/actions/workflow/status/neuxdotdev/libplcdocx/rust.yml?branch=main&style=flat-square&logo=github" alt="Build"></a>
  <a href="https://blog.rust-lang.org/2024/11/28/Rust-1.83.0.html"><img src="https://img.shields.io/badge/Rust-1.83%2B-orange?style=flat-square&logo=rust" alt="Rust Version"></a>
  <a href="https://www.contributor-covenant.org/version/2/1/code_of_conduct/"><img src="https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=flat-square" alt="Code of Conduct"></a>
</p>

---

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
  - [Placeholder Syntax](#placeholder-syntax)
  - [Processing Modes](#processing-modes)
  - [Security Settings](#security-settings)
- [Library API](#library-api)
  - [Engine & EngineBuilder](#engine--enginebuilder)
  - [PlaceholderMap & Types](#placeholdermap--types)
  - [Resolver Trait](#resolver-trait)
  - [Hooks System](#hooks-system)
- [Error Handling](#error-handling)
- [Utilities](#utilities)
- [Project Structure](#project-structure)
- [Contributing](#contributing)
- [License](#license)

---

## Features

- **DOCX Processing**: Reads `.docx` files (ZIP archives) and replaces placeholders in XML text files (document.xml, headers, footers, etc.).
- **Type-Safe Placeholders**: Strongly typed `PlaceholderKey` and `ReplacementValue` prevent runtime errors.
- **XML Safety**: Automatic XML escaping (`&`, `<`, `>`, `"`, `'`) for safe document generation.
- **Extensible Architecture**: Implement `PlaceholderResolver` for custom data sources (database, API) or `DocxHooks` for lifecycle events.
- **Security First**: Built-in protections against path traversal (Zip Slip), file size limits, and empty file checks.
- **Flexible Configuration**: Fluent builder pattern for `Config` and `Engine`. Supports strict, warning, and lenient processing modes.
- **Indonesian Locale Support**: Built-in utilities for formatting dates (e.g., "26 Maret 2026") and time ranges.

---

## Installation

Run Following Command

```sh
cargo add libplcdocx
```

**Optional Features:**

- `xml`: Enables `quick-xml` dependencies for advanced XML manipulation.
- `config-helpers`: Enables helper functions for configuration loading (uses `dirs`).
- `debug`: Enables maximum tracing level for detailed logs.

```sh
cargo add libplcdocx --features debug
```

---

## Quick Start

This example demonstrates how to create an engine, configure it, and process a document with a simple map.

```rust
use libplcdocx::{
    Engine, Config, ProcessingMode, PlaceholderSyntax,
    PlaceholderKey, PlaceholderMap, ReplacementValue
};
use std::path::Path;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup Configuration
    let config = Config::builder()
        .syntax(PlaceholderSyntax::new("{{", "}}", '\\')?) // Custom syntax {{key}}
        .mode(ProcessingMode::Strict) // Fail if a key is missing
        .security_checks(true)
        .build()?;

    // 2. Build the Engine
    // Engine requires Arc<Config> for thread-safe sharing internally
    let engine = Engine::builder()
        .with_config(config)
        .build()?;

    // 3. Prepare Mappings
    let mut map = PlaceholderMap::new();
    // PlaceholderKey validates the key string (no spaces, forbidden chars)
    map.insert(
        PlaceholderKey::new("name")?,
        ReplacementValue::new("John Doe")
    );
    map.insert(
        PlaceholderKey::new("date")?,
        ReplacementValue::new("26/03/2026")
    );

    // 4. Process the Template
    engine.process_with_mappings(
        Path::new("template.docx"),
        Path::new("output.docx"),
        &map
    )?;

    println!("Document processed successfully.");
    Ok(())
}
```

---

## Configuration

Configuration is handled via the `Config::builder()` pattern.

### Placeholder Syntax

Define the delimiters for your placeholders. The default is `[[%%...%%]]`.

```rust
let syntax = PlaceholderSyntax::new("{{", "}}", '\\')?;
let config = Config::builder().syntax(syntax).build()?;
```

### Processing Modes

Control how the engine handles missing placeholders.

| Mode        | Behavior                                                   |
| :---------- | :--------------------------------------------------------- |
| **Lenient** | Missing placeholders are ignored (left as is).             |
| **Warn**    | Missing placeholders are logged as warnings via `tracing`. |
| **Strict**  | Returns an `Error::PlaceholderNotFound` immediately.       |

```rust
config.with_mode(ProcessingMode::Strict);
```

### Security Settings

The engine includes safety features to prevent common vulnerabilities.

- `max_file_size`: Limits template file size (default: 50MB).
- `security_checks`: Enables path traversal detection and DOCX structure validation.

```rust
let config = Config::builder()
    .max_file_size(10 * 1024 * 1024) // 10 MB limit
    .security_checks(true)
    .build()?;
```

---

## Library API

### Engine & EngineBuilder

The `Engine` is the primary entry point. Use `EngineBuilder` to construct it.

```rust
let engine = Engine::builder()
    .with_config(config)
    .with_resolver(MyCustomResolver) // impl PlaceholderResolver
    .with_hooks(MyCustomHooks)       // impl DocxHooks
    .build()?;
```

### PlaceholderMap & Types

`PlaceholderMap` is a type alias for `HashMap<PlaceholderKey, ReplacementValue>`.

- **`PlaceholderKey`**: Validates keys (no whitespace, no forbidden chars like `{`, `}`, `<`).
- **`ReplacementValue`**: Holds the string content.
  - `ReplacementValue::new("text")`: Will be XML escaped (`&` -> `&amp;`).
  - `ReplacementValue::pre_escaped("<b>text</b>")`: Bypasses escaping (use for HTML injection).

### Resolver Trait

Implement `PlaceholderResolver` to fetch data dynamically from databases or APIs.

```rust
use libplcdocx::{PlaceholderResolver, ProcessingContext, PlaceholderKey, ReplacementValue, Result};

struct DatabaseResolver;

impl PlaceholderResolver for DatabaseResolver {
    fn resolve(
        &self,
        key: &PlaceholderKey,
        _ctx: Option<&ProcessingContext>
    ) -> Result<Option<ReplacementValue>> {
        // Mock database lookup
        match key.as_str() {
            "user_id" => Ok(Some(ReplacementValue::new("12345"))),
            _ => Ok(None),
        }
    }
}
```

### Hooks System

Use `DocxHooks` to intercept the processing lifecycle for logging, metrics, or validation.

```rust
use libplcdocx::{DocxHooks, ProcessingContext, HookResult};

struct MetricsHook;

impl DocxHooks for MetricsHook {
    fn on_before_process(&self, ctx: &ProcessingContext) -> HookResult {
        println!("Starting: {:?}", ctx.template_path);
        Ok(())
    }

    fn on_after_file(&self, ctx: &ProcessingContext, modified: bool) -> HookResult {
        if modified {
            println!("Modified file: {}", ctx.current_file());
        }
        Ok(())
    }
}
```

---

## Error Handling

Errors are consolidated in `handler::error::Error`. The library uses `thiserror` for precise error definitions.

- `FileNotFound`: Template path does not exist.
- `InvalidDocx`: Malformed DOCX structure (missing required files).
- `PlaceholderNotFound`: Missing mapping in Strict mode.
- `SecurityViolation`: Path traversal attempt detected.
- `ResourceLimit`: File too large or too many placeholders.

All operations return `Result<T, libplcdocx::Error>`.

---

## Utilities

The library provides helper functions for Indonesian locale formatting in `handler::utils`.

```rust
use libplcdocx::utils::{format_date_indonesia, format_time_range, escape_xml};

// Date Formatting: "26/03/2026" -> "26 Maret 2026"
let date_str = format_date_indonesia("26/03/2026")?;

// Time Range: "10-14" -> "10:00 - 14:00 WIB"
let time_str = format_time_range("10-14", "default")?;

// XML Escaping
let safe_xml = escape_xml("Tom & Jerry <cat>");
// Result: "Tom &amp; Jerry &lt;cat&gt;"
```

---

## Project Structure

```
.
├── src
│   ├── core           # Core logic: Engine & Parser
│   ├── framework      # Extensibility: Hooks, Resolvers, Context
│   ├── handler        # Infrastructure: Config, Errors, Types, I/O, Utils
│   └── lib.rs         # Public API exports
├── Cargo.toml
└── readme.md
```

---

## Contributing

We welcome contributions! Please follow these steps:

1. Fork the repository.
2. Create a feature branch (`git checkout -b feat/amazing-feature`).
3. Ensure code passes `cargo test` and `cargo clippy`.
4. Commit changes with conventional commit messages.
5. Open a Pull Request.

**Code Style**: This project uses `#![forbid(unsafe_code)]` and standard Rust formatting guidelines.

---

## License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.

---

> **Repository**: https://github.com/neuxdotdev/libplcdocx  
> **Crates.io**: https://crates.io/crates/libplcdocx  
> **Documentation**: https://docs.rs/libplcdocx

```

```
