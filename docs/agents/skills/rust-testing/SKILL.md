---
name: rust-testing
description: Place and write Penoos Rust tests (unit, integration, end-to-end, doc, bench, security) using the Rust Book layout. Use for any new or changed behaviour.
---

# Rust testing

Full policy: [testing.md](../../../engineering/testing.md).

## Pick the category

| Need                      | Where                                       | Command                             |
| ------------------------- | ------------------------------------------- | ----------------------------------- |
| Single function or type   | `#[cfg(test)] mod tests` in the source file | `cargo test --lib`                  |
| Public API across modules | `tests/<area>_*.rs`                         | `cargo test --tests`                |
| Multi-node round trip     | `tests/e2e_*.rs`                            | `cargo test --test e2e_<name>`      |
| Example in docs           | `/// # Examples`                            | `cargo test --doc`                  |
| Security input            | `tests/security_*.rs`                       | `cargo test --test security_<name>` |

## Template

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Test Case: TC-SVCACT-001
    /// Test demonstrating the explicit async teardown wrapper pattern.
    #[tokio::test]
    async fn returns_service_not_found_when_action_is_missing() {
        let broker = ServiceBroker::new().expect("broker starts");
        let result = broker.call("missing.action", serde_json::json!({})).await;
        assert!(matches!(result, Err(PenoosError::ServiceNotFound { .. })));
    }
}
```

and for teardown, use `Drop` implementations or `tokio::test` cleanup code.
or use `#[tokio::test]` with a cleanup section at the end of the test.

Key Takeaways for Tokio Tests

• Use #[tokio::test(flavor = "multi_thread")] if your code relies on blocking I/O or multi-threaded scheduling during tests.
• Use tokio_util::sync::CancellationToken for graceful shutdowns instead of handle.abort() if your background task needs to flush data before exiting.

### Pattern 1: Explicit Async Teardown Wrapper (Safest)
```rust

use futures::FutureExt; // Requires the `futures` crate
use std::panic::AssertUnwindSafe;

struct TestEnv {
    // Your DB pool, server handle, etc.
}

impl TestEnv {
    async fn setup() -> Self {
        println!("Running async setup...");
        Self {}
    }

    async fn teardown(self) {
        println!("Running async teardown...");
    }
}

async fn run_test<F, Fut>(test_fn: F)
where
    F: FnOnce(&mut TestEnv) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let mut env = TestEnv::setup().await;

    // Catch panics so teardown still runs if an assert! fails
    let result = AssertUnwindSafe(test_fn(&mut env)).catch_unwind().await;

    env.teardown().await;

    if let Err(err) = result {
        std::panic::resume_unwind(err);
    }
}

/// Test Case: TC-MYASYC-001
/// Test demonstrating the explicit async teardown wrapper pattern.
#[tokio::test]
async fn my_async_test() {
    run_test(|env| async move {
        // Your test logic here
        assert_eq!(1 + 1, 2);
    }).await;
}


```

### Pattern 2: Stopping Background Tasks with Drop
```rust
use tokio::task::JoinHandle;

struct BackgroundService {
    handle: JoinHandle<()>,
}

impl BackgroundService {
    async fn start() -> Self {
        let handle = tokio::spawn(async {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });
        Self { handle }
    }
}

impl Drop for BackgroundService {
    fn drop(&mut self) {
        // Immediately cancels the background task when the test ends
        self.handle.abort();
    }
}

/// Test Case: TC-BGNWRK-001
/// Test demonstrating how to use a background service that is automatically stopped when the test ends.
#[tokio::test]
async fn test_with_background_worker() {
    let _service = BackgroundService::start().await;
    
    // Test finishes cleanly without hanging because _service drops and aborts the loop
}

```

## Rules

- Name tests as outcomes: `returns_..._when_...`.
- One behaviour per test. Arrange, act, assert.
- No sleeps. Use `tokio::time::pause()` or channels.
- External services: `#[ignore = "requires <service>"]`.
- Integration tests use the public API only, re-exported from the crate root.
- Gap-dependent tests (G1 middleware, G2 call options) are `#[ignore]` with the gap number until fixed.
