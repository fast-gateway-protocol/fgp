"""
FGP Blender Daemon - Fast Gateway Protocol daemon for Blender automation.

This package provides high-performance Blender automation through persistent
UNIX socket connections, eliminating cold-start latency.

CHANGELOG:
01/17/2026 - Initial implementation (Claude)
"""

from .service import BlenderService
from .session import SessionManager
from .jobs import JobQueue

__all__ = ["BlenderService", "SessionManager", "JobQueue"]
__version__ = "0.1.0"
