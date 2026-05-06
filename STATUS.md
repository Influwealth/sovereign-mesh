# Sovereign Mesh

A sovereign fork of the Internet Computer redesigned around capsules.

## Status: MVP (v0.1.0-alpha)

### ✅ Complete
- [x] SPEC.md (Capsule Specification)
- [x] RUNTIME.md (Runtime Architecture)
- [x] ROADMAP.md (Development Roadmap)
- [x] CAPSULE_SDK_OVERVIEW.md (SDK Guide)
- [x] Core types and traits (sovereign-mesh-core)
- [x] Capsule SDK (sovereign-mesh-sdk)
- [x] Runtime implementation (sovereign-mesh-runtime)
  - [x] Ingress module
  - [x] Policy module
  - [x] Scheduler module
  - [x] Execution module (stub)
  - [x] Quantum module (mock)
  - [x] Audit module
- [x] CLI with 7 commands (sovereign-mesh-cli)
- [x] 3 Example capsules (counter, ledger, quantum)
- [x] Documentation (README, CONTRIBUTING, SECURITY, ARCHITECTURE)

### 🚧 In Progress / Planned
- [ ] Full Wasm integration (wasmtime/wasmer integration)
- [ ] Integration tests
- [ ] Performance benchmarks
- [ ] Security hardening & audit
- [ ] Consensus layer (BFT)
- [ ] Cross-chain messaging

---

## Quick Links

- [README.md](README.md) - Overview and quick start
- [SPEC.md](SPEC.md) - Capsule Specification
- [RUNTIME.md](RUNTIME.md) - Runtime Architecture
- [CAPSULE_SDK_OVERVIEW.md](CAPSULE_SDK_OVERVIEW.md) - SDK Guide
- [ROADMAP.md](ROADMAP.md) - Development Roadmap
- [ARCHITECTURE.md](ARCHITECTURE.md) - Architecture Diagrams
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution Guidelines
- [SECURITY.md](SECURITY.md) - Security Policy

---

## Getting Started

```bash
# Clone repo
git clone https://github.com/Influwealth/sovereign-mesh
cd sovereign-mesh

# Build all crates
cargo build --release

# Run tests
cargo test --all

# Initialize a new capsule
cargo run -p sovereign-mesh-cli -- init my-capsule

# Build and run the capsule
cd my-capsule
cargo run -p sovereign-mesh-cli -- build --release
cargo run -p sovereign-mesh-cli -- run
```

---

## Project Structure

```
sovereign-mesh/
├── sovereign-mesh-core/        # Core types
├── sovereign-mesh-sdk/         # Capsule SDK
├── sovereign-mesh-runtime/     # Runtime
├── sovereign-mesh-cli/         # CLI
├── examples/
│   ├── counter-capsule/
│   ├── ledger-capsule/
│   └── quantum-capsule/
├── SPEC.md
├── RUNTIME.md
├── ROADMAP.md
├── CAPSULE_SDK_OVERVIEW.md
├── README.md
├── CONTRIBUTING.md
├── SECURITY.md
├── ARCHITECTURE.md
├── LICENSE
└── Cargo.toml
```

---

Last Updated: **2026-05-05**  
Status: **Alpha v0.1.0**
