---
name: commit-and-versioning
description: Write Penoos commit subjects (feat, fix, hotfix, deps, SEMVER-MINOR, SEMVER-MAJOR, breaking changes) and decide SemVer bumps. Use before every commit or when choosing a version.
---

# Commit and versioning

Full policy: [git-workflow.md](../../../engineering/git-workflow.md) and
[versioning.md](../../../engineering/versioning.md).

## Subject format

```
<type>[(scope)][!]: <subject>
```

| Type | Bump |
|------|------|
| `feat`, `SEMVER-MINOR` | MINOR |
| `fix`, `hotfix`, `deps`, `perf` | PATCH |
| `SEMVER-MAJOR`, or `!`, or `BREAKING CHANGE:` footer | MAJOR (see pre-1.0 note) |
| `chore`, `docs`, `refactor`, `test`, `ci`, `build`, `style`, `revert` | none |

Rules: at most 72 characters, imperative, no trailing period.

## Examples

```
feat(broker): add call timeout from CallOptions
fix(transport): return error instead of panicking in tcp call
SEMVER-MAJOR: rename ServiceBroker::call to call_action
deps: update tokio to 1.40
```

## Pre-1.0 note

The bump rule for `0.x` is not yet confirmed. The proposal is that breaking
changes bump the minor version. Ask the user before tagging a breaking change.

## Hooks

```bash
scripts/install-hooks.sh     # one time per clone
```

`commit-msg` rejects invalid subjects. `pre-push` runs fmt, clippy, and tests.
Do not use `--no-verify`.
