# Git workflow

## Branches

- `main` is always releasable. Direct pushes are not allowed.
- Work happens on short-lived branches named `<type>/<short-description>`, for example `fix/broker-timeout`.
- Rebase on `main` before opening a pull request. Squash on merge.

## Commit subject format

```
<type>[(scope)][!]: <subject>

[body]

[footer]
```

Allowed types:

| Type | Use for | SemVer effect |
|------|---------|---------------|
| `feat` | New user-visible capability | MINOR |
| `SEMVER-MINOR` | Explicit minor-level change without a feature type | MINOR |
| `fix` | Bug fix | PATCH |
| `hotfix` | Urgent production fix | PATCH |
| `deps` | Dependency added, removed, or updated | PATCH (or none) |
| `SEMVER-MAJOR` | Explicit breaking change | MAJOR |
| `chore` | Maintenance with no runtime effect | none |
| `docs` | Documentation only | none |
| `refactor` | Restructure without behaviour change | none |
| `perf` | Performance change | PATCH |
| `test` | Tests only | none |
| `ci` | CI and workflow changes | none |
| `build` | Build system, Cargo metadata | none |
| `style` | Formatting only | none |
| `revert` | Revert of a previous commit | as reverted |

Breaking changes are marked either with `!` after the type (`feat(api)!: ...`),
with a `BREAKING CHANGE: <description>` footer, or with the `SEMVER-MAJOR:` prefix.

Rules:

1. Subject is at most 72 characters, imperative mood, no trailing period.
2. Body explains *why*, wrapped at 72 characters.
3. Automatic prefixes are accepted: `Merge `, `fixup!`, `squash!`, and `Revert "`.

Examples:

```
feat(broker): add call timeout from CallOptions
fix(transport): return error instead of panicking in tcp call
SEMVER-MAJOR: rename ServiceBroker::call to call_action
deps: update tokio to 1.40
hotfix(runner): stop leaking connections on shutdown
```

## Git hooks

Hooks live in `.githooks/` and are activated per clone with:

```bash
scripts/install-hooks.sh      # sets git config core.hooksPath .githooks
```

| Hook | Runs | Blocks on |
|------|------|-----------|
| `commit-msg` | Validates the subject against the table above | Missing or unknown type, subject over 72 characters |
| `pre-push` | `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` | Any failure |

Bypassing hooks with `--no-verify` is not allowed for pushes to shared branches.

## Pull requests

- Title follows the same format as commit subjects.
- Description uses `.github/PULL_REQUEST_TEMPLATE.md` and lists the tests run.
- At least one approving review is required. The reviewer checks against [../agents/rules.md](../agents/rules.md).
