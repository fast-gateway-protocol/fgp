#!/usr/bin/env python3
"""
Run FGP Blender daemon inside Blender with GUI - V2.

Uses Blender's timer system to execute operations on the main thread
while the socket server runs in a background thread.

Usage:
    blender --python run_daemon_gui_v2.py
"""

import sys
import os
import threading
import queue
import json
import socket
import time
import uuid

# Add FGP packages to path
sys.path.insert(0, "/Users/wolfgangschoenberger/Projects/fgp/daemon-py")
sys.path.insert(0, "/Users/wolfgangschoenberger/Projects/fgp/blender/src")

import bpy
print(f"[FGP] Blender version: {bpy.app.version_string}")

from fgp_daemon.protocol import Request, Response, ErrorCodes, PROTOCOL_VERSION
from fgp_blender.service import BlenderService

SOCKET_PATH = os.path.expanduser("~/.fgp/services/blender/daemon.sock")

# Queue for main-thread execution
operation_queue = queue.Queue()
result_store = {}
result_events = {}

# Service instance (initialized on main thread)
service = None
started_at = None

def init_service():
    """Initialize service on main thread."""
    global service, started_at
    service = BlenderService()
    service.on_start()
    started_at = time.time()
    print("[FGP] Service initialized on main thread")

def process_queue():
    """Process queued operations on main thread (called by timer)."""
    try:
        while True:
            try:
                request_id, method, params = operation_queue.get_nowait()
            except queue.Empty:
                break

            try:
                if method == "health":
                    result = {
                        "status": "healthy",
                        "version": service.version(),
                        "pid": os.getpid(),
                        "uptime_seconds": int(time.time() - started_at),
                        "started_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(started_at)),
                        "services": {
                            name: {"ok": status.ok, "message": status.message}
                            for name, status in service.health_check().items()
                        },
                    }
                elif method == "methods":
                    result = {
                        "methods": [
                            {
                                "name": m.name,
                                "description": m.description,
                                "params": [
                                    {
                                        "name": p.name,
                                        "type": p.param_type,
                                        "required": p.required,
                                        "default": p.default,
                                    }
                                    for p in (m.params or [])
                                ],
                            }
                            for m in service.method_list()
                        ]
                    }
                elif method == "stop":
                    result = {"stopping": True}
                    # Could implement graceful shutdown here
                else:
                    result = service.dispatch(method, params)

                result_store[request_id] = ("ok", result)
            except Exception as e:
                result_store[request_id] = ("error", str(e))

            # Signal that result is ready
            if request_id in result_events:
                result_events[request_id].set()
    except Exception as e:
        print(f"[FGP] Queue processor error: {e}")

    return 0.01  # Run every 10ms

def handle_request(request_data: bytes) -> bytes:
    """Handle a single request, queueing for main thread execution."""
    start_time = time.perf_counter()

    try:
        request = Request.from_ndjson_line(request_data.decode())
    except Exception as e:
        elapsed_ms = (time.perf_counter() - start_time) * 1000
        response = Response.error(
            request_id="unknown",
            code=ErrorCodes.PARSE_ERROR,
            message=f"Invalid request: {e}",
            server_ms=elapsed_ms,
        )
        return response.to_ndjson_line().encode()

    # Create event for this request
    event = threading.Event()
    result_events[request.id] = event

    # Queue the operation for main thread
    operation_queue.put((request.id, request.method, request.params))

    # Wait for result (with timeout)
    if event.wait(timeout=30.0):
        status, data = result_store.pop(request.id, ("error", "No result"))
        del result_events[request.id]
        elapsed_ms = (time.perf_counter() - start_time) * 1000

        if status == "ok":
            response = Response.success(request.id, data, elapsed_ms)
        else:
            response = Response.error(request.id, ErrorCodes.INTERNAL_ERROR, data, elapsed_ms)
    else:
        elapsed_ms = (time.perf_counter() - start_time) * 1000
        response = Response.error(request.id, ErrorCodes.INTERNAL_ERROR, "Operation timed out", elapsed_ms)

    return response.to_ndjson_line().encode()

def socket_server():
    """Run the socket server in a background thread."""
    # Ensure directory exists
    os.makedirs(os.path.dirname(SOCKET_PATH), exist_ok=True)

    # Remove stale socket
    if os.path.exists(SOCKET_PATH):
        os.unlink(SOCKET_PATH)

    # Create socket
    sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    sock.bind(SOCKET_PATH)
    sock.listen(5)
    os.chmod(SOCKET_PATH, 0o600)

    print(f"[FGP] Socket server listening on: {SOCKET_PATH}", flush=True)

    while True:
        try:
            sock.settimeout(1.0)
            try:
                conn, _ = sock.accept()
            except socket.timeout:
                continue

            # Handle connection
            try:
                conn.settimeout(30.0)
                data = b""
                while True:
                    chunk = conn.recv(4096)
                    if not chunk:
                        break
                    data += chunk
                    if b"\n" in data:
                        break

                if data:
                    response = handle_request(data.strip())
                    conn.sendall(response)
            finally:
                conn.close()
        except Exception as e:
            print(f"[FGP] Connection error: {e}")

# Initialize service on main thread
init_service()

# Start socket server in background thread
server_thread = threading.Thread(target=socket_server, daemon=True)
server_thread.start()

# Register timer to process queue on main thread
bpy.app.timers.register(process_queue)

print("[FGP] Daemon running! GUI is interactive, socket is ready.")
print("[FGP] Test with: echo '{\"id\":\"1\",\"v\":1,\"method\":\"health\",\"params\":{}}' | nc -U ~/.fgp/services/blender/daemon.sock")
