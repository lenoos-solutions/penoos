# Service and Broker Lifecycle

This document covers how a broker and its services move between states, and
the order of work during startup and shutdown. Source references:
`src/broker.rs` (`start`, `stop`, `create_service`, `start_service`, `stop_service`)
and `src/runner.rs` (CLI-driven startup).

## Broker states

```mermaid
stateDiagram-v2
    [*] --> Created: ServiceBroker::new / with_config
    Created --> Created: create_service (registers service, not yet started)
    Created --> Starting: start()
    Starting --> Running: transport connected, all services started
    Starting --> Created: error (returned to caller)
    Running --> Running: create_service (started immediately)
    Running --> Stopping: stop()
    Stopping --> Stopped: services stopped, transport disconnected
    Stopped --> Starting: start() again
    Stopped --> [*]: drop
```

`start()` returns `PenoosError::AlreadyStarted` when the broker is already running.
`stop()` on a stopped broker returns `Ok(())` without side effects.

## Service states

A service has no explicit state field. Its state is derived from the broker's
started flag and the service's lifecycle hooks.

```mermaid
stateDiagram-v2
    [*] --> Registered: create_service (registry + action map updated)
    Registered --> Started: start_service (broker started, or create_service while running)
    Started --> Stopped: stop_service (during broker stop)
    Stopped --> [*]
```

`start_service` invokes the optional `schema.started` hook. `stop_service`
invokes `schema.stopped`. A hook error aborts the operation and is returned to the caller.

## Startup sequence

```mermaid
sequenceDiagram
    autonumber
    participant App as Application or penoos-runner
    participant B as ServiceBroker
    participant T as Transport (optional)
    participant S as Services
    participant E as internal event channel

    App->>B: start()
    alt already started
        B-->>App: Err(AlreadyStarted)
    end
    B->>B: log version, node id, namespace
    opt transport configured
        B->>T: connect()
        T-->>B: Result<()>
    end
    loop each registered service, in insertion order
        B->>S: start_service: schema.started hook
        S-->>B: Result<()>
    end
    B->>B: started = true
    B->>E: send InternalEvent::ServiceStarted(node_id)
    B-->>App: Ok(())
```

## Registration while running

`create_service` on a running broker registers the service, inserts its actions
into the call map, and starts it right away. A service created before `start()`
is started by `start()`.

## Shutdown sequence

```mermaid
sequenceDiagram
    autonumber
    participant App as Application or penoos-runner
    participant B as ServiceBroker
    participant S as Services
    participant T as Transport (optional)

    App->>B: stop()
    alt not started
        B-->>App: Ok(())
    end
    loop each service
        B->>S: stop_service: schema.stopped hook
        S-->>B: Result<()>
    end
    opt transport configured
        B->>T: disconnect()
        T-->>B: Result<()>
    end
    B->>B: started = false
    B-->>App: Ok(())
```

Services are stopped before the transport is disconnected, so stop hooks can
still reach remote peers.

## CLI-driven lifecycle (`penoos-runner`)

The runner wraps the broker with configuration loading and signal handling.

```mermaid
flowchart TD
    A([penoos-runner starts]) --> B[process_flags: parse CLI args]
    B --> C[load_env_file]
    C --> D[load_config_file: TOML or JSON]
    D --> E[merge_options: CLI overrides config]
    E --> F[load_services: glob service files]
    F --> G[start_broker]
    G --> H[wait_for_shutdown: Ctrl+C or SIGTERM]
    H --> I[broker.stop]
    I --> J([exit])
```

Source: `src/runner.rs` lines 53, 139, 177, 291, 329, 479, 527.

## Lifecycle hooks versus middleware

Service hooks (`schema.started`, `schema.stopped`) run today. The matching
`MiddlewareManager::call_service_started` and `call_service_stopped` calls are
not connected, so middleware lifecycle hooks do not run (see
[events-and-middleware.md](./events-and-middleware.md)).
