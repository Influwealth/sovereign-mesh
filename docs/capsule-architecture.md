# Three-Tier Capsule Architecture

## Overview

The Sovereign Automation System uses three distinct capsule tiers, each optimized for
a different execution context. They are intentionally different — not duplicates.

```
┌─────────────────────────────────────────────────────────┐
│                   DeepFlex Supervisor                    │
│              (TypeScript, port 8000)                     │
│         Decides WHICH capsule tier handles task          │
└──────────────┬──────────────┬──────────────┬────────────┘
               │              │              │
               ▼              ▼              ▼
    ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
    │  Tier 1:     │  │  Tier 2:     │  │  Tier 3:     │
    │  Python      │  │  TypeScript  │  │  Rust/WASM   │
    │  Argus       │  │  WealthBridge│  │  Sovereign   │
    │  Capsules    │  │  Capsules    │  │  Mesh        │
    │  (port 7700) │  │  (port 8001) │  │  Capsules    │
    └──────────────┘  └──────────────┘  └──────────────┘
```

## Tier 1 — Python Argus Capsules

**Location**: `argus-prime/capsules/`
**Runtime**: ArgusRuntime (Python)
**Purpose**: Device-level operations, local automation, offline-capable tasks

### Existing Capsules
| Capsule | Path | Function |
|---------|------|----------|
| `social_farm` | capsules/social_farm/ | Social media scheduling |
| `financials` | capsules/financials/ | Local financial data processing |
| `openwispr` | capsules/openwispr/ | Voice command processing |
| `openplanter` | capsules/openplanter/ | Environment/greenhouse control |

### Capsule Contract
Each Python capsule must implement:
```python
def run(task: str, context: dict) -> dict:
    # Returns: {"status": "ok|error", "result": any, "capsule": "<name>"}
    ...
```

### Config Format
```yaml
# capsules/<name>/capsule.yaml
name: my_capsule
version: 1.0.0
triggers:
  - keyword: "my trigger phrase"
runtime: python
entry: run
```

---

## Tier 2 — TypeScript WealthBridge Capsules

**Location**: `sovereign-stack/wealthbridge-os/capsules/`
**Runtime**: WealthBridge OS Orchestrator (TypeScript, port 8001)
**Purpose**: Business automation (invoices, cashflow, financing, client accounts)

### Existing Capsules (from capsule-registry.json)
| Capsule | Function |
|---------|----------|
| `cap-wealthbridge-genesis` | Core WealthBridge initialization |
| `cap-synapz-feed` | Synapz social feed integration |
| `cap-youth-financial` | Youth financial literacy programs |

### Capsule Contract
```typescript
interface WealthBridgeCapsule {
  id: string;
  version: string;
  execute(params: Record<string, unknown>): Promise<CapsuleResult>;
}
```

### Scheduler Lanes
Capsules run in priority lanes (from `wealthbridge-os/scheduler.ts`):
- `edge` — highest priority
- `enterprise`
- `standard` — default
- `underserved`
- `youth` — lowest priority (background)

---

## Tier 3 — Rust/WASM Sovereign Mesh Capsules

**Location**: `sovereign-mesh/capsules/`
**Runtime**: Sovereign Mesh Runtime (Rust)
**Purpose**: Portable execution units — run anywhere (local, edge, WASM sandbox)

### Capsule Structure
```
capsules/<name>/
├── capsule.toml      # Identity, version, dependencies
├── graph.yaml        # Execution dependency graph
├── policy.yaml       # Permission and resource policy
├── audit.toml        # Audit trail configuration
└── src/
    └── lib.rs        # WASM-compiled Rust logic
```

### capsule.toml Format
```toml
[capsule]
id = "cap-my-capsule"
version = "1.0.0"
runtime = "wasm32-wasi"
entry = "run"

[dependencies]
sovereign-mesh-sdk = "0.1.0"

[permissions]
network = false
filesystem = ["read:/data"]
```

### Quantum Boundary
Tier 3 capsules can interface with the quantum boundary:
```rust
use sovereign_mesh_sdk::quantum::QuantumBoundary;

pub fn run(input: &[u8]) -> Vec<u8> {
    let qb = QuantumBoundary::new(); // mocked; extendable to NVQ hardware
    let result = qb.execute(input);
    result
}
```

### meshctl Commands
```bash
meshctl status              # Node and capsule status
meshctl capsule list        # List registered capsules
meshctl capsule run <id>    # Execute a capsule
meshctl graph show          # Dependency graph
meshctl governance vote     # Governance actions
```

---

## Cross-Tier Communication

Capsules can call up to the DeepFlex Supervisor but NEVER directly to other tiers.
Cross-tier calls go through the supervisor:

```
Tier 1 (Python) ──POST /task──► DeepFlex (port 8000) ──dispatch──► Tier 2 or Tier 3
Tier 2 (TS)     ──POST /task──► DeepFlex (port 8000) ──dispatch──► Tier 1 or Tier 3
Tier 3 (Rust)   ──POST /task──► DeepFlex (port 8000) ──dispatch──► Tier 1 or Tier 2
```

SAP headers are required on all cross-tier calls:
```
x-sap-node-id: <calling-capsule-tier>
x-sap-trace-id: <uuid>
x-sap-version: 1.0
x-sap-capsule: <capsule-id>
```

---

## Adding a New Capsule

1. Determine the correct tier based on execution context
2. Follow the contract for that tier
3. Register in the appropriate registry:
   - Tier 1: Add to `argus-prime/router.py` keyword matching
   - Tier 2: Add to `sovereign-stack/wealthbridge-os/capsule-registry.json`
   - Tier 3: `meshctl capsule register <path>`
4. Add integration test
5. Document in this file
