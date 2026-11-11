# Agents index

This folder holds the agent personas and skills for Penoos. It is written for
any AI coding tool. Each tool reaches these files through a symlink or a root
pointer, so there is one copy to maintain.

| Path | What it is | Used by |
|------|------------|---------|
| [rules.md](./rules.md) | Master rules every agent follows | All |
| `personas/*.agent.md` | Role definitions with tool and scope limits | Copilot custom agents, Claude subagents |
| `skills/<name>/SKILL.md` | Task playbooks, loaded on demand | Claude skills, Copilot and Codex skills |

## Access paths

| Tool | Entry point (real file) | Links to |
|------|------------------------|----------|
| Any tool | [AGENTS.md](../../AGENTS.md) | this index |
| Claude Code | [CLAUDE.md](../../CLAUDE.md) | this index |
| GitHub Copilot | [.github/copilot-instructions.md](../../.github/copilot-instructions.md) | this index |
| Skills (Claude, Codex, opencode) | `.claude/skills/`, `.agents/skills/` | symlinks to `docs/agents/skills/` |
| Agents (Copilot, Claude) | `.github/agents/`, `.claude/agents/` | symlinks to `docs/agents/personas/` |

The dot-directories are symlinks. Edit files under `docs/agents/`, never through
the symlink paths, so the real path stays under `docs/`.

## Personas

| Persona | Use for | Writes code |
|---------|---------|-------------|
| [rust-tech-lead](./personas/rust-tech-lead.agent.md) | Design decisions, plan review, ADRs | No |
| [rust-engineer](./personas/rust-engineer.agent.md) | Implementing features and fixes | Yes |
| [test-engineer](./personas/test-engineer.agent.md) | Unit, integration, e2e, doc, and bench tests | Tests only |
| [code-reviewer](./personas/code-reviewer.agent.md) | Review against rules and lints | No |
| [security-reviewer](./personas/security-reviewer.agent.md) | Security review of transport, auth, serializers | No |
| [release-engineer](./personas/release-engineer.agent.md) | Versioning, changelog, tags, CI | Release files only |
| [docs-architect](./personas/docs-architect.agent.md) | Architecture docs and diagrams | Docs only |

## Skills

Load one skill per task.

| Skill | Load when |
|-------|-----------|
| [rust-lint-and-format](./skills/rust-lint-and-format/SKILL.md) | Running or fixing fmt, clippy, rustdoc |
| [rust-naming](./skills/rust-naming/SKILL.md) | Naming items, files, or features |
| [rust-testing](./skills/rust-testing/SKILL.md) | Writing or placing any test |
| [rust-security](./skills/rust-security/SKILL.md) | Touching packets, auth, serializers, dependencies |
| [rust-benchmarks](./skills/rust-benchmarks/SKILL.md) | Adding or comparing criterion benchmarks |
| [rustdoc-style](./skills/rustdoc-style/SKILL.md) | Writing `///` and `//!` docs |
| [commit-and-versioning](./skills/commit-and-versioning/SKILL.md) | Commit subjects, SemVer, hooks |
| [penoos-architecture](./skills/penoos-architecture/SKILL.md) | Understanding broker, transports, middleware |
| [release-process](./skills/release-process/SKILL.md) | Cutting a release or tag |

## Token budget

- Start with [rules.md](./rules.md) and one persona. That is enough for most tasks.
- Load a skill only when its trigger applies, and read only the sections needed.
- Read the architecture docs when the change crosses modules, not for local edits.
