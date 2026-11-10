//! Constants used throughout the Penoos framework

/// Protocol version
pub const PROTOCOL_VERSION: &str = "5";

/// Packet types for transport
pub mod packet_types {
    pub const EVENT: &str = "EVENT";
    pub const REQUEST: &str = "REQ";
    pub const RESPONSE: &str = "RES";
    pub const DISCOVER: &str = "DISCOVER";
    pub const INFO: &str = "INFO";
    pub const DISCONNECT: &str = "DISCONNECT";
    pub const HEARTBEAT: &str = "HEARTBEAT";
    pub const PING: &str = "PING";
    pub const PONG: &str = "PONG";
}

/// Error types
pub mod error_types {
    pub const SERVICE_NOT_FOUND: &str = "SERVICE_NOT_FOUND";
    pub const SERVICE_NOT_AVAILABLE: &str = "SERVICE_NOT_AVAILABLE";
    pub const REQUEST_TIMEOUT: &str = "REQUEST_TIMEOUT";
    pub const REQUEST_SKIPPED: &str = "REQUEST_SKIPPED";
    pub const MAX_CALL_LEVEL: &str = "MAX_CALL_LEVEL";
    pub const VALIDATION_ERROR: &str = "VALIDATION_ERROR";
    pub const GRACEFUL_STOP_TIMEOUT: &str = "GRACEFUL_STOP_TIMEOUT";
    pub const PROTOCOL_VERSION_MISMATCH: &str = "PROTOCOL_VERSION_MISMATCH";
}

/// Circuit breaker states
pub mod circuit_breaker {
    pub const CLOSED: u8 = 0;
    pub const OPEN: u8 = 1;
    pub const HALF_OPEN: u8 = 2;
}

/// Event types
pub mod event_types {
    pub const EMIT: &str = "emit";
    pub const BROADCAST: &str = "broadcast";
    pub const BROADCAST_LOCAL: &str = "broadcastLocal";
}

/// Load balancing strategies
pub mod strategies {
    pub const ROUND_ROBIN: &str = "RoundRobin";
    pub const RANDOM: &str = "Random";
    pub const CPU_USAGE: &str = "CpuUsage";
    pub const LATENCY: &str = "Latency";
    pub const SHARD: &str = "Shard";
}

/// Cache types
pub mod cache_types {
    pub const MEMORY: &str = "Memory";
    pub const REDIS: &str = "Redis";
    pub const MEMORY_LRU: &str = "MemoryLRU";
}

/// Transporter types
pub mod transporters {
    pub const TCP: &str = "TCP";
    pub const NATS: &str = "NATS";
    pub const REDIS: &str = "Redis";
    pub const MQTT: &str = "MQTT";
    pub const AMQP: &str = "AMQP";
    pub const KAFKA: &str = "Kafka";
    pub const STAN: &str = "STAN";
}

/// Serializer types
pub mod serializers {
    pub const JSON: &str = "JSON";
    pub const MSGPACK: &str = "MsgPack";
    pub const AVRO: &str = "Avro";
    pub const PROTOBUF: &str = "ProtoBuf";
    pub const THRIFT: &str = "Thrift";
    pub const NOTEPACK: &str = "NotePack";
    pub const CBOR: &str = "CBOR";
}

/// Internal service names
pub mod internal_services {
    pub const NODE: &str = "$node";
}

/// Default configuration values
pub mod defaults {
    pub const REQUEST_TIMEOUT: u64 = 10000; // 10 seconds
    pub const HEARTBEAT_INTERVAL: u64 = 10; // seconds
    pub const HEARTBEAT_TIMEOUT: u64 = 30; // seconds
    pub const MAX_CALL_LEVEL: u32 = 0; // unlimited
    pub const RETRY_DELAY: u64 = 100; // milliseconds
    pub const RETRY_MAX_DELAY: u64 = 1000; // milliseconds
    pub const RETRY_FACTOR: f64 = 2.0;
    pub const RETRY_COUNT: u32 = 5;
    pub const STOP_DELAY: u64 = 100; // milliseconds
    pub const DEPENDENCY_INTERVAL: u64 = 1000; // milliseconds
    pub const DEPENDENCY_TIMEOUT: u64 = 0; // unlimited
}

/// Metric names
pub mod metrics {
    pub const MOLECULER_BROKER_STARTED: &str = "penoos.broker.started";
    pub const MOLECULER_BROKER_LOCAL_SERVICES_TOTAL: &str = "penoos.broker.local.services.total";
    pub const MOLECULER_BROKER_MIDDLEWARES_TOTAL: &str = "penoos.broker.middlewares.total";
    pub const MOLECULER_REQUEST_TOTAL: &str = "penoos.request.total";
    pub const MOLECULER_REQUEST_ACTIVE: &str = "penoos.request.active";
    pub const MOLECULER_REQUEST_TIME: &str = "penoos.request.time";
    pub const MOLECULER_REQUEST_LEVELS: &str = "penoos.request.levels";
    pub const MOLECULER_EVENT_EMIT_TOTAL: &str = "penoos.event.emit.total";
    pub const MOLECULER_EVENT_BROADCAST_TOTAL: &str = "penoos.event.broadcast.total";
    pub const MOLECULER_TRANSIT_PUBLISH_TOTAL: &str = "penoos.transit.publish.total";
    pub const MOLECULER_TRANSIT_RECEIVE_TOTAL: &str = "penoos.transit.receive.total";
    pub const MOLECULER_TRANSIT_REQUESTS_ACTIVE: &str = "penoos.transit.requests.active";
}

/// Log levels
pub mod log_levels {
    pub const FATAL: &str = "fatal";
    pub const ERROR: &str = "error";
    pub const WARN: &str = "warn";
    pub const INFO: &str = "info";
    pub const DEBUG: &str = "debug";
    pub const TRACE: &str = "trace";
}

/// Service lifecycle events
pub mod lifecycle_events {
    pub const CREATED: &str = "created";
    pub const STARTED: &str = "started";
    pub const STOPPED: &str = "stopped";
}

/// Transport events
pub mod transport_events {
    pub const CONNECTED: &str = "connected";
    pub const DISCONNECTED: &str = "disconnected";
    pub const ERROR: &str = "error";
}

/// Middleware hooks
pub mod middleware_hooks {
    pub const LOCAL_ACTION: &str = "localAction";
    pub const REMOTE_ACTION: &str = "remoteAction";
    pub const LOCAL_EVENT: &str = "localEvent";
    pub const NEW_LOG_ENTRY: &str = "newLogEntry";
    pub const SERVICE_CREATED: &str = "serviceCreated";
    pub const SERVICE_STARTING: &str = "serviceStarting";
    pub const SERVICE_STARTED: &str = "serviceStarted";
    pub const SERVICE_STOPPING: &str = "serviceStopping";
    pub const SERVICE_STOPPED: &str = "serviceStopped";
    pub const BROKER_STARTING: &str = "brokerStarting";
    pub const BROKER_STARTED: &str = "brokerStarted";
    pub const BROKER_STOPPING: &str = "brokerStopping";
    pub const BROKER_STOPPED: &str = "brokerStopped";
    pub const TRANSIT_CONNECTED: &str = "transitConnected";
    pub const TRANSIT_DISCONNECTED: &str = "transitDisconnected";
    pub const TRANSIT_PUBLISH: &str = "transitPublish";
    pub const TRANSIT_MESSAGE_HANDLER: &str = "transitMessageHandler";
}

/// Internal errors
pub mod internal_errors {
    pub const INVALID_PACKET_DATA: &str = "INVALID_PACKET_DATA";
    pub const MISSING_REQUEST_ID: &str = "MISSING_REQUEST_ID";
    pub const MISSING_RESPONSE_HANDLER: &str = "MISSING_RESPONSE_HANDLER";
    pub const QUEUE_FULL: &str = "QUEUE_FULL";
    pub const MOLECULER_STOPPED: &str = "MOLECULER_STOPPED";
    pub const UNABLE_DECODE_PACKET: &str = "UNABLE_DECODE_PACKET";
}
