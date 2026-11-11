# Agent rules

These rules apply to every AI agent working in this repository, whatever the
tool (GitHub Copilot, Claude, Codex, opencode, or others). They are the contract
that code and reviews are judged against. Read this file first, then load only
the skill or persona you need.

## Read order (token budget)

1. This file (about 1 page).
2. [README.md](./README.md) to pick a persona and skill.
3. The one skill that matches the task. Do not load every skill.
4. Architecture or engineering docs only when the task touches that area.

## Non-negotiable rules

1. **Docs are the source of truth.** Canonical content lives under `docs/`. Root files (`AGENTS.md`, `CLAUDE.md`, `.github/copilot-instructions.md`) only point here.
2. **Read before writing.** Read the file you change and its tests before editing. Do not guess APIs; check `src/lib.rs` re-exports and the module that defines the item.
3. **Scope discipline.** Change only what the task needs. Note unrelated problems in the final summary instead of fixing them.
4. **Surgical edits.** Prefer the smallest diff that is complete. Do not reformat unrelated code.
5. **No forbidden patterns.** See [engineering/forbidden-patterns.md](../engineering/forbidden-patterns.md). In particular: no new `unwrap`, `todo!`, `println!`, or `#[allow]` in library code.
6. **Tests with every behaviour change.** Add or update unit tests, and integration tests when the public API changes. See [engineering/testing.md](../engineering/testing.md).
7. **Run the checks before claiming done.** `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`. Report any check you could not run.
8. **Commit format.** Subjects use the prefixes in [engineering/git-workflow.md](../engineering/git-workflow.md). Include the `Co-authored-by` trailer when the user has not opted out.
9. **Minimal comments.** Comment public items (`///`) and non-obvious logic only. Do not restate code.
10. **No secrets.** Never write tokens, passwords, or connection strings into code, docs, tests, or logs.
11. **Ask when unsure.** A decision that changes behaviour, the wire protocol, or the public API needs a human answer. Write an ADR proposal instead of deciding silently.
12. **Do not bypass hooks.** Do not use `--no-verify` to push. If a hook fails, fix the cause.

## Definition of done

- The change does exactly what the task asked, and nothing else.
- Public items are documented; fallible functions have `# Errors`.
- Tests exist for the new behaviour and pass.
- `cargo fmt`, `clippy -D warnings`, and tests pass locally, or the failure is reported with its cause.
- The commit subject is valid for the `commit-msg` hook.
- Docs that describe the changed behaviour are updated.

## Known constraints

- The repository does not yet pass the checks above. See [engineering/health-report.md](../engineering/health-report.md). Do not fix unrelated lint debt inside a feature change; open or follow the remediation item.
- Architecture gaps G1 to G8 are listed in [architecture/overview.md](../architecture/overview.md#known-gaps). Do not describe them as working.
