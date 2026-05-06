# Sovereign Mesh Runtime Crate

`sovereign-mesh-runtime` defines the host-side execution boundary for capsules.

The crate currently provides:

- ingress request structures
- `handle_ingress(req)` for the basic pipeline entrypoint
- policy decision types
- `check_policy(req)` for placeholder capsule/method validation
- scheduler placeholders
- `schedule(req)` for deterministic placeholder placement
- WASM executor traits
- `execute_capsule(req)` for placeholder capsule execution
- mocked quantum inference
- `run_quantum_job(params)` for future quantum adapter integration
- audit record structures
- `record_audit_event(event, details)` for placeholder audit emission
- an orchestration function for end-to-end capsule execution

## Pipeline

The current function flow is:

```text
ingress -> policy -> scheduler -> execution -> quantum (optional) -> audit
```

`ingress::handle_ingress` calls policy, scheduler, and execution directly, then emits placeholder audit events. The trait-based `CapsuleRuntime` remains available for more configurable host implementations.

## Build

```bash
cargo check
```

## Runtime Boundary

The crate intentionally keeps external dependencies at zero for the initial scaffold. WASM engines, YAML/TOML parsing, cryptographic proofs, and networking should be added behind traits so the runtime remains portable.
