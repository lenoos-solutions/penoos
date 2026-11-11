---
name: rust-naming
description: Apply Penoos Rust naming conventions to items, modules, files, features, and errors. Use when introducing or renaming anything public.
---

# Rust naming

Full policy: [naming.md](../../../engineering/naming.md).

## Quick rules

- Types and traits: `UpperCamelCase`. Functions, modules, fields: `snake_case`. Constants: `SCREAMING_SNAKE_CASE`.
- Getters have no `get_`. Use `as_`, `to_`, `into_` by cost and ownership.
- Predicates read as questions: `is_*`, `has_*`.
- Errors end in `Error`. Use `penoos::Result<T>`; do not add other `Result` aliases.
- Features are `kebab-case`.
- Action names use `service.action`; events use `domain.past_tense`.
- Banned module names: `util`, `helpers`, `misc`, `common`.

## Before renaming a public item

1. Search all uses: `rg -n "OldName" src tests benches examples docs`.
2. Decide whether it is breaking. If so, it needs `SEMVER-MAJOR:` (see [commit-and-versioning](../commit-and-versioning/SKILL.md)).
3. Rename in one commit, with a deprecation alias only when the user asks for one.
