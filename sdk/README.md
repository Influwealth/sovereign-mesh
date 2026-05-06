# Sovereign Mesh SDK

`sovereign-mesh-sdk` provides lightweight Rust helpers for writing capsule logic that can be compiled to WASM.

The SDK intentionally avoids runtime-only concerns. Capsule authors should describe their capsule through metadata and implement deterministic business logic in Rust.

## Build

```bash
cargo check
```

## Example

See `examples/prediction_capsule` for a starter capsule with Rust code plus metadata, policy, graph, and audit templates.
