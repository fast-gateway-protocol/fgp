"""
Session Manager for FGP Blender Daemon.

Manages isolated Blender sessions using .blend files. Each session provides
complete state isolation for parallel workflows.

CHANGELOG:
01/17/2026 - Initial implementation (Claude)
"""

from __future__ import annotations

import json
import logging
import os
import tempfile
import uuid
from dataclasses import dataclass, field, asdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Optional

logger = logging.getLogger(__name__)

# Session storage location
FGP_SESSIONS_DIR = Path.home() / ".fgp" / "services" / "blender" / "sessions"


@dataclass
class Session:
    """A Blender session with isolated state."""

    id: str
    name: str
    blend_file: str
    created_at: str
    modified_at: str
    is_default: bool = False
    metadata: dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        return asdict(self)

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> Session:
        """Create from dictionary."""
        return cls(**data)


class SessionManager:
    """
    Manages Blender sessions with .blend file isolation.

    Each session is backed by a separate .blend file, providing complete
    state isolation. Sessions persist across daemon restarts.

    The default session is ephemeral and used for quick operations.
    Named sessions persist and can be resumed later.
    """

    def __init__(self) -> None:
        """Initialize session manager."""
        self.sessions: dict[str, Session] = {}
        self.current_session_id: Optional[str] = None
        self._sessions_dir = FGP_SESSIONS_DIR
        self._sessions_dir.mkdir(parents=True, exist_ok=True)
        self._sessions_file = self._sessions_dir / "sessions.json"

        # Load persisted sessions
        self._load_sessions()

        # Ensure default session exists
        if not any(s.is_default for s in self.sessions.values()):
            self._create_default_session()

        # Set current to default
        default = next((s for s in self.sessions.values() if s.is_default), None)
        if default:
            self.current_session_id = default.id

    def _load_sessions(self) -> None:
        """Load sessions from disk."""
        if not self._sessions_file.exists():
            return

        try:
            with open(self._sessions_file, "r") as f:
                data = json.load(f)
                for session_data in data.get("sessions", []):
                    session = Session.from_dict(session_data)
                    # Only load if blend file still exists
                    if Path(session.blend_file).exists():
                        self.sessions[session.id] = session
                    else:
                        logger.warning(f"Session blend file missing: {session.blend_file}")
        except Exception as e:
            logger.error(f"Failed to load sessions: {e}")

    def _save_sessions(self) -> None:
        """Persist sessions to disk."""
        try:
            data = {
                "sessions": [s.to_dict() for s in self.sessions.values()],
                "current": self.current_session_id,
            }
            with open(self._sessions_file, "w") as f:
                json.dump(data, f, indent=2)
        except Exception as e:
            logger.error(f"Failed to save sessions: {e}")

    def _create_default_session(self) -> Session:
        """Create the default (ephemeral) session."""
        session_id = "default"
        blend_file = self._sessions_dir / "default.blend"
        now = datetime.now(timezone.utc).isoformat()

        session = Session(
            id=session_id,
            name="Default",
            blend_file=str(blend_file),
            created_at=now,
            modified_at=now,
            is_default=True,
        )

        self.sessions[session_id] = session
        self._save_sessions()
        return session

    def create(self, name: str, metadata: Optional[dict[str, Any]] = None) -> Session:
        """
        Create a new named session.

        Args:
            name: Human-readable session name
            metadata: Optional metadata to attach

        Returns:
            Created session
        """
        session_id = str(uuid.uuid4())[:8]
        blend_file = self._sessions_dir / f"{session_id}.blend"
        now = datetime.now(timezone.utc).isoformat()

        session = Session(
            id=session_id,
            name=name,
            blend_file=str(blend_file),
            created_at=now,
            modified_at=now,
            is_default=False,
            metadata=metadata or {},
        )

        self.sessions[session_id] = session
        self._save_sessions()

        logger.info(f"Created session: {session_id} ({name})")
        return session

    def get(self, session_id: str) -> Optional[Session]:
        """Get session by ID."""
        return self.sessions.get(session_id)

    def get_current(self) -> Optional[Session]:
        """Get the current active session."""
        if self.current_session_id:
            return self.sessions.get(self.current_session_id)
        return None

    def switch(self, session_id: str) -> Session:
        """
        Switch to a different session.

        Args:
            session_id: Session ID to switch to

        Returns:
            The switched-to session

        Raises:
            ValueError: If session not found
        """
        session = self.sessions.get(session_id)
        if not session:
            raise ValueError(f"Session not found: {session_id}")

        self.current_session_id = session_id
        self._save_sessions()

        logger.info(f"Switched to session: {session_id}")
        return session

    def list(self) -> list[Session]:
        """List all sessions."""
        return list(self.sessions.values())

    def delete(self, session_id: str) -> bool:
        """
        Delete a session.

        Args:
            session_id: Session ID to delete

        Returns:
            True if deleted

        Raises:
            ValueError: If trying to delete default session
        """
        session = self.sessions.get(session_id)
        if not session:
            return False

        if session.is_default:
            raise ValueError("Cannot delete default session")

        # Delete blend file
        blend_path = Path(session.blend_file)
        if blend_path.exists():
            blend_path.unlink()

        # Remove from sessions
        del self.sessions[session_id]

        # Switch to default if this was current
        if self.current_session_id == session_id:
            default = next((s for s in self.sessions.values() if s.is_default), None)
            if default:
                self.current_session_id = default.id

        self._save_sessions()

        logger.info(f"Deleted session: {session_id}")
        return True

    def reset(self, session_id: Optional[str] = None) -> Session:
        """
        Reset a session to empty state.

        Args:
            session_id: Session to reset (default: current)

        Returns:
            The reset session
        """
        sid = session_id or self.current_session_id
        if not sid:
            raise ValueError("No session specified and no current session")

        session = self.sessions.get(sid)
        if not session:
            raise ValueError(f"Session not found: {sid}")

        # Delete blend file to reset
        blend_path = Path(session.blend_file)
        if blend_path.exists():
            blend_path.unlink()

        # Update modified time
        session.modified_at = datetime.now(timezone.utc).isoformat()
        self._save_sessions()

        logger.info(f"Reset session: {sid}")
        return session

    def update_modified(self, session_id: Optional[str] = None) -> None:
        """Update the modified timestamp for a session."""
        sid = session_id or self.current_session_id
        if sid and sid in self.sessions:
            self.sessions[sid].modified_at = datetime.now(timezone.utc).isoformat()
            self._save_sessions()

    def get_blend_file(self, session_id: Optional[str] = None) -> str:
        """Get the blend file path for a session."""
        sid = session_id or self.current_session_id
        if not sid:
            raise ValueError("No session specified and no current session")

        session = self.sessions.get(sid)
        if not session:
            raise ValueError(f"Session not found: {sid}")

        return session.blend_file
