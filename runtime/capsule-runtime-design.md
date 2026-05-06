# Capsule Runtime Design

## Overview

The capsule runtime is a deterministic host for sovereign WASM execution. It accepts ingress messages, checks policy, schedules capsule execution, invokes the WASM boundary, optionally calls a quantum inference adapter, and emits audit records.

## Runtime Components

### Ingress

Ingress messages identify a caller, capsule, method, payload, and trace identifier. The runtime validates basic structure before policy evaluation.

### Policy

The policy layer determines whether a caller may execute a capsule method. Decisions are explicit: allow, deny, or escalate.

### Scheduler

The scheduler chooses execution priority. The initial scheduler is deterministic and simple. Future schedulers may consider graph dependencies, governance requirements, and resource budgets.

### Execution

The execution layer is represented by a trait. It will later host a WASM engine while preserving deterministic inputs and outputs.

### Quantum

The quantum layer is a mocked inference boundary. It is called only when requested by capsule execution context or policy. It returns deterministic placeholder scores.

### Audit

The audit layer records runtime decisions. Production implementations should use append-only logs, cryptographic hashes, and sovereign retention policies.

## Execution Flow

```text
Ingress -> Policy -> Scheduler -> Execution -> Quantum Boundary -> Audit -> Receipt
```

## Failure Behavior

Policy denial and governance escalation stop execution before capsule code runs. Execution failures produce receipts and audit records.
