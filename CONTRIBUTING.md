# Contributing to Penoos

Thank you for your interest in contributing to **Penoos**!  
We welcome contributions of all kinds — bug reports, feature requests, documentation improvements, code, tests, and more.

Penoos is a community-driven project under the Lenoos brand. By contributing, you help make a high-quality, modern microservices framework for Rust.

---

## Code of Conduct

Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).  
We expect all contributors to help maintain a friendly, inclusive, and respectful environment.

---

## How Can I Contribute?

### 1. Reporting Bugs

- Search existing issues first to avoid duplicates.
- Use the bug report template (if available).
- Include:
  - Clear and descriptive title
  - Steps to reproduce
  - Expected vs actual behavior
  - Rust version (`rustc --version`)
  - Operating system
  - Relevant logs or stack traces

### 2. Suggesting Features

- Open an issue with the `enhancement` label.
- Explain the use case and why the feature would be valuable.
- Discuss the design before submitting a large pull request.

### 3. Pull Requests

We love pull requests! Here’s how to contribute code:

1. **Fork** the repository.
2. Create a new branch from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/issue-number-description
   ```
3. Make your changes.
4. Write clear, concise commit messages.
5. Add or update tests when applicable.
6. Ensure the code builds and tests pass:
   ```bash
   cargo check
   cargo test
   cargo fmt --check
   cargo clippy
   ```
7. Push to your fork and open a Pull Request against `main`.

#### Commit Message Guidelines

We prefer conventional commits style. The full type list and the enforced
rules are in [docs/engineering/git-workflow.md](docs/engineering/git-workflow.md).

- `feat: add NATS transporter`
- `fix: resolve service discovery race condition`
- `docs: improve getting started guide`
- `refactor: simplify broker internal state`
- `test: add unit tests for circuit breaker`
- `chore: update dependencies`
- `SEMVER-MINOR: ...` and `SEMVER-MAJOR: ...`, or `!` after the type, for breaking changes

Run `scripts/install-hooks.sh` once after cloning. The `commit-msg` hook checks
the subject and the `pre-push` hook runs fmt, clippy, and tests.

### 4. Documentation

Improvements to README, docs, examples, or comments are highly appreciated.

---

## Development Setup

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/penoos.git
cd penoos

# Build
cargo build

# Run tests
cargo test

# Format
cargo fmt

# Lint
cargo clippy
```

---

## Coding Style

- Follow the Rust standards in [docs/engineering/rust-standards.md](docs/engineering/rust-standards.md) and the rest of [docs/engineering/](docs/engineering/).
- Prefer clear and readable code over clever one-liners.
- Add documentation comments (`///`) for public APIs.
- Keep functions and modules focused and reasonably sized.

Architecture, testing, security, and release guides are indexed in
[docs/README.md](docs/README.md).

---

## Review Process

- All pull requests require at least one review.
- We may request changes before merging.
- Once approved, maintainers will merge the PR.

---

## License

By contributing to Penoos, you agree that your contributions will be licensed under the **MIT License** (same as the project).

---

## Questions?

Feel free to open an issue or start a discussion.  
We’re happy to help!

Thank you for contributing to Penoos ❤️

**Lenoos Team**
