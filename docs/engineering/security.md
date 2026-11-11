# Security

## Principles

- Treat every packet from a remote node as untrusted input. Validate length, type, and version before deserializing into domain types.
- Fail closed. An authorization or validation error rejects the call.
- Never log payloads, tokens, or connection strings. Log action names, node IDs, and error kinds.
- Secrets come from the environment or a secret store, never from source or committed config.

## Threat areas

| Area | Risk | Control |
|------|------|---------|
| Transport packets | Malformed or oversized payloads, memory exhaustion | Max packet size, bounded deserialization, versioned protocol (`PROTOCOL_VERSION`) |
| Action calls | Unauthorized access | `AuthMiddleware` must run before the handler (blocked by G1) |
| Rate limits | Flooding | `RateLimitMiddleware` must run before the handler (blocked by G1) |
| Serialization | Unsafe decoders, type confusion | Use `serde` with explicit types; reject unknown versions |
| Dependencies | Known CVEs | `cargo audit` in CI; Dependabot for `cargo` |
| Panics | Denial of service | No `unwrap` in non-test code; no `todo!()` reachable from a network path (G6) |
| Logs | Data leaks | Structured logging without payloads |

## Required checks

```bash
cargo audit                    # known vulnerabilities (install: cargo install cargo-audit)
cargo test --test security_*   # security test suite
```

## Security tests

Place them in `tests/security_*.rs`. Each must cover one of: oversized packet
rejected, unknown protocol version rejected, unauthorized call rejected, rate
limit enforced, malformed JSON or malformed binary payloads (oxicode, postcard, wincode) rejected without panic.

## Reporting

Report vulnerabilities privately through the repository's security advisory
feature. Do not open a public issue for an unfixed vulnerability.

## Review

Security-sensitive changes (transport, middleware, serializers, auth, and
dependency updates that change TLS or crypto) need review by the Security
Reviewer persona in [../agents/README.md](../agents/README.md).
