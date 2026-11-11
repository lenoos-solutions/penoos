# Rust standards

These standards apply to every Rust change in this repository. Agents and
reviewers enforce them. Each rule states the reason so it can be applied to
cases the rule does not mention.

## Core principles

- **KISS.** Choose the simplest solution that works. Do not add an abstraction before a second real use.
- **DRY.** Put shared logic in modules, mixins, or base services. Never copy business rules.
- **Single responsibility.** One function does one job. Extend through traits, mixins, and events rather than by editing unrelated code.
- **Scope discipline.** Change only what the task needs. Record unrelated problems as issues.
- **Minimal comments.** Document public items and non-obvious logic. Do not restate the code.

## Project layout

| Path | Purpose |
|------|---------|
| `src/` | Library crate `penoos`. Modules are declared in `src/lib.rs`. |
| `src/bin/` | Binaries. `penoos-runner` is declared in `Cargo.toml`. |
| `src/**` `#[cfg(test)] mod tests` | Unit tests, next to the code they test. |
| `tests/` | Integration and end-to-end tests. Each file is a separate crate. |
| `benches/` | Criterion benchmarks (`cargo bench`). |
| `examples/` | Runnable examples. Each example must compile in CI. |
| `docs/` | All human and agent documentation. See [docs/README.md](../README.md). |

Do not create new top-level folders without an ADR. The `test/` and
`benchmark/` directories are legacy placeholders (see [health-report.md](./health-report.md));
new tests go in `tests/` and `benches/`.

## Language rules

1. Edition 2024 is set in `Cargo.toml`. Do not lower it. Set `rust-version` (MSRV) before the first release.
2. Public types are `Send + Sync` unless there is a documented reason otherwise.
3. Prefer borrowing (`&str`, `&[T]`) over owned types in function arguments.
4. Return `Result<T, PenoosError>` (re-exported as `penoos::Result`) from fallible functions. Do not return `Option` where an error reason matters.
5. Use `thiserror` for library error types and `anyhow` only in binaries and examples.
6. Do not use `unsafe`. If it becomes unavoidable, it needs an ADR, a `// SAFETY:` comment, and a test under Miri.
7. Async code runs on Tokio. Never call blocking I/O or `std::thread::sleep` inside an async function; use `tokio::task::spawn_blocking` or the async equivalent.
8. Do not hold a `std::sync` lock across an `.await`. Use `tokio::sync` locks when the guard must live across an await.
9. Implement `Debug` for every public type. Derive `Clone` and `PartialEq` where they make sense.

## Documentation rules

Doc comments follow the Rust Book and the rustdoc conventions:

- Every module starts with a `//!` summary of one or two sentences.
- Every public function, type, and trait has a `///` summary line.
- Fallible public functions have an `# Errors` section.
- Functions that can panic have a `# Panics` section. Avoid panics in library code.
- Public functions with non-trivial usage have an `# Examples` section. Examples are compiled as doc tests.
- Use backticks for code identifiers. Do not restate the function signature in prose.

See [../agents/skills/rustdoc-style/SKILL.md](../agents/skills/rustdoc-style/SKILL.md) for a template.

## Dependencies

- Add a dependency only when the standard library or an existing dependency cannot do the job.
- Keep feature flags minimal: `default-features = false` when only part of a crate is needed.
- Dependency changes are committed with the `deps:` prefix (see [git-workflow.md](./git-workflow.md)).
- Do not commit `Cargo.lock` changes that are unrelated to the change being made.
