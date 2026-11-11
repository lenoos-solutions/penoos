# Forbidden patterns

These patterns are rejected in review and, where a lint exists, by CI. Counts
for the current code are in [health-report.md](./health-report.md).

| Pattern | Why | Use instead |
|---------|-----|-------------|
| `todo!()`, `unimplemented!()` in a reachable path | Panics at runtime (G6) | Return a `PenoosError` describing the missing feature |
| `.unwrap()`, `.expect()` in non-test library code | Panics on bad input or runtime state | `?`, `ok_or`, or a typed error |
| `panic!()` for expected failures | Callers cannot recover | `Result` |
| `println!`, `eprintln!`, `dbg!` in library code | Pollutes stdout and leaks data | `log::info!` / `debug!` / `error!` (the crate already uses `log`) |
| `unsafe` | Needs an ADR and Miri | Safe abstraction |
| `#[allow(...)]` without a reason | Hides real problems | Fix the lint or justify it on the line above |
| Holding `std::sync::Mutex` guards across `.await` | Deadlocks the runtime | `tokio::sync::Mutex` or drop the guard first |
| Blocking calls inside `async fn` (`std::thread::sleep`, blocking `std::fs`) | Starves other tasks | `tokio::time::sleep`, `tokio::fs`, `spawn_blocking` |
| `Box<dyn Error>` in library public APIs | Erases the error type | A `thiserror` enum |
| `String` where `&str` suffices in parameters | Needless allocation | `&str` |
| `static mut` | Unsound without `unsafe` | `OnceLock`, `LazyLock`, or `Mutex` |
| Commented-out code | Noise; git keeps history | Delete it |
| `TODO` without an issue number | Untracked debt | `TODO(#123): ...` or open an issue |
| Hard-coded secrets, tokens, connection strings with passwords | Security | Environment variables or config files outside git |
| Copy-pasted business rules across services | Divergence | Shared function or mixin |

## Exceptions

An exception needs a comment with the reason, and reviewer approval. Test code
may use `unwrap` and `expect` for clarity, but not `todo!()`.
