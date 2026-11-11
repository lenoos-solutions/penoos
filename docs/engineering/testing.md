# Testing

Tests follow the layout and style of the Rust Book (chapter 11). Each category
has one place and one purpose. Test names describe the behaviour under test.

## Categories

| Category | Location | Runs with | Purpose |
|----------|----------|-----------|---------|
| Unit | `src/<module>.rs` in `#[cfg(test)] mod tests` | `cargo test --lib` | One function or type in isolation |
| Integration | `tests/<area>_*.rs` | `cargo test --tests` | Public API across modules, including broker and middleware |
| End to end | `tests/e2e_*.rs` | `cargo test --test e2e_<name>` | Two or more broker nodes over a real transport |
| Documentation | `///` `# Examples` blocks | `cargo test --doc` | Public examples compile and run |
| Benchmark | `benches/*.rs` (criterion) | `cargo bench` | Performance regressions in hot paths |
| Security | `tests/security_*.rs` | `cargo test --test security_<name>` | Input validation, auth bypass, malformed packets, DoS limits |

Benchmarks run on demand, not on every push. Security tests run on every pull
request that touches `transport`, `middleware`, `serializers`, or `auth`.

## Unit tests

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

Rules:

- Name tests as sentences describing the outcome: `returns_..._when_...`.
- One behaviour per test. Use helper functions for setup, not shared mutable state.
- Do not sleep to wait for async work. Use `tokio::time::pause()` or explicit channels.

## Integration tests

Each file in `tests/` is its own crate and may only use the public API, which is
re-exported from the crate root (for example `penoos::{ServiceBroker, PenoosError}`).

```rust
// tests/middleware_order.rs
use penoos::ServiceBroker;

#[tokio::test]
async fn runs_before_call_hooks_in_registration_order() {
    // Arrange, act, and assert through the public API only.
}
```

Integration tests for G1 (middleware wiring) are required before that gap is closed.

## End to end tests

Use two brokers connected by a real transport and assert a full round trip,
including failure cases (peer down, timeout). Mark tests that need an external
service (NATS, Redis) with `#[ignore = "requires <service>"]` and run them in a
dedicated CI job that starts the service.

## Documentation tests

Every public example must compile. Use `no_run` only when the example needs
external services. Do not use `ignore`.

## Benchmarks

```rust
// benches/call_overhead.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn local_call(c: &mut Criterion) {
    c.bench_function("local_call", |b| b.iter(|| { /* ... */ }));
}

criterion_group!(benches, local_call);
criterion_main!(benches);
```

Report the baseline in the pull request and fail a change that regresses a
tracked benchmark by more than 10% without a stated reason.

## Commands

```bash
cargo test --lib                       # unit
cargo test --tests                     # integration and e2e
cargo test --doc                       # documentation
cargo test --test security_<name>      # one security suite
cargo bench                            # benchmarks
cargo test --all-features              # everything, before a release
```

## Migration

The existing `test/` (singular) directory is empty and not discovered by
Cargo. Move any future tests to `tests/`. Remove the empty `test/` and
`benchmark/` directories as part of the cleanup in [health-report.md](./health-report.md).
