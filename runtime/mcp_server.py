#!/usr/bin/env python3
"""Sovereign Mesh runtime MCP server.

This stdio MCP server exposes placeholder SAP execution tools backed by the
current repository metadata. It intentionally uses only the Python standard
library so Codex Desktop can launch it without dependency installation.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]


TOOLS = [
    ("runtime_call_capsule", "Call a capsule through the SAP execution placeholder."),
    ("runtime_send_sap_message", "Send a raw SAP message to the runtime placeholder."),
    ("runtime_query_state", "Query capsule state placeholder."),
    ("runtime_run_update", "Run a capsule update placeholder."),
    ("runtime_run_query", "Run a capsule query placeholder."),
    ("runtime_inspect_events", "Inspect recent runtime events placeholder."),
    ("runtime_inspect_execution_result", "Inspect execution result placeholder."),
    ("runtime_route_deepflex", "Route a SAP message through the DeepFlex adapter placeholder."),
]


def main() -> None:
    for line in sys.stdin:
        if not line.strip():
            continue
        request = json.loads(line)
        response = handle_request(request)
        print(json.dumps(response), flush=True)


def handle_request(request: dict[str, Any]) -> dict[str, Any]:
    method = request.get("method")
    if method == "initialize":
        return result(request, {
            "protocolVersion": "2024-11-05",
            "serverInfo": {"name": "sovereign_runtime", "version": "0.1.0"},
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
    capsule_id = str(args.get("capsule_id", "capsule.prediction.v1"))
    method = str(args.get("method", "health"))
    if name == "runtime_route_deepflex":
        return json.dumps({
            "trace_id": args.get("trace_id", "mcp-trace"),
            "capsule_id": capsule_id,
            "method": method,
            "backend": "deepflex",
            "result": "deepflex_execution_result",
        })
    if name in {tool_name for tool_name, _ in TOOLS}:
        return json.dumps({
            "trace_id": args.get("trace_id", "mcp-trace"),
            "capsule_id": capsule_id,
            "method": method,
            "result": "sovereign_runtime_placeholder",
            "events": [f"{capsule_id}.{method}"],
        })
    return f"unknown runtime MCP tool: {name}"


def tool(name: str, description: str) -> dict[str, Any]:
    return {
        "name": name,
        "description": description,
        "inputSchema": {"type": "object", "properties": {}},
    }


def result(request: dict[str, Any], value: dict[str, Any]) -> dict[str, Any]:
    return {"jsonrpc": "2.0", "id": request.get("id"), "result": value}


if __name__ == "__main__":
    main()
