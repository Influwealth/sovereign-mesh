# Capsule SDK Overview

The Capsule SDK helps developers author WASM-targeted capsule logic and package it with metadata, policy, graph, and audit declarations.

## SDK Responsibilities

- Provide capsule identity types.
- Provide ingress and response helpers.
- Provide metadata structures shared with the runtime.
- Keep exported capsule code simple and WASM-friendly.
- Avoid runtime-specific assumptions inside capsule business logic.

## Capsule Authoring Flow

1. Create capsule Rust code.
2. Compile the capsule to WASM.
3. Write `capsule.toml`.
4. Write `policy.yaml`.
5. Write `graph.yaml`.
6. Write `audit.toml`.
7. Submit the artifact to a Sovereign Mesh runtime.

## Example Capsule

The prediction capsule example demonstrates:

- capsule metadata
- sovereign policy rules
- graph declarations
- audit sink settings
- placeholder Rust logic that can be compiled toward WASM

## Compatibility

The SDK should remain lightweight and stable. Runtime concerns such as scheduling, audit sinks, policy enforcement, and quantum inference remain outside the capsule SDK.
