# Sovereign Mesh Specification

## 1. Purpose

Sovereign Mesh defines a capsule-first execution network for sovereign compute. It borrows the ambition of decentralized WASM execution from the Internet Computer while changing the primary abstraction from canisters to capsules.

Capsules are self-describing execution units. Each capsule contains enough information for a runtime, scheduler, policy engine, graph resolver, and audit system to understand how it should be executed and governed.

## 2. Design Principles

### Rust-first

Runtime components and the SDK are written in Rust. Capsules target WebAssembly and should be portable across sovereign runtimes.

### WASM-targeted

Capsule business logic compiles to WASM. The host runtime provides stable ingress, policy, scheduling, audit, and quantum-boundary APIs.

### Governance-native

Policies are loaded and evaluated as capsule inputs. Runtime execution must be able to deny, approve, or escalate ingress before capsule code runs.

### Audit-complete

Each ingress and execution result is converted into an audit event. Audit events are deterministic records suitable for local logs, distributed ledgers, or later cryptographic proof systems.

### Quantum-ready

The runtime exposes a mocked quantum inference boundary. The initial implementation is deterministic and local, but the interface is structured so it can later route to quantum simulation, NVQ mesh inference, or hardware-backed quantum services.

## 3. Capsule Artifact

A capsule artifact includes:

- `capsule.toml`: identity, version, authorship, runtime target, WASM module, state declaration, and governance metadata.
- `policy.yaml`: ingress authorization, caller classes, mutation permissions, escalation rules, and risk thresholds.
- `graph.yaml`: capsule dependencies, emitted events, consumed events, and sovereign graph relationships.
- `audit.toml`: audit sinks, retention classes, redaction settings, and proof strategy.
- WASM module: compiled capsule execution logic.

## 4. Runtime Responsibilities

The runtime is responsible for:

- accepting ingress messages
- stamping runtime trace identifiers
- loading capsule metadata
- evaluating policy
- scheduling execution
- invoking WASM code through a deterministic boundary
- calling optional quantum inference adapters
- emitting audit records
- returning execution receipts

## 5. Policy Responsibilities

Policy evaluation must happen before execution. A policy decision can be:

- `Allow`: execute the capsule.
- `Deny`: reject the ingress request.
- `Escalate`: stop automatic execution and route to governance review.

Initial policy evaluation is intentionally minimal. Future versions should support signed policies, role-based rules, jurisdiction filters, budget limits, graph-aware constraints, and simulation requirements.

## 6. Graph Responsibilities

The graph layer describes relationships between capsules and external domains. It does not execute code directly. It helps the scheduler and policy engine reason about dependencies, upstream data, downstream events, and governance scope.

Graph relationships should be explicit and versioned so capsule interactions remain auditable.

## 7. Audit Responsibilities

Audit records should include:

- capsule identity
- ingress identity
- policy decision
- scheduler decision
- execution status
- quantum boundary result when used
- trace identifier
- timestamp or deterministic logical time

The scaffold uses a simple in-memory record structure. Production deployments should support append-only sinks and cryptographic integrity proofs.

## 8. Quantum Boundary

The quantum boundary is not a quantum computer implementation. It is a stable interface for inference calls that may later be backed by:

- deterministic local mocks
- quantum simulators
- NVQ mesh inference
- hardware provider APIs
- hybrid classical-quantum scoring

The runtime must remain deterministic unless the capsule policy explicitly permits probabilistic inference.

## 9. Compatibility Notes

Sovereign Mesh should remain aware of Internet Computer concepts such as WASM modules, replicated execution, ingress, certified state, and subnet-style scheduling. It is not currently a drop-in replacement for ICP canisters.
