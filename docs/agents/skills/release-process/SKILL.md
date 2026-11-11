---
name: release-process
description: Cut a Penoos release: check CI, bump version, write changelog, tag, and publish through CI. Use when preparing or tagging a version.
---

# Release process

Full policy: [deployment.md](../../../engineering/deployment.md#release-process).

## Steps

1. Confirm `main` is green in `rust-ci.yml`.
2. Determine the bump from commits since the last tag (see [commit-and-versioning](../commit-and-versioning/SKILL.md)).
3. Update `version` in `Cargo.toml` and add the changelog section.
4. Run locally:

   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-features
   cargo package --dry-run
   ```

5. Commit: `chore(release): vX.Y.Z`.
6. Create an annotated tag `vX.Y.Z` and push it.
7. Let CI publish. Do not publish from a laptop.

## Stop conditions

- Any check fails.
- The release contains a breaking change and the pre-1.0 policy is not confirmed.
- Known blocker gaps (G6, nats-transport build failure) are in the release scope and not documented in the release notes.
