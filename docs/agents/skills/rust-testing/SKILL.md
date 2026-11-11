---
name: rust-testing
description: Place and write Penoos Rust tests (unit, integration, end-to-end, doc, bench, security) using the Rust Book layout. Use for any new or changed behaviour.
---

# Rust testing

Full policy: [testing.md](../../../engineering/testing.md).

## Pick the category

| Need | Where | Command |
|------|-------|---------|
| Single function or type | `#[cfg(test)] mod tests` in the source file | `cargo test --lib` |
| Public API across modules | `tests/<area>_*.rs` | `cargo test --tests` |
| Multi-node round trip | `tests/e2e_*.rs` | `cargo test --test e2e_<name>` |
| Example in docs | `/// # Examples` | `cargo test --doc` |
| Security input | `tests/security_*.rs` | `cargo test --test security_<name>` |

## Template

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn returns_service_not_found_when_action_is_missing() {
        let broker = ServiceBroker::new().expect("broker starts");
        let result = broker.call("missing.action", serde_json::json!({})).await;
        assert!(matches!(result, Err(PenoosError::ServiceNotFound { .. })));
    }
}
```

## Rules

- Name tests as outcomes: `returns_..._when_...`.
- One behaviour per test. Arrange, act, assert.
- No sleeps. Use `tokio::time::pause()` or channels.
- External services: `#[ignore = "requires <service>"]`.
- Integration tests use the public API only, re-exported from the crate root.
- Gap-dependent tests (G1 middleware, G2 call options) are `#[ignore]` with the gap number until fixed.
