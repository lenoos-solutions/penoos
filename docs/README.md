# Penoos Documentation

Canonical documentation for the Penoos Rust microservices framework. Every
AI agent and human contributor starts here.

## Start here

| Audience | Read first |
|----------|------------|
| AI agents | [agents/rules.md](./agents/rules.md), then [agents/README.md](./agents/README.md) |
| New contributors | [../CONTRIBUTING.md](../CONTRIBUTING.md), then [architecture/overview.md](./architecture/overview.md) |
| Reviewers | [engineering/health-report.md](./engineering/health-report.md), then [engineering/rust-standards.md](./engineering/rust-standards.md) |
| Release managers | [engineering/versioning.md](./engineering/versioning.md), [engineering/deployment.md](./engineering/deployment.md) |

## Architecture

- [overview.md](./architecture/overview.md): system context, layers, module map, known gaps (G1–G8).
- [request-lifecycle.md](./architecture/request-lifecycle.md): sequence diagrams for `call`, `mcall`, `emit`, `broadcast`, and middleware.
- [service-lifecycle.md](./architecture/service-lifecycle.md): service state machine, startup, and shutdown.
- [events-and-middleware.md](./architecture/events-and-middleware.md): event bus, the seven middleware hooks, and the integration checklist.
- [transports.md](./architecture/transports.md): transport traits, packet model, and backend status.
- [PENOOS_SUMMARY.md](./architecture/PENOOS_SUMMARY.md): original project summary.

## Engineering standards

- [rust-standards.md](./engineering/rust-standards.md): toolchain, edition, and general Rust rules.
- [lint-and-format.md](./engineering/lint-and-format.md): rustfmt and clippy policy.
- [naming.md](./engineering/naming.md): naming conventions.
- [forbidden-patterns.md](./engineering/forbidden-patterns.md): patterns that are rejected in review.
- [testing.md](./engineering/testing.md): unit, integration, end-to-end, doc, and benchmark tests.
- [security.md](./engineering/security.md): security testing and review rules.
- [versioning.md](./engineering/versioning.md): SemVer policy.
- [git-workflow.md](./engineering/git-workflow.md): commit prefixes, branches, and hooks.
- [deployment.md](./engineering/deployment.md): CI gates and release process.
- [health-report.md](./engineering/health-report.md): measured baseline and remediation plan.

## Agents

- [agents/rules.md](./agents/rules.md): master rules for all agents.
- [agents/README.md](./agents/README.md): index of personas and skills, with access paths per tool.
- `agents/personas/`: role definitions (tech lead, engineer, test, review, security, release, docs).
- `agents/skills/`: task-specific skills (lint, naming, testing, security, benchmarks, rustdoc, commits, architecture, release).

## Other project docs

- [runner/RUST_RUNNER_SUMMARY.md](./runner/RUST_RUNNER_SUMMARY.md): `penoos-runner` binary notes.
- [Penoos.code-snippets](./Penoos.code-snippets): editor snippets.
- `assets/`: image assets used by the root README.
