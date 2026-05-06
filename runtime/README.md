# Sovereign Mesh Runtime Crate

`sovereign-mesh-runtime` defines the host-side execution boundary for capsules.

The crate currently provides:

- ingress request structures
- policy decision types
- scheduler placeholders
- WASM executor traits
- mocked quantum inference
- audit record structures
- an orchestration function for end-to-end capsule execution

## Build

```bash
cargo check
```

## Runtime Boundary

The crate intentionally keeps external dependencies at zero for the initial scaffold. WASM engines, YAML/TOML parsing, cryptographic proofs, and networking should be added behind traits so the runtime remains portable.
