# CODEX.md — Sovereign Engineering Protocol for Codex Desktop

## 1. Purpose

Codex operates inside a sovereign multi-repo workspace.
This document defines how Codex must behave when modifying, generating, or orchestrating code across:

- Sovereign Mesh Runtime
- DeepFlex OS (runtime layer)
- WealthBridge OS (economic layer)
- Capsule SDK
- Agent Federation
- Observability + Governance Capsules
- meshctl CLI

Codex must follow all rules in this document.

## 2. Architecture Overview

### 2.1 Sovereign Mesh

A capsule-based execution runtime with:

- Scheduler (youth, underserved, standard, enterprise, edge lanes)
- Execution engine
- State store (in-memory + file-backed)
- SAP message router
- Capsule registry
- Governance capsule
- Observability capsule
- Prediction capsule
- DeepFlex adapter
- meshctl CLI

### 2.2 DeepFlex OS

The runtime compute layer:

- CPU/GPU routing
- Quantum hooks
- Model execution
- SAP ↔ DeepFlex translation

Codex must never mix DeepFlex code with WealthBridge OS code.

### 2.3 WealthBridge OS

The economic layer:

- Compliance
- Subsidy classes
- Governance
- Economic capsules
- Ledger integrations

Codex must never place runtime code inside WealthBridge OS.

## 3. Repository Rules

### 3.1 Never delete files unless explicitly instructed

Codex must preserve all files unless the user explicitly says:

```text
delete this file
```

### 3.2 Never move files across capsules or modules

Capsules are sovereign boundaries.

### 3.3 Maintain monorepo structure

Codex must respect:

```text
runtime/
sdk/
capsules/
meshctl/
observability/
governance/
prediction/
```

### 3.4 Follow SAP (Sovereign Agent Protocol)

All agent communication must use:

- `#[capsule]`
- `#[update]`
- `#[query]`
- `#[event]`
- SAP message routing rules

## 4. Scheduler Rules

Codex must preserve deterministic lanes:

- Youth → `youth-priority`
- Underserved → `underserved-priority`
- Standard → `standard`
- Enterprise → `enterprise`
- Edge-compatible → `edge`

Codex must not modify lane logic unless explicitly instructed.

## 5. Capsule Rules

### 5.1 Capsule boundaries

Each capsule has:

- `capsule.toml`
- `graph.yaml`
- `policy.yaml`
- `audit.toml`
- `src/lib.rs`

Codex must never:

- merge capsules
- move code between capsules
- break capsule metadata

### 5.2 Capsule metadata

Codex must respect:

```text
backend = "sovereign-mesh-runtime"
backend = "deepflex"
```

Codex must route execution accordingly.

## 6. Runtime Rules

### 6.1 Execution Engine

Codex must preserve:

- panic isolation
- SAP routing
- state writes
- event collection
- DeepFlex backend routing

### 6.2 State Store

Codex must preserve:

- `InMemoryStateStore`
- `FileStateStore`
- Snapshot + restore
- Replay mode

### 6.3 Observability

Codex must preserve:

- Prometheus exporter
- Metrics counters
- Histograms
- Gauges
- Observability capsule

## 7. meshctl CLI Rules

Codex must maintain:

- `meshctl status`
- `meshctl graph`
- `meshctl schedule`
- `meshctl capsule <id>`
- `meshctl governance`
- `--json` output

Codex must not break CLI compatibility.

## 8. Commit Style (WealthBridge OS Standard)

### 8.1 Format

```text
<scope>: <imperative summary> — <module>
```

### 8.2 Examples

```text
runtime: add replay mode to execution engine
capsule: governance — add compliance flag query
observability: add scheduler lane metrics
meshctl: implement capsule introspection
```

### 8.3 Requirements

- Imperative mood
- Reference capsule or module
- Include SAP routing changes when relevant

## 9. PR Style

Codex must generate PRs with:

- Summary
- Affected modules
- SAP changes
- Capsule boundaries
- Testing steps
- DeepFlex compatibility notes

## 10. Safety Rules

Codex must:

- Ask before fetching external dependencies
- Ask before making network calls
- Ask before modifying build systems
- Ask before touching DeepFlex or WealthBridge OS internals

## 11. Allowed Actions

Codex may:

- Generate new capsules
- Modify runtime modules
- Add scheduler lanes
- Add governance rules
- Add observability metrics
- Add CLI commands
- Add DeepFlex adapters
- Add SAP message types

## 12. Forbidden Actions

Codex must not:

- Delete files
- Move files across capsules
- Break capsule metadata
- Mix DeepFlex and WealthBridge OS code
- Modify scheduler lane semantics
- Modify governance capsule without explicit instruction
- Modify observability capsule without explicit instruction

## 13. MCP Integration Rules

Codex must use the following MCP servers when interacting with the Sovereign Mesh:

- `sovereign_mesh` → meshctl commands
- `sovereign_runtime` → SAP message execution
- `mesh_observability` → metrics and observability capsule
- `mesh_governance` → governance capsule

Codex must route:

- operational queries → `sovereign_mesh`
- capsule execution → `sovereign_runtime`
- metrics → `mesh_observability`
- governance → `mesh_governance`

## CODEX.md Complete

This is the canonical brain file for Codex inside your sovereign workspace.

Once you add this file, Codex will:

- understand your architecture
- respect your boundaries
- follow SAP
- generate correct commits
- avoid breaking runtime logic
- operate as a sovereign engineering agent
