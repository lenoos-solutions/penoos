---
name: penoos-architecture
description: Orientation to the Penoos broker architecture (ServiceBroker, services, actions, events, transports, middleware, lifecycle). Use before changing cross-module behaviour.
---

# Penoos architecture

Read the full docs only for the area you change:

| Topic | Doc |
|-------|-----|
| Modules, layers, gaps G1 to G8 | [overview.md](../../../architecture/overview.md) |
| Call, mcall, emit, broadcast flows | [request-lifecycle.md](../../../architecture/request-lifecycle.md) |
| Startup, shutdown, service states | [service-lifecycle.md](../../../architecture/service-lifecycle.md) |
| Events and middleware hooks | [events-and-middleware.md](../../../architecture/events-and-middleware.md) |
| Transports and packets | [transports.md](../../../architecture/transports.md) |

## Key facts

- `ServiceBroker::new()` returns `Result<ServiceBroker>`. `call` and `call_with_options` are async.
- Local calls dispatch to the handler in `self.services`. Remote calls use `transport.call`.
- `emit` is local only. `broadcast` also uses the transport.
- The middleware manager exists but the broker does not call it yet (G1).
- `CallOptions` apply only to remote calls (G2).
- `Transport` is the trait the broker uses. `Transporter` is not wired (G3).
- TCP `call`, `broadcast`, and `send_to_node` panic with `todo!()` (G6).

Before you rely on any of these, confirm it in the source file named in the doc.
