#!/usr/bin/env python3
"""
FGP MCP Server - Bridge Claude Code/Codex to FGP daemons.

This MCP server discovers installed FGP daemons and exposes their methods
as MCP tools. It communicates with daemons via Unix sockets using the
FGP protocol (NDJSON over Unix sockets).

Usage:
    python3 fgp-mcp-server.py

Register with Claude Code:
    claude mcp add fgp -- python3 /path/to/fgp-mcp-server.py
"""

import asyncio
import json
import os
import socket
import sys
import uuid
from pathlib import Path
from typing import Any

# MCP SDK imports
try:
    from mcp.server import Server
    from mcp.server.stdio import stdio_server
    from mcp.types import Tool, TextContent
except ImportError:
    print("Error: MCP SDK not installed. Run: pip install mcp", file=sys.stderr)
    sys.exit(1)


# ============================================================================
# Configuration
# ============================================================================

FGP_SERVICES_DIR = Path.home() / ".fgp" / "services"
SOCKET_TIMEOUT = 30.0


# ============================================================================
# FGP Client
# ============================================================================

class FgpClient:
    """Client for communicating with FGP daemons via Unix socket."""

    def __init__(self, socket_path: Path):
        self.socket_path = socket_path

    def call(self, method: str, params: dict | None = None) -> dict:
        """Call a method on the FGP daemon."""
        if not self.socket_path.exists():
            raise ConnectionError(f"Socket not found: {self.socket_path}")

        request = {
            "id": str(uuid.uuid4()),
            "v": 1,
            "method": method,
            "params": params or {}
        }

        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        sock.settimeout(SOCKET_TIMEOUT)

        try:
            sock.connect(str(self.socket_path))
            sock.sendall((json.dumps(request) + "\n").encode())

            # Read response (NDJSON - single line)
            response_data = b""
            while True:
                chunk = sock.recv(4096)
                if not chunk:
                    break
                response_data += chunk
                if b"\n" in response_data:
                    break

            response = json.loads(response_data.decode().strip())
            return response
        finally:
            sock.close()

    def health(self) -> dict:
        """Check daemon health."""
        return self.call("health")


# ============================================================================
# Daemon Discovery
# ============================================================================

def discover_daemons() -> dict[str, dict]:
    """Discover installed FGP daemons and their methods."""
    daemons = {}

    if not FGP_SERVICES_DIR.exists():
        return daemons

    for service_dir in FGP_SERVICES_DIR.iterdir():
        if not service_dir.is_dir():
            continue

        manifest_path = service_dir / "manifest.json"
        socket_path = service_dir / "daemon.sock"

        if not manifest_path.exists():
            continue

        try:
            with open(manifest_path) as f:
                manifest = json.load(f)

            daemons[manifest["name"]] = {
                "manifest": manifest,
                "socket_path": socket_path,
                "is_running": socket_path.exists()
            }
        except (json.JSONDecodeError, KeyError) as e:
            print(f"Warning: Failed to load manifest for {service_dir.name}: {e}", file=sys.stderr)

    return daemons


def get_daemon_tools(daemons: dict[str, dict]) -> list[Tool]:
    """Generate MCP tools from daemon manifests."""
    tools = []

    # Add meta tools
    tools.append(Tool(
        name="fgp_list_daemons",
        description="List all installed FGP daemons and their status (running/stopped)",
        inputSchema={
            "type": "object",
            "properties": {},
            "required": []
        }
    ))

    tools.append(Tool(
        name="fgp_start_daemon",
        description="Start an FGP daemon by name",
        inputSchema={
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name of the daemon to start (e.g., 'browser', 'gmail')"
                }
            },
            "required": ["name"]
        }
    ))

    tools.append(Tool(
        name="fgp_stop_daemon",
        description="Stop an FGP daemon by name",
        inputSchema={
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name of the daemon to stop"
                }
            },
            "required": ["name"]
        }
    ))

    # Add daemon-specific tools
    for daemon_name, daemon_info in daemons.items():
        manifest = daemon_info["manifest"]
        methods = manifest.get("methods", [])

        for method in methods:
            method_name = method["name"]
            # Convert browser.open to fgp_browser_open
            tool_name = f"fgp_{method_name.replace('.', '_')}"

            # Build input schema from method params
            properties = {}
            required = []

            for param in method.get("params", []):
                param_name = param["name"]
                param_type = param.get("type", "string")

                # Map FGP types to JSON Schema types
                json_type = {
                    "string": "string",
                    "integer": "integer",
                    "number": "number",
                    "boolean": "boolean",
                    "array": "array",
                    "object": "object"
                }.get(param_type, "string")

                properties[param_name] = {
                    "type": json_type,
                    "description": param.get("description", f"{param_name} parameter")
                }

                if param.get("default") is not None:
                    properties[param_name]["default"] = param["default"]

                if param.get("required", False):
                    required.append(param_name)

            tools.append(Tool(
                name=tool_name,
                description=f"[{daemon_name}] {method.get('description', method_name)}",
                inputSchema={
                    "type": "object",
                    "properties": properties,
                    "required": required
                }
            ))

    return tools


# ============================================================================
# Tool Execution
# ============================================================================

def execute_tool(tool_name: str, arguments: dict, daemons: dict[str, dict]) -> str:
    """Execute an FGP tool and return the result."""

    # Meta tools
    if tool_name == "fgp_list_daemons":
        result = []
        for name, info in daemons.items():
            socket_path = info["socket_path"]
            is_running = socket_path.exists()

            status = {
                "name": name,
                "is_running": is_running,
                "version": info["manifest"].get("version"),
                "description": info["manifest"].get("description"),
                "methods_count": len(info["manifest"].get("methods", []))
            }

            if is_running:
                try:
                    client = FgpClient(socket_path)
                    health = client.health()
                    if health.get("ok"):
                        status["uptime_seconds"] = health.get("result", {}).get("uptime_seconds")
                        status["status"] = health.get("result", {}).get("status", "running")
                except Exception as e:
                    status["status"] = "error"
                    status["error"] = str(e)
            else:
                status["status"] = "stopped"

            result.append(status)

        return json.dumps(result, indent=2)

    if tool_name == "fgp_start_daemon":
        name = arguments.get("name")
        if not name:
            return json.dumps({"error": "Missing 'name' parameter"})

        if name not in daemons:
            return json.dumps({"error": f"Daemon '{name}' not installed"})

        daemon_info = daemons[name]
        manifest = daemon_info["manifest"]
        service_dir = FGP_SERVICES_DIR / name

        # Find the entrypoint
        entrypoint = manifest.get("daemon", {}).get("entrypoint")
        if not entrypoint:
            return json.dumps({"error": f"No entrypoint defined for '{name}'"})

        entrypoint_path = service_dir / entrypoint
        if not entrypoint_path.exists():
            return json.dumps({"error": f"Entrypoint not found: {entrypoint_path}"})

        # Start the daemon
        import subprocess
        try:
            subprocess.Popen(
                [str(entrypoint_path), "start"],
                cwd=str(service_dir),
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                start_new_session=True
            )
            return json.dumps({"ok": True, "message": f"Started daemon '{name}'"})
        except Exception as e:
            return json.dumps({"error": f"Failed to start: {e}"})

    if tool_name == "fgp_stop_daemon":
        name = arguments.get("name")
        if not name:
            return json.dumps({"error": "Missing 'name' parameter"})

        if name not in daemons:
            return json.dumps({"error": f"Daemon '{name}' not installed"})

        socket_path = daemons[name]["socket_path"]
        if not socket_path.exists():
            return json.dumps({"ok": True, "message": f"Daemon '{name}' is already stopped"})

        try:
            client = FgpClient(socket_path)
            result = client.call("stop")
            return json.dumps(result)
        except Exception as e:
            return json.dumps({"error": f"Failed to stop: {e}"})

    # Daemon method tools (fgp_browser_open -> browser.open)
    if tool_name.startswith("fgp_"):
        # Parse tool name: fgp_browser_open -> browser.open
        parts = tool_name[4:].split("_", 1)  # Remove "fgp_" prefix
        if len(parts) == 2:
            daemon_name, method_suffix = parts
            method_name = f"{daemon_name}.{method_suffix}"

            if daemon_name in daemons:
                socket_path = daemons[daemon_name]["socket_path"]

                if not socket_path.exists():
                    return json.dumps({
                        "error": f"Daemon '{daemon_name}' is not running. Start it with fgp_start_daemon."
                    })

                try:
                    client = FgpClient(socket_path)
                    result = client.call(method_name, arguments)
                    return json.dumps(result, indent=2)
                except Exception as e:
                    return json.dumps({"error": str(e)})

    return json.dumps({"error": f"Unknown tool: {tool_name}"})


# ============================================================================
# MCP Server
# ============================================================================

async def main():
    """Run the FGP MCP server."""
    server = Server("fgp")

    # Discover daemons on startup
    daemons = discover_daemons()
    print(f"Discovered {len(daemons)} FGP daemons", file=sys.stderr)

    @server.list_tools()
    async def list_tools() -> list[Tool]:
        """List available FGP tools."""
        # Re-discover daemons to catch new installations
        nonlocal daemons
        daemons = discover_daemons()
        return get_daemon_tools(daemons)

    @server.call_tool()
    async def call_tool(name: str, arguments: dict) -> list[TextContent]:
        """Execute an FGP tool."""
        # Re-discover daemons to get current state
        nonlocal daemons
        daemons = discover_daemons()

        result = execute_tool(name, arguments, daemons)
        return [TextContent(type="text", text=result)]

    # Run the server
    async with stdio_server() as (read_stream, write_stream):
        await server.run(read_stream, write_stream, server.create_initialization_options())


if __name__ == "__main__":
    asyncio.run(main())
