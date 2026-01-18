#!/usr/bin/env python3
"""
Run FGP Blender daemon inside Blender's Python environment.

This script is executed by Blender in background mode to provide bpy access.

Usage:
    blender --background --python run_daemon.py
"""

import sys
import os

# Add FGP packages to path if not already installed
sys.path.insert(0, "/Users/wolfgangschoenberger/Projects/fgp/daemon-py")
sys.path.insert(0, "/Users/wolfgangschoenberger/Projects/fgp/blender/src")

# Now import bpy (should be available since we're running inside Blender)
try:
    import bpy
    print(f"[FGP] Blender version: {bpy.app.version_string}")
    print(f"[FGP] bpy module loaded successfully")
except ImportError as e:
    print(f"[FGP] ERROR: Could not import bpy: {e}")
    sys.exit(1)

# Import and run the daemon
from fgp_daemon import FgpServer
from fgp_blender.service import BlenderService

SOCKET_PATH = os.path.expanduser("~/.fgp/services/blender/daemon.sock")

def main():
    print("[FGP] Starting FGP Blender daemon with bpy support...")

    service = BlenderService()
    server = FgpServer(service, SOCKET_PATH)

    print(f"[FGP] Listening on: {SOCKET_PATH}")
    print("[FGP] Press Ctrl+C to stop")

    try:
        server.serve()
    except KeyboardInterrupt:
        print("\n[FGP] Shutting down...")

if __name__ == "__main__":
    main()
