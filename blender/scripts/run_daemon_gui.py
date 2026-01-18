#!/usr/bin/env python3
"""
Run FGP Blender daemon inside Blender with GUI.

This script starts the daemon in a background thread so the GUI remains responsive.

Usage:
    blender --python run_daemon_gui.py
"""

import sys
import os
import threading

# Add FGP packages to path
sys.path.insert(0, "/Users/wolfgangschoenberger/Projects/fgp/daemon-py")
sys.path.insert(0, "/Users/wolfgangschoenberger/Projects/fgp/blender/src")

import bpy
print(f"[FGP] Blender version: {bpy.app.version_string}")
print(f"[FGP] bpy module loaded successfully")

from fgp_daemon import FgpServer
from fgp_blender.service import BlenderService

SOCKET_PATH = os.path.expanduser("~/.fgp/services/blender/daemon.sock")

# Ensure directory exists
os.makedirs(os.path.dirname(SOCKET_PATH), exist_ok=True)

def run_server():
    """Run the FGP server in a background thread."""
    print("[FGP] Starting FGP Blender daemon with GUI support...")
    service = BlenderService()
    # skip_signals=True allows running in a background thread
    server = FgpServer(service, SOCKET_PATH, skip_signals=True)
    print(f"[FGP] Listening on: {SOCKET_PATH}")
    server.serve()

# Start daemon in background thread
daemon_thread = threading.Thread(target=run_server, daemon=True)
daemon_thread.start()

print("[FGP] Daemon running in background - GUI is ready!")
print("[FGP] You can now send commands to the daemon via the socket.")
