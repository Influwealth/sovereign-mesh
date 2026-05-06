# Sovereign Mesh Roadmap

## Phase 0: Scaffold

- Define capsule-first documentation.
- Create compile-safe Rust runtime crate.
- Create compile-safe Rust SDK crate.
- Add a prediction capsule example with metadata, policy, graph, and audit templates.

## Phase 1: Local Capsule Runtime

- Add capsule manifest loading.
- Add policy parsing for YAML rules.
- Add audit sink implementations.
- Add WASM engine integration behind the executor trait.
- Add local capsule execution tests.

## Phase 2: Sovereign Governance

- Add signed capsule manifests.
- Add governance review queues.
- Add role and jurisdiction-aware policy evaluation.
- Add graph-aware dependency validation.
- Add deterministic replay tooling.

## Phase 3: Mesh Networking

- Add node identity.
- Add capsule discovery.
- Add execution receipts.
- Add state checkpoint exchange.
- Add peer-to-peer audit propagation.

## Phase 4: Quantum-ready Inference

- Add quantum adapter registry.
- Add simulator-backed inference.
- Add NVQ mesh routing.
- Add policy-gated probabilistic execution.
- Add audit proofs for quantum-boundary decisions.

## Phase 5: ICP Compatibility Research

- Map canister lifecycle concepts to capsule lifecycle concepts.
- Evaluate compatibility with selected Internet Computer interfaces.
- Research state certification and subnet-style scheduling.
- Identify migration paths for WASM canister logic into capsule artifacts.
