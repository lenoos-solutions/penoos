---
name: rustdoc-style
description: Write Penoos Rust doc comments (//! module docs, /// item docs, # Errors, # Panics, # Examples) following the Rust Book and rustdoc conventions. Use when adding or changing public items.
---

# Rustdoc style

Full policy: [rust-standards.md](../../../engineering/rust-standards.md#documentation-rules).

## Module

```rust
//! Dispatches action calls to local handlers or to a transport.
//!
//! The broker is the only public entry point for calls.
```

## Function

````rust
/// Calls an action and waits for its result.
///
/// # Errors
///
/// Returns [`PenoosError::ServiceNotFound`] when no node exposes `action_name`.
///
/// # Examples
///
/// ```no_run
/// # async fn run() -> penoos::Result<()> {
/// let broker = penoos::ServiceBroker::new()?;
/// let value = broker.call("math.add", serde_json::json!({ "a": 1, "b": 2 })).await?;
/// # Ok(())
/// # }
/// ```
pub async fn call(&self, action_name: &str, params: Value) -> Result<Value> { /* ... */ }
````

## Rules

- First line is a short summary sentence in the present tense.
- Sections in this order: `# Errors`, `# Panics`, `# Safety` (only with `unsafe`), `# Examples`.
- Link to items with intra-doc links: [`PenoosError`].
- Doc examples compile. Use `no_run` for examples that need services. Never use `ignore`.
- Do not repeat the signature in prose. Do not document private items unless the logic is non-obvious.
- Verify with `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`.
