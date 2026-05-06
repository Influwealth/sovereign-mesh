# Sovereign Mesh SDK

`sovereign-mesh-sdk` provides lightweight Rust helpers for writing capsule logic that can be compiled to WASM.

The SDK intentionally avoids runtime-only concerns. Capsule authors should describe their capsule through metadata and implement deterministic business logic in Rust.

## Capsule Macros

The SDK re-exports dependency-free attribute macros:

```rust
use sovereign_mesh_sdk::{capsule, query, update};

#[capsule]
pub mod prediction_capsule {
    #[query]
    pub fn health() -> &'static str {
        "ok"
    }

    #[update]
    pub async fn predict(payload: Vec<u8>) -> Vec<u8> {
        payload
    }
}
```

The initial macros are pass-through markers. They make capsule intent explicit today and leave room for future manifest generation, entrypoint registration, and WASM export wiring.

## Update and Query Methods

Use `#[update]` for methods that may mutate stable, ephemeral, or sealed state. Use `#[query]` for read-only methods. The SDK also exposes `UpdateMethod` and `QueryMethod` traits for runtime-facing entrypoint adapters.

## State Helpers

- `KeyValueStableState`: simple placeholder stable key-value state.
- `EphemeralState`: in-memory state that should not be treated as durable.
- `SealedState`: placeholder trait for future encrypted state.
- `NoopSealedState`: development-only sealer that echoes bytes without cryptography.

## Events

Use `emit_event(name, payload)` to create a `CapsuleEvent`. Future runtime integration will route these events into the capsule graph and audit sinks.

## Quantum Helper

The `quantum` module exposes a deterministic mocked boundary:

```rust
let result = sovereign_mesh_sdk::quantum::run_job("prediction:v1").await?;
```

This is not a real quantum backend. It is the SDK boundary that future adapters will preserve.

## Build

```bash
cargo check
```

## Example

See `examples/prediction_capsule` for a starter capsule with Rust code plus metadata, policy, graph, and audit templates.
