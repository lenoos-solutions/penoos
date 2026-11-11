---
name: rust-security
description: Security checks for Penoos packet handling, auth, serializers, dependencies, and network-reachable panics. Use when a change touches transport, middleware, serialization, or Cargo dependencies.
---

# Rust security

Full policy: [security.md](../../../engineering/security.md).

## Checklist for network-facing code

1. Size limit before allocation. Reject oversized packets before deserializing.
2. Version check. Reject unknown `PROTOCOL_VERSION` values.
3. No panic on bad input: no `unwrap`, `expect`, unchecked indexing, or `todo!()`.
4. Authorization and rate limits run before the handler (G1).
5. No payloads or secrets in logs.
6. Errors returned as `PenoosError`, not as text that leaks internals.

## Dependency changes

```bash
cargo audit
cargo tree -i <crate>
```

Commit with `deps:` (or `chore(deps):`). If the change enables TLS or crypto
features, request the security reviewer persona.

## Tests

Put malformed-input tests in `tests/security_*.rs`: oversized packet,
unknown version, truncated payload, unauthorized call, and rate-limit overflow.
Each test asserts an `Err`, not a panic.
