# Sovereign Mesh

Sovereign Mesh is a sovereign, capsule-oriented fork concept inspired by the Internet Computer. It redesigns distributed execution around capsules: self-describing, governable, quantum-ready execution units that wrap WASM code with state, metadata, policy, graph, and audit information.

The project is Rust-first and WASM-targeted. Its runtime model keeps execution deterministic, governance explicit, and audit trails portable across sovereign deployments.

## Goals

- Run capsule-packaged WASM modules with deterministic ingress, scheduling, and execution boundaries.
- Make governance and policy part of the capsule artifact instead of an external afterthought.
- Preserve auditable state transitions for community, institutional, and regulator-facing review.
- Provide a quantum-ready inference boundary that can route future quantum or NVQ decisions without binding the runtime to a vendor.
- Keep the architecture compatible with sovereign deployment environments that may require local-first control, offline review, or jurisdiction-specific governance.

## Capsule Model

A capsule is an execution package with:

- WASM code
- declared state interfaces
- metadata
- policy
- graph relationships
- audit configuration
- optional quantum inference hints

Capsules are not just binaries. They are governable units that describe who can invoke them, how they relate to other capsules, what they are allowed to mutate, and how their decisions should be reviewed.

## Repository Layout

```text
.
├── CAPSULE_SDK_OVERVIEW.md
├── README.md
├── ROADMAP.md
├── RUNTIME.md
├── SPEC.md
├── runtime/
│   ├── Cargo.toml
│   ├── README.md
│   ├── capsule-runtime-design.md
│   └── src/
└── sdk/
    ├── Cargo.toml
    ├── README.md
    ├── src/
    └── examples/
```

## Current Status

This is the initial scaffold. Runtime execution, policy evaluation, scheduling, audit, and quantum inference are represented by compile-safe placeholder implementations that define the intended boundaries.

## Upstream Relationship

Sovereign Mesh is architecturally inspired by the Internet Computer and designed as a sovereign fork direction. The current GitHub repository is not marked by GitHub as a fork of `dfinity/ic`; it is an independent repository prepared for Sovereign Mesh development.

## License

License selection is pending. Do not assume production licensing terms until a `LICENSE` file is added.
