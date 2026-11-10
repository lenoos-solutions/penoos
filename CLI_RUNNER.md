# Penoos Rust CLI Runner

The Penoos Rust CLI runner (`penoos-runner`) is a command-line tool for running Penoos services, similar to the Node.js `penoos-runner`.

## Installation

```bash
cargo build --bin penoos-runner
```

## Usage

```bash
./target/debug/penoos-runner [OPTIONS] [services]...
```

### Arguments

- `[services]...` - Service files or directories to load

### Options

- `-c, --config <FILE>` - Load configuration from a file
- `-r, --repl` - Start REPL mode (not yet implemented)
- `-H, --hot` - Hot reload services if changed (not yet implemented)
- `-s, --silent` - Silent mode, no logger
- `-e, --env` - Load .env file from the current directory
- `-E, --envfile <FILE>` - Load a specified .env file
- `-m, --mask <PATTERN>` - File mask for service loading (default: `**/*.rs`)
- `-h, --help` - Print help
- `-V, --version` - Print version

### Environment Variables

The runner supports environment variables for configuration:

-- `PENOOS_CONFIG` - Path to configuration file
- `SERVICEDIR` - Directory containing services
- `SERVICES` - Comma-separated list of service files to load
-- `PENOOS_*` - Any environment variable starting with `PENOOS_` will be mapped to configuration

Examples:
-- `PENOOS_LOGLEVEL=debug` → `loglevel: debug`
-- `PENOOS_TRANSPORTER_TYPE=nats` → `transporter.type: nats`

### Configuration Files

The runner supports both JSON and TOML configuration files:

#### penoos.config.json
```json
{
  "nodeID": "node-1",
  "transporter": "nats://localhost:4222",
  "logger": true,
  "logLevel": "info"
}
```

#### penoos.config.toml
```toml
nodeID = "node-1"
logger = true
logLevel = "info"

[transporter]
type = "nats"
url = "nats://localhost:4222"
```

### Examples

#### Run a single service
```bash
./target/debug/penoos-runner examples/hello_world.rs
```

#### Run all services in a directory
```bash
./target/debug/penoos-runner examples/
```

#### Run with configuration file
```bash
./target/debug/penoos-runner --config penoos.config.toml examples/
```

#### Run with environment variables
```bash
PENOOS_LOGLEVEL=debug ./target/debug/penoos-runner examples/hello_world.rs
```

#### Load .env file and run
```bash
./target/debug/penoos-runner --env examples/hello_world.rs
```

### Service Loading

The runner loads Rust service files and attempts to integrate them with the Penoos broker. Currently, dynamic service loading from Rust files is not fully implemented, but the infrastructure is in place.

### Signal Handling

The runner listens for `CTRL+C` (SIGINT) and gracefully shuts down the broker when received.

## Current Limitations

- Dynamic loading of Rust services is not yet implemented (requires compilation or plugin system)
- REPL mode is not implemented
- Hot reload is not implemented
- Clustering support is not implemented

## Future Enhancements

- Dynamic service compilation and loading
- REPL integration for runtime service inspection
- Hot reload with file watching
- Multi-process clustering support
- Better integration with Cargo workspace projects
