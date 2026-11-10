# Penoos

**Progressive microservices framework for Rust**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![GitHub stars](https://img.shields.io/github/stars/lenoos-soutions/penoos?style=social)](https://github.com/lenoos-soutions/penoos)
[![GitHub forks](https://img.shields.io/github/forks/lenoos-soutions/penoos?style=social)](https://github.com/lenoos-soutions/penoos)
[![GitHub issues](https://img.shields.io/github/issues/lenoos-soutions/penoos)](https://github.com/lenoos-soutions/penoos/issues)
[![GitHub pull requests](https://img.shields.io/github/issues-pr/lenoos-soutions/penoos)](https://github.com/lenoos-soutions/penoos/pulls)
[![Crates.io](https://img.shields.io/crates/v/penoos.svg)](https://crates.io/crates/penoos)
[![Docs.rs](https://docs.rs/penoos/badge.svg)](https://docs.rs/penoos)

---

**Penoos** is a modern, high-performance microservices framework written in pure Rust.  
It is heavily inspired by the architecture and design principles of [MoleculerJs](https://github.com/moleculerjs/moleculer), while being a complete clean-room rewrite optimized for Rust’s memory safety, performance, and concurrency model.

> **Important**: Penoos is an **independent** project and is **not** an official port of MoleculerJs.  
> It aims for conceptual compatibility and, where practical, protocol-level interoperability with MoleculerJs nodes.

---

## Features

- Clean service-oriented architecture
- Built-in service discovery and dynamic service registry
- Request/Response and Event-driven communication
- Pluggable transporters (NATS, Redis, TCP, and more)
- Flexible middleware system
- Built-in fault tolerance (retry, circuit breaker, timeout, bulkhead)
- Fully async/await based on Tokio
- Strong typing and compile-time guarantees
- Designed for high concurrency and low latency
- Zero-cost abstractions where possible

---

## Project Status

Penoos is under active development.

The core architecture, service broker model, and communication patterns are inspired by Penoos and reimplemented from the ground up in Rust.

---

## Why Penoos?

- You love the productive and elegant programming model of Penoos
- You need the performance, safety, and concurrency benefits of Rust
- You want a framework that follows proven microservices patterns
- You prefer a modern, idiomatic Rust codebase
- You may want future interoperability with existing Penoos clusters

---

## Getting Started

> Documentation and examples are currently being prepared.

```bash
# Once published
cargo add penoos
```

### Quick Example (coming soon)

```rust
// Example will be added soon
```

---

## Inspiration & Attribution

**Penoos** is inspired by the excellent work of the **MoleculerJs** team, especially:

- [MoleculerJs](https://github.com/MoleculerJS/moleculer) by **Icebob** and contributors

This project is a complete rewrite in Rust.  
The original MoleculerJs project is licensed under the MIT License.

We deeply appreciate the original design and the open-source community around it.

---

## License

This project is dual-attributed under the **MIT License**:

- Copyright (c) 2025 **Lenoos** (Lenoos.com owned by [Saeed Tabrizi](https://github.com/saeedtabrizi))
- Copyright (c) 2018 **Icebob** (original MoleculerJs)

See the [LICENSE](LICENSE) file for the full license text.

---

## Contributing

Contributions are very welcome!

Whether you want to report a bug, suggest a feature, improve documentation, or submit a pull request — we would love your help.

Please:

- Open an issue first for larger changes
- Follow the existing code style
- Write clear and descriptive commit messages
- Add tests when possible

---

## Roadmap

- [ ] Core Service Broker
- [ ] Service definition & lifecycle management
- [ ] Local and remote action calling
- [ ] Event bus
- [ ] Transporters (NATS, TCP, Redis)
- [ ] Service discovery & registry
- [ ] Middleware system
- [ ] Fault tolerance patterns
- [ ] Metrics and tracing hooks
- [ ] Optional protocol compatibility layer with **Penoos**
- [ ] Official documentation site
- [ ] Example applications

---

## About Lenoos

Penoos is developed and maintained under the **Lenoos** brand.

- Organization: [lenoos-soutions](https://github.com/lenoos-soutions)
- Repository: [https://github.com/lenoos-soutions/penoos](https://github.com/lenoos-soutions/penoos)

---

**Made with ❤️ in Rust by [Lenoos](https://lenoos.com)**
