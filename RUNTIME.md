# Sovereign Mesh Runtime

The Sovereign Mesh runtime executes capsule-packaged WASM modules through deterministic host boundaries.

## Runtime Pipeline

1. Receive ingress.
2. Resolve capsule metadata.
3. Evaluate policy.
4. Schedule execution.
5. Execute capsule WASM.
6. Optionally invoke the mocked quantum boundary.
7. Emit audit records.
8. Return an execution receipt.

## Core Modules

- `ingress`: request types and validation.
- `policy`: policy decisions and evaluator interface.
- `scheduler`: execution scheduling and priority selection.
- `execution`: deterministic capsule execution boundary.
- `quantum`: mocked quantum inference boundary.
- `audit`: audit events and sinks.

## Determinism

The runtime is designed to be deterministic by default. Any operation that introduces uncertainty must be represented as an explicit boundary and recorded in audit output.

## WASM Strategy

The initial scaffold does not embed a WASM engine. The `WasmExecutor` trait defines the execution boundary so future implementations can integrate engines such as Wasmtime, Wasmer, or an ICP-compatible execution layer.

## Policy Strategy

The runtime denies execution by default when policy evaluation fails. Governance escalation is represented as a first-class decision.

## Quantum Strategy

Quantum inference is represented by the `QuantumBoundary` trait. The default implementation returns deterministic mock values. This allows capsule authors to design for future quantum routing without making the runtime dependent on external providers.
