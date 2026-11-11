# Deployment

## Continuous integration

The Node.js workflows under `.github/workflows/` (`ci.yml`, `lint.yml`,
`e2e.yml`, `bench.yml`, `pkg-publish.yml`, `notification.yml`) were copied from a
Moleculer project and do not check this crate. They should be removed once the
Rust workflow is green. The Rust workflow is `.github/workflows/rust-ci.yml`.

`rust-ci.yml` runs on every push and pull request:

| Job | Steps |
|-----|-------|
| `fmt` | `cargo fmt --all -- --check` |
| `clippy` | `cargo clippy --all-targets -- -D warnings` (default features) |
| `test` | `cargo test` (unit, integration, doc; default features) |
| `features` | `cargo check` for `--no-default-features`, `--features tcp-transport`, `--features nats-transport`, `--features msgpack`, `--features oxicode-serializer`, `--features wincode-serializer`, and `--features postcard-serializer`. `cargo test --all-features` is the release gate |
| `docs` | `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` |
| `audit` | `cargo audit` |

## Pre-push gate

The `pre-push` hook runs `fmt`, `clippy`, and the test suite. It mirrors the
`fmt`, `clippy`, and `test` CI jobs, so a push that passes the hook should not
fail CI for those reasons.

## Release process

1. Confirm `main` is green in `rust-ci.yml`.
2. Pick the version from [versioning.md](./versioning.md) and update `Cargo.toml`.
3. Commit with `chore(release): vX.Y.Z`.
4. Create an annotated tag `vX.Y.Z` and push it.
5. The release job runs the full test suite and `cargo package --dry-run`, then `cargo publish` using a token stored as a repository secret.
6. Create a GitHub release with the changelog section for the version.

Do not publish from a local machine.

## Runtime deployment

- Binary: `penoos-runner` (declared in `Cargo.toml`).
- Containers: `examples/docker/` shows a client and worker setup. It is a smoke test only, until the TCP and NATS transports are implemented (G6).
- Configuration comes from environment variables and `PenoosConfig`. Secrets never go in the image.

## Rollback

Redeploy the previous tag. Do not roll back by editing the protocol version in
place; see the wire protocol rule in [versioning.md](./versioning.md).
