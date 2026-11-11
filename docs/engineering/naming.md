# Naming conventions

Names follow the Rust API Guidelines and the Rust Book. Automated lints
(`clippy`, `rustc`) enforce most of these; the rest are reviewer checks.

## Casing

| Item | Convention | Example |
|------|------------|---------|
| Crates and modules | `snake_case` | `registry`, `prefer_local` |
| Types, traits, enum variants | `UpperCamelCase` | `ServiceBroker`, `TransportPacket` |
| Functions, methods, locals, fields | `snake_case` | `call_with_options`, `node_id` |
| Constants and statics | `SCREAMING_SNAKE_CASE` | `PROTOCOL_VERSION` |
| Type parameters | short `UpperCamelCase` | `T`, `Req` |
| Lifetimes | short lowercase | `'a`, `'ctx` |
| Cargo features | `kebab-case` | `tcp-transport`, `redis-cache` |

## Names

- Use full words. `broker`, not `brk`. Accepted abbreviations: `id`, `ctx` (context), `cfg` (config), `tx` / `rx` (channel ends).
- Getters have no `get_` prefix: `fn name(&self)`, not `fn get_name(&self)`.
- Conversions follow the prefix rule: `as_` (borrowed, cheap), `to_` (owned, may allocate), `into_` (consumes self).
- Boolean accessors read as predicates: `is_connected`, `has_handler`.
- Constructors are `new` for the common case; `with_*` for builder-style options (`with_timeout`).
- Error types end in `Error`. The crate-level alias is `penoos::Result<T>`; do not define other `Result` aliases.
- Traits describe capability (`Transport`, `Middleware`), not implementation (`TransportImpl`).
- Action names use `service.action` dot notation at the API boundary. Event names use `domain.event` in past tense (`user.created`).

## Files

- One concept per file. File names match the primary type in `snake_case`.
- Integration test files describe the behaviour (`call_timeout.rs`, `middleware_order.rs`).
- Benchmarks in `benches/` are named `<area>_bench.rs`.

## Forbidden names

- Do not use `util`, `helpers`, `misc`, or `common` as module names. Name the module after what it does.
- Do not use single-letter names outside closures and short iterator chains.
- Do not reuse a name for a different meaning in the same crate. For example, `Transporter` must not also mean a transport backend.
