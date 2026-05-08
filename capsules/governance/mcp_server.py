#!/usr/bin/env python3
"""Sovereign Mesh governance MCP server."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
GOVERNANCE_EXAMPLE = ROOT / "sdk" / "examples" / "governance_compliance"

TOOLS = [
    ("governance_inspect_subsidy_classes", "Inspect subsidy class overrides placeholder."),
    ("governance_inspect_compliance_flags", "Inspect compliance flags placeholder."),
    ("governance_update_governance_state", "Update governance state placeholder."),
    ("governance_read_audit_log", "Read governance audit template."),
    ("governance_read_policy_yaml", "Read governance policy.yaml."),
    ("governance_read_graph_yaml", "Read governance graph.yaml."),
]


def main() -> None:
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
            "serverInfo": {"name": "mesh_governance", "version": "0.1.0"},
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
    if name == "governance_read_policy_yaml":
        return read_template("policy.yaml")
    if name == "governance_read_graph_yaml":
        return read_template("graph.yaml")
    if name == "governance_read_audit_log":
        return read_template("audit.toml")
    if name == "governance_inspect_subsidy_classes":
        return json.dumps({"capsule_id": args.get("capsule_id", "all"), "subsidy_class": "standard"})
    if name == "governance_inspect_compliance_flags":
        return json.dumps({"capsule_id": args.get("capsule_id", "all"), "requires_manual_review": False})
    if name == "governance_update_governance_state":
        return json.dumps({"status": "accepted-placeholder", "requires_governance_admin": True})
    return f"unknown governance MCP tool: {name}"


def read_template(name: str) -> str:
    path = GOVERNANCE_EXAMPLE / name
    if path.exists():
        return path.read_text(encoding="utf-8")
    return f"missing governance template: {name}"


def tool(name: str, description: str) -> dict[str, Any]:
    return {"name": name, "description": description, "inputSchema": {"type": "object", "properties": {}}}


def result(request: dict[str, Any], value: dict[str, Any]) -> dict[str, Any]:
    return {"jsonrpc": "2.0", "id": request.get("id"), "result": value}


if __name__ == "__main__":
    main()
