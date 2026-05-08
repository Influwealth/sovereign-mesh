#!/usr/bin/env python3
"""Windows-friendly meshctl MCP/operator shim.

Codex Desktop can launch this through meshctl.cmd when the MCP server command is
configured as `meshctl` with arguments `mcp`.
"""

from __future__ import annotations

import json
import sys
from typing import Any


TOOLS = [
    ("meshctl_status", "Return Sovereign Mesh operator status."),
    ("meshctl_graph", "Return capsule graph overview."),
    ("meshctl_schedule", "Return deterministic scheduler lane decision."),
    ("meshctl_capsule", "Inspect one capsule by id."),
    ("meshctl_governance", "Return governance operator status."),
    ("meshctl_metrics", "Return runtime metric summary."),
]


def main() -> None:
    args = sys.argv[1:]
    if args[:1] == ["mcp"]:
        run_mcp()
        return
    if args[:1] == ["cli"]:
        args = args[1:]
    print(render_cli(args))


def run_mcp() -> None:
    for line in sys.stdin:
        if not line.strip():
            continue
        request = json.loads(line)
        print(json.dumps(handle_request(request)), flush=True)


def handle_request(request: dict[str, Any]) -> dict[str, Any]:
    method = request.get("method")
    if method == "initialize":
        return result(request, {
            "protocolVersion": "2024-11-05",
            "serverInfo": {"name": "sovereign_mesh", "version": "0.1.0"},
            "capabilities": {"tools": {}},
        })
    if method == "tools/list":
        return result(request, {"tools": [tool(name, description) for name, description in TOOLS]})
    if method == "tools/call":
        name = request.get("params", {}).get("name", "")
        args = request.get("params", {}).get("arguments", {})
        return result(request, {"content": [{"type": "text", "text": call_tool(name, args)}]})
    return result(request, {})


def call_tool(name: str, args: dict[str, Any]) -> str:
    if name == "meshctl_status":
        return render_status(False)
    if name == "meshctl_graph":
        return render_graph(False)
    if name == "meshctl_schedule":
        return render_schedule(False)
    if name == "meshctl_capsule":
        return render_capsule(str(args.get("id", "capsule.unknown")), False)
    if name == "meshctl_governance":
        return render_governance(False)
    if name == "meshctl_metrics":
        return render_metrics(False)
    return f"unknown meshctl MCP tool: {name}"


def render_cli(args: list[str]) -> str:
    json_output = "--json" in args
    command = next((arg for arg in args if not arg.startswith("--")), "status")
    if command == "status":
        return render_status(json_output)
    if command == "graph":
        return render_graph(json_output)
    if command == "schedule":
        return render_schedule(json_output)
    if command == "capsule":
        capsule_id = args[args.index("capsule") + 1] if len(args) > args.index("capsule") + 1 else "unknown"
        return render_capsule(capsule_id, json_output)
    if command == "governance":
        return render_governance(json_output)
    if command == "metrics":
        return render_metrics(json_output)
    return "Commands: status | graph | schedule | capsule <id> | governance | metrics"


def render_status(json_output: bool) -> str:
    if json_output:
        return json.dumps({
            "node_id": "local-sovereign-node",
            "capsule_count": 4,
            "scheduler_lanes": ["youth-priority", "underserved-priority", "standard", "enterprise"],
            "pending_requests": 0,
        })
    return table([
        ("Node ID", "local-sovereign-node"),
        ("Capsule Count", "4"),
        ("Scheduler Lanes", "youth-priority,underserved-priority,standard,enterprise"),
        ("Pending Requests", "0"),
    ])


def render_graph(json_output: bool) -> str:
    capsules = ["prediction", "dames_legacy_account", "governance_compliance", "observability"]
    if json_output:
        return json.dumps({"graph": "placeholder", "capsules": capsules})
    return table([("Graph", "placeholder"), ("Capsules", ",".join(capsules))])


def render_schedule(json_output: bool) -> str:
    if json_output:
        return json.dumps({"lane": "standard", "edge_eligible": True})
    return table([("Lane", "standard"), ("Edge Eligible", "true")])


def render_capsule(capsule_id: str, json_output: bool) -> str:
    if json_output:
        return json.dumps({"capsule_id": capsule_id, "status": "registered-placeholder"})
    return table([("Capsule", capsule_id), ("Status", "registered-placeholder")])


def render_governance(json_output: bool) -> str:
    if json_output:
        return json.dumps({"governance": "placeholder", "role": "governance-admin"})
    return table([("Governance", "placeholder"), ("Required Role", "governance-admin")])


def render_metrics(json_output: bool) -> str:
    if json_output:
        return json.dumps({
            "scheduler_decisions_total": 0,
            "capsule_executions_total": 0,
            "pending_requests": 0,
        })
    return table([
        ("Scheduler Decisions", "0"),
        ("Capsule Executions", "0"),
        ("Pending Requests", "0"),
    ])


def table(rows: list[tuple[str, str]]) -> str:
    lines = ["FIELD                 VALUE", "--------------------  ------------------------------"]
    lines.extend(f"{field:<20}  {value}" for field, value in rows)
    return "\n".join(lines)


def tool(name: str, description: str) -> dict[str, Any]:
    return {"name": name, "description": description, "inputSchema": {"type": "object", "properties": {}}}


def result(request: dict[str, Any], value: dict[str, Any]) -> dict[str, Any]:
    return {"jsonrpc": "2.0", "id": request.get("id"), "result": value}


if __name__ == "__main__":
    main()
