# Versioning

This crate follows [Semantic Versioning 2.0.0](https://semver.org/).
`Cargo.toml` is the source of truth for the version (currently `0.1.0`).

## Version increments

| Change | Increment | Commit prefix |
|--------|-----------|---------------|
| Backward-compatible bug fix | PATCH | `fix:` or `hotfix:` |
| Dependency update with no API change | PATCH | `deps:` |
| Backward-compatible new feature | MINOR | `feat:` or `SEMVER-MINOR:` |
| Breaking change to public API, behaviour, or wire protocol | MAJOR | `SEMVER-MAJOR:`, or any type with `!` / a `BREAKING CHANGE:` footer |
| Tooling, docs, CI, tests only | none | `chore:`, `docs:`, `ci:`, `test:`, `refactor:`, `style:` |

## Pre-1.0 policy (decision pending)

While the version is `0.x`, SemVer allows breaking changes in minor releases.
The current proposal is:

- Breaking changes bump the **minor** version while `0.x` (`0.1.0` to `0.2.0`), matching Cargo's convention.
- The `1.0.0` release is cut when the public API in [architecture/overview.md](../architecture/overview.md) is stable and every known gap in that document is closed.

This proposal is not yet confirmed. Confirm it before the first release and record the decision in an ADR.

## Wire protocol

`PROTOCOL_VERSION` in `src/lib.rs` is independent of the crate version. Bump it
when packet layout or semantics change, and treat that bump as a breaking change
even if the crate version does not require one. Nodes with different protocol
versions must refuse to talk to each other.

## Release steps

See [deployment.md](./deployment.md#release-process).

1. Update `version` in `Cargo.toml`, following the table above.
2. Update the changelog section for the release.
3. Run the full checks in [testing.md](./testing.md) and [lint-and-format.md](./lint-and-format.md).
4. Tag the commit `vX.Y.Z` and push the tag.
