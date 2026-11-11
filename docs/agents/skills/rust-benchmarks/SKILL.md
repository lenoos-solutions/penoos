---
name: rust-benchmarks
description: Add or compare Penoos criterion benchmarks in benches/ and record baselines. Use when a change may affect hot paths such as call dispatch, serialization, or event fan-out.
---

# Rust benchmarks

Benchmarks use [criterion](https://crates.io/crates/criterion) and live in
`benches/`. The policy is in [testing.md](../../../engineering/testing.md#benchmarks).

## Add a benchmark

```rust
// benches/call_overhead.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn local_call(c: &mut Criterion) {
    c.bench_function("local_call", |b| b.iter(|| { /* ... */ }));
}

criterion_group!(benches, local_call);
criterion_main!(benches);
```

Register the file in `Cargo.toml` with `[[bench]] name = "..." harness = false`
if it is not auto-discovered.

## Compare

```bash
cargo bench -- --save-baseline before     # on the base commit
cargo bench -- --baseline before          # on the change
```

## Report

Put the before and after numbers in the pull request. A regression of more than
10% needs a reason. Do not run benchmarks in the pre-push hook; they are slow.
