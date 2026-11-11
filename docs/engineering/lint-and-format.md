# Lint and format

Formatting and linting are enforced by the pre-push hook and by CI. Passing them
locally is required before a pull request is opened.

## Commands

```bash
cargo fmt --all                                   # format
cargo fmt --all -- --check                        # verify formatting (CI)
cargo clippy --all-targets --all-features -- -D warnings
cargo doc --no-deps --all-features                # rustdoc must build without warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps    # treat rustdoc warnings as errors
```

## rustfmt

Use the default rustfmt configuration unless a `rustfmt.toml` is added through
an ADR. Do not add `#[rustfmt::skip]` without a comment that says why.

## Clippy

- Run clippy with `-D warnings`.
- Do not add `#[allow(clippy::...)]` without a justification comment on the line above.
- `#[allow(...)]` attributes are counted and must not grow. Each new one needs reviewer approval.
- Fix the root cause before allowing a lint. Known example: `clippy::approx_constant` for `PI` must be solved with `std::f64::consts::PI`, not an allow.

## Recommended lint set

Enable these in `Cargo.toml` as the codebase is cleaned up (tracked in
[health-report.md](./health-report.md)):

```toml
[lints.rust]
unsafe_code = "forbid"
missing_debug_implementations = "warn"

[lints.clippy]
unwrap_used = "warn"
expect_used = "warn"
todo = "deny"
dbg_macro = "deny"
print_stdout = "warn"
```

Enable the stricter lints per module when a module is clean, rather than
enabling them all at once.

## EditorConfig

`.editorconfig` must set Rust files to four spaces, matching rustfmt:

```ini
[*.rs]
indent_style = space
indent_size = 4
```

## Verification

- Formatting: `cargo fmt --all -- --check` exits 0.
- Lints: `cargo clippy --all-targets --all-features -- -D warnings` exits 0.
- Docs: `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` exits 0.
