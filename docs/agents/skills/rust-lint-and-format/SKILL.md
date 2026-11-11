---
name: rust-lint-and-format
description: Run and fix rustfmt, clippy (-D warnings), and rustdoc checks for the Penoos crate. Use when formatting, lint errors, or CI lint failures come up.
---

# Rust lint and format

Full policy: [lint-and-format.md](../../../engineering/lint-and-format.md).

## Run

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

## Fix order

1. Run `cargo fmt --all` first. Commit formatting-only changes separately (`style:`).
2. Fix clippy errors before warnings. Fix the root cause; do not add `#[allow]`.
3. Known fix: `clippy::approx_constant` for PI. Use `std::f64::consts::PI`.
4. For `#[allow(...)]` you must keep, add a justification on the line above.

## Do not

- Add `#[rustfmt::skip]` or `#![allow(clippy::all)]`.
- Mix formatting and logic changes in one commit.
- Silence `unwrap_used` or `todo` lints at module level.
