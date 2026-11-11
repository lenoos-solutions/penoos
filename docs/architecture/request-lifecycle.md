# Request Lifecycle

This document describes how actions and events move through the broker. Each
diagram matches the code path named in its heading. Steps marked **(intended)**
are defined in the design but not yet connected; see [overview.md](./overview.md#known-gaps).

## 1. Local action call

`broker.call(name, params)` delegates to `call_with_options` with default options.

```mermaid
sequenceDiagram
    autonumber
    participant App as Application
    participant B as ServiceBroker
    participant H as call_handlers (DashMap)
    participant S as Service action handler

    App->>B: call("math.add", params)
    B->>B: call_with_options(name, params, CallOptions::default())
    B->>B: Context::new(node_id, action, params, meta)
    B->>H: get("math.add")
    H-->>B: handler found
    B->>S: handler(context)
    S-->>B: Result<Value>
    B-->>App: Result<Value>
    Note over B,S: (intended) before_call / after_call middleware and timeout are not applied (G1, G2)
```

## 2. Remote action call

When no local handler exists, the broker delegates to the transport.

```mermaid
sequenceDiagram
    autonumber
    participant App as Application
    participant B as ServiceBroker
    participant T as Transport
    participant N as Remote node broker
    participant S as Remote service

    App->>B: call("billing.charge", params)
    B->>B: call_handlers.get("billing.charge") => None
    alt transport configured
        B->>T: call(action_name, context)
        T->>N: request packet
        N->>S: handler(context)
        S-->>N: Result<Value>
        N-->>T: response packet
        T-->>B: Result<Value>
        B-->>App: Result<Value>
    else no transport
        B-->>App: Err(ServiceNotFound)
    end
```

## 3. Decision logic for `call_with_options`

```mermaid
flowchart TD
    Start([call_with_options]) --> Ctx[Build Context from node_id, action, params, meta]
    Ctx --> Local{Local handler for action?}
    Local -- yes --> Run[Run handler synchronously]
    Run --> RetLocal([Return Result])
    Local -- no --> HasT{Transport configured?}
    HasT -- yes --> Remote[transport.call action, context]
    Remote --> RetRemote([Return Result])
    HasT -- no --> Err([Err ServiceNotFound])
```

## 4. Batch calls (`mcall`)

`mcall` runs each definition in order and stops at the first error with `?`.
Array input returns a JSON array; object input returns a JSON object keyed by
the original keys.

```mermaid
sequenceDiagram
    autonumber
    participant App as Application
    participant B as ServiceBroker

    App->>B: mcall(Array or Object of CallDefinition)
    loop each definition, in order
        B->>B: call_with_options(action, params, options or default)
        alt error
            B-->>App: Err (remaining calls are not executed)
        end
    end
    B-->>App: Ok(serialized results)
```

Calls run **sequentially**. Parallel fan-out is not implemented; callers that
need it should use `tokio::join!` over `call` futures.

## 5. Event emit (local fan-out)

```mermaid
sequenceDiagram
    autonumber
    participant App as Application
    participant B as ServiceBroker
    participant Svc as Local services (RwLock read)
    participant T as tokio task

    App->>B: emit("user.created", payload)
    B->>Svc: read lock, iterate services
    loop each service whose schema.events has "user.created"
        B->>T: spawn(handler(Context clone))
        Note over T: handler errors are logged, not returned
    end
    B-->>App: Ok(())  (returns before handlers finish)
```

`emit` does not wait for handlers and does not report handler failure to the caller.

## 6. Broadcast (local plus transport)

```mermaid
sequenceDiagram
    autonumber
    participant App as Application
    participant B as ServiceBroker
    participant T as Transport

    App->>B: broadcast("user.created", payload)
    B->>B: emit(event, payload.clone())
    opt transport configured
        B->>T: broadcast(event, payload)
        T-->>B: Result<()>
    end
    B-->>App: Result<()>
```

Remote delivery depends on the transport implementation and on remote nodes
subscribing to the event. Local `emit` is never forwarded to remote nodes on its own (G4).

## 7. Middleware pipeline (intended)

This is the target order from `src/middleware.rs`. It is **not executed today**
because `ServiceBroker` never constructs or calls `MiddlewareManager` (G1).

```mermaid
sequenceDiagram
    autonumber
    participant B as ServiceBroker
    participant M as MiddlewareManager
    participant H as Action handler

    B->>M: call_before_call(&mut ctx)
    Note over M: Middleware run in registration order; first Err aborts the call
    M-->>B: Ok or Err
    B->>H: handler(ctx)
    H-->>B: result
    B->>M: call_after_call(&ctx, &result)
    M-->>B: Ok or Err
```

Event emission has matching hooks, `call_before_emit` and `call_after_emit`,
defined in the same manager. See [events-and-middleware.md](./events-and-middleware.md).
