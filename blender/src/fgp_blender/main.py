#!/usr/bin/env python3
"""
FGP Blender Daemon - CLI Entry Point

Manages the Blender FGP daemon lifecycle.

CHANGELOG:
01/17/2026 - Initial implementation (Claude)
"""

from __future__ import annotations

import argparse
import json
import logging
import os
import socket
import sys
from pathlib import Path

from fgp_daemon import FgpServer
from fgp_daemon.protocol import Request, Response

from .service import BlenderService

# Socket path
SOCKET_PATH = Path.home() / ".fgp" / "services" / "blender" / "daemon.sock"

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
)
logger = logging.getLogger("fgp-blender")


def is_running() -> bool:
    """Check if daemon is already running."""
    if not SOCKET_PATH.exists():
        return False

    try:
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        sock.settimeout(1.0)
        sock.connect(str(SOCKET_PATH))

        # Send health check
        request = Request.new("health", {})
        sock.sendall(request.to_ndjson_line().encode())

        # Read response
        data = sock.recv(4096).decode()
        sock.close()

        response = Response.from_ndjson_line(data)
        return response.ok

    except Exception:
        return False


def send_request(method: str, params: dict = None) -> dict:
    """Send request to running daemon."""
    if not SOCKET_PATH.exists():
        raise RuntimeError("Daemon not running")

    sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    sock.settimeout(30.0)
    sock.connect(str(SOCKET_PATH))

    try:
        request = Request.new(method, params or {})
        sock.sendall(request.to_ndjson_line().encode())

        # Read response
        data = b""
        while True:
            chunk = sock.recv(4096)
            if not chunk:
                break
            data += chunk
            if b"\n" in data:
                break

        response = Response.from_ndjson_line(data.decode())

        if response.ok:
            return response.result
        else:
            raise RuntimeError(f"Error: {response.error.message}")

    finally:
        sock.close()


def cmd_start(args: argparse.Namespace) -> int:
    """Start the daemon."""
    if is_running():
        print("Daemon is already running")
        return 1

    if args.foreground:
        # Run in foreground
        logger.info("Starting FGP Blender daemon in foreground...")
        service = BlenderService()
        server = FgpServer(service, str(SOCKET_PATH))
        try:
            server.serve()
        except KeyboardInterrupt:
            logger.info("Shutting down...")
        return 0
    else:
        # Daemonize
        print("Starting FGP Blender daemon...")

        # Fork and detach
        pid = os.fork()
        if pid > 0:
            # Parent process
            print(f"Daemon started (PID: {pid})")
            return 0

        # Child process - become session leader
        os.setsid()

        # Fork again to prevent terminal acquisition
        pid = os.fork()
        if pid > 0:
            os._exit(0)

        # Set up daemon environment
        os.chdir("/")
        os.umask(0)

        # Close standard file descriptors
        sys.stdout.flush()
        sys.stderr.flush()

        # Redirect to /dev/null
        with open("/dev/null", "r") as devnull:
            os.dup2(devnull.fileno(), sys.stdin.fileno())

        log_file = SOCKET_PATH.parent / "daemon.log"
        log_file.parent.mkdir(parents=True, exist_ok=True)
        with open(log_file, "a") as log:
            os.dup2(log.fileno(), sys.stdout.fileno())
            os.dup2(log.fileno(), sys.stderr.fileno())

        # Start server
        service = BlenderService()
        server = FgpServer(service, str(SOCKET_PATH))
        server.serve()
        return 0


def cmd_stop(args: argparse.Namespace) -> int:
    """Stop the daemon."""
    if not is_running():
        print("Daemon is not running")
        return 1

    try:
        result = send_request("stop")
        print("Daemon stopped")
        return 0
    except Exception as e:
        print(f"Error stopping daemon: {e}")
        return 1


def cmd_status(args: argparse.Namespace) -> int:
    """Check daemon status."""
    if not is_running():
        print("Daemon is not running")
        return 1

    try:
        result = send_request("health")
        print(f"Status: {result['status']}")
        print(f"Version: {result['version']}")
        print(f"Uptime: {result['uptime_seconds']}s")
        print(f"PID: {result['pid']}")

        if result.get("services"):
            print("\nServices:")
            for name, status in result["services"].items():
                icon = "✓" if status["ok"] else "✗"
                print(f"  {icon} {name}: {status['message']}")

        return 0
    except Exception as e:
        print(f"Error: {e}")
        return 1


def cmd_health(args: argparse.Namespace) -> int:
    """Health check."""
    return cmd_status(args)


def cmd_methods(args: argparse.Namespace) -> int:
    """List available methods."""
    if not is_running():
        print("Daemon is not running")
        return 1

    try:
        result = send_request("methods")
        methods = result.get("methods", [])

        print(f"Available methods ({len(methods)}):\n")

        for method in methods:
            name = method["name"]
            desc = method.get("description", "")
            print(f"  {name}")
            if desc:
                print(f"    {desc}")
            if method.get("params"):
                for param in method["params"]:
                    required = "*" if param.get("required") else ""
                    default = f" (default: {param['default']})" if "default" in param else ""
                    print(f"      - {param['name']}{required}: {param['type']}{default}")
            print()

        return 0
    except Exception as e:
        print(f"Error: {e}")
        return 1


def cmd_call(args: argparse.Namespace) -> int:
    """Call a method directly."""
    if not is_running():
        print("Daemon is not running")
        return 1

    method = args.method
    params = {}

    if args.params:
        try:
            params = json.loads(args.params)
        except json.JSONDecodeError as e:
            print(f"Invalid JSON params: {e}")
            return 1

    try:
        result = send_request(method, params)
        print(json.dumps(result, indent=2))
        return 0
    except Exception as e:
        print(f"Error: {e}")
        return 1


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="FGP Blender Daemon - Fast 3D automation",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  fgp-blender start              Start daemon in background
  fgp-blender start --foreground Start daemon in foreground
  fgp-blender stop               Stop daemon
  fgp-blender status             Check daemon status
  fgp-blender methods            List available methods
  fgp-blender call object.create '{"type": "CUBE", "name": "MyCube"}'
""",
    )

    subparsers = parser.add_subparsers(dest="command", help="Commands")

    # start
    start_parser = subparsers.add_parser("start", help="Start the daemon")
    start_parser.add_argument(
        "-f", "--foreground",
        action="store_true",
        help="Run in foreground (don't daemonize)",
    )
    start_parser.set_defaults(func=cmd_start)

    # stop
    stop_parser = subparsers.add_parser("stop", help="Stop the daemon")
    stop_parser.set_defaults(func=cmd_stop)

    # status
    status_parser = subparsers.add_parser("status", help="Check daemon status")
    status_parser.set_defaults(func=cmd_status)

    # health
    health_parser = subparsers.add_parser("health", help="Health check")
    health_parser.set_defaults(func=cmd_health)

    # methods
    methods_parser = subparsers.add_parser("methods", help="List available methods")
    methods_parser.set_defaults(func=cmd_methods)

    # call
    call_parser = subparsers.add_parser("call", help="Call a method")
    call_parser.add_argument("method", help="Method name (e.g., object.create)")
    call_parser.add_argument("params", nargs="?", default="{}", help="JSON parameters")
    call_parser.set_defaults(func=cmd_call)

    args = parser.parse_args()

    if not args.command:
        parser.print_help()
        return 0

    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
