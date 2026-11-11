# Engineering health report

Baseline measured on the Penoos crate before any remediation. Re-run the commands
in this document and update the numbers when a remediation PR lands.

## Toolchain

| Tool | Version observed |
|------|------------------|
| rustc / cargo | 1.99.1 (stable) |
| rustfmt | installed |
| clippy | installed |

## Results

| Check | Command | Result | Status |
|-------|---------|--------|--------|
| Build (default features) | `cargo build` | Passes | OK |
| Unit tests | `cargo test --lib` | 96 pass, 2 ignored | OK |
| Doc tests | `cargo test --doc` | 1 pass (`no_run`) | OK |
| Rustdoc | `cargo doc --no-deps` | Builds | OK |
| Format | `cargo fmt --check` | 713 diff hunks | **Fail** |
| Clippy (lib test target) | `cargo clippy --all-targets -- -D warnings` | Hard error `approximate value of f{32,64}::consts::PI`, about 30 warnings | **Fail** |
| `tcp-transport` feature | `cargo check --features tcp-transport` | Compiles | OK |
| `nats-transport` feature | `cargo check --features nats-transport` | `cannot find type 'Connection' in module 'nats'` | **Fail** |
| No default features | `cargo check --no-default-features` | Compiles | OK |

## Forbidden-pattern counts in `src/`

Counts come from ripgrep over `src/`. The rules are in
[forbidden-patterns.md](./forbidden-patterns.md).

| Pattern | Count |
|---------|-------|
| `.unwrap()` | 200 |
| `println!(` | 39 |
| `eprintln!(` | 3 |
| `todo!(` | 6 |
| `panic!(` | 4 |
| `#[allow(` | 9 |
| `unsafe` | 0 |
| `dbg!` | 0 |
| `expect(` | 0 |

## Configuration findings

- `.editorconfig` sets `indent_style = tab` for all files, but all Rust sources use 4 spaces. See [lint-and-format.md](./lint-and-format.md#editorconfig).
- `Cargo.toml` pins `toml = "1.2.1+spec-1.1.0"`. Semver build metadata in a requirement is ignored by Cargo and produces a warning.
- `.github/workflows/*.yml` run Node.js (`npm ci`) jobs and do not check this crate. A Rust workflow is required, see [deployment.md](./deployment.md).

## Remediation plan

Each item is a separate pull request so that history stays reviewable.

| Priority | Item | Done when |
|----------|------|-----------|
| P0 | Fix the `PI` constant (use `std::f64::consts::PI`) | `cargo clippy --all-targets` has no errors |
| P0 | Run `cargo fmt` once, in a dedicated formatting-only commit | `cargo fmt --check` passes |
| P1 | Clear clippy warnings; enable `-D warnings` in CI | Clippy passes with `-D warnings` |
| P1 | Replace `todo!()` in `TcpTransport` with errors | No `todo!(` in `src/transport.rs` |
| P1 | Fix or feature-gate `nats-transport` | `cargo check --features nats-transport` passes |
| P2 | Replace `.unwrap()` in non-test code | Count in non-test code is 0, enforced by a lint |
| P2 | Remove `src/lib_old.rs`, `src/lib_new.rs`, empty placeholder dirs | No dead files in `src/` |
| P2 | Wire `MiddlewareManager` into the broker (G1) | Integration test asserts hook order |
| P3 | Delete stale Node.js workflows once Rust CI is green | Only Rust workflows remain |

Until P0 and P1 items are done, the pre-push hook fails on this repository. That
is intentional; do not weaken the hook to make it pass.
