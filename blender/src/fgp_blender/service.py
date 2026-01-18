"""
Blender Service for FGP daemon.

Main service class that routes method calls to handlers.

CHANGELOG:
01/17/2026 - Initial implementation (Claude)
"""

from __future__ import annotations

import logging
from typing import Any, Optional

from fgp_daemon import FgpService, MethodInfo, ParamInfo, HealthStatus

from .session import SessionManager
from .jobs import JobQueue, JobStatus, JobType

logger = logging.getLogger(__name__)


class BlenderService(FgpService):
    """
    FGP Blender daemon service.

    Provides 55+ methods for comprehensive Blender automation:
    - Scene management
    - Object CRUD and transforms
    - Mesh operations and modifiers
    - Materials and shader nodes
    - Rendering (async with job queue)
    - Camera and lighting
    - Animation and keyframes
    - Import/export
    - Viewport operations
    - Python script execution

    The service maintains a warm Blender instance for sub-10ms latency.
    """

    name_val = "blender"
    version_val = "0.1.0"

    def __init__(self) -> None:
        """Initialize Blender service."""
        self._bpy: Optional[Any] = None  # Blender Python module
        self.sessions = SessionManager()
        self.jobs = JobQueue()

        # Handler instances (lazy loaded after bpy init)
        self._scene_handler: Optional[Any] = None
        self._object_handler: Optional[Any] = None
        self._mesh_handler: Optional[Any] = None
        self._material_handler: Optional[Any] = None
        self._render_handler: Optional[Any] = None
        self._camera_handler: Optional[Any] = None
        self._animation_handler: Optional[Any] = None
        self._io_handler: Optional[Any] = None
        self._physics_handler: Optional[Any] = None
        self._ai_handler: Optional[Any] = None
        self._gpencil_handler: Optional[Any] = None

    def name(self) -> str:
        """Return service name."""
        return self.name_val

    def version(self) -> str:
        """Return service version."""
        return self.version_val

    def on_start(self) -> None:
        """Initialize Blender and handlers."""
        # Import bpy (must be available when running inside Blender)
        try:
            import bpy
            self._bpy = bpy
            logger.info("Blender Python module loaded")
        except ImportError:
            logger.warning("bpy not available - running in mock mode")
            self._bpy = None

        # Initialize handlers
        from .handlers.scene import SceneHandler
        from .handlers.object import ObjectHandler
        from .handlers.mesh import MeshHandler
        from .handlers.material import MaterialHandler
        from .handlers.render import RenderHandler
        from .handlers.camera import CameraHandler
        from .handlers.animation import AnimationHandler
        from .handlers.io import IOHandler
        from .handlers.physics import PhysicsHandler
        from .handlers.ai_generation import AIGenerationHandler
        from .handlers.grease_pencil import GreasePencilHandler

        self._scene_handler = SceneHandler(self._bpy, self.sessions)
        self._object_handler = ObjectHandler(self._bpy, self.sessions)
        self._mesh_handler = MeshHandler(self._bpy, self.sessions)
        self._material_handler = MaterialHandler(self._bpy, self.sessions)
        self._render_handler = RenderHandler(self._bpy, self.sessions, self.jobs)
        self._camera_handler = CameraHandler(self._bpy, self.sessions)
        self._animation_handler = AnimationHandler(self._bpy, self.sessions)
        self._io_handler = IOHandler(self._bpy, self.sessions)
        self._physics_handler = PhysicsHandler(self._bpy, self.sessions)
        self._ai_handler = AIGenerationHandler(self._bpy, self.sessions, self.jobs)
        self._gpencil_handler = GreasePencilHandler(self._bpy, self.sessions)

        # Start job queue
        self.jobs.start()

        logger.info("Blender service started")

    def on_stop(self) -> None:
        """Clean up resources."""
        self.jobs.stop()
        logger.info("Blender service stopped")

    def health_check(self) -> dict[str, HealthStatus]:
        """Return health status of sub-services."""
        return {
            "blender": HealthStatus(
                ok=self._bpy is not None,
                message="Blender loaded" if self._bpy else "Running in mock mode",
            ),
            "sessions": HealthStatus(
                ok=True,
                message=f"{len(self.sessions.sessions)} sessions",
            ),
            "jobs": HealthStatus(
                ok=True,
                message=f"{len(self.jobs.list(status=JobStatus.RUNNING))} running",
            ),
        }

    def dispatch(self, method: str, params: dict[str, Any]) -> Any:
        """
        Route method calls to handlers.

        Args:
            method: Fully-qualified method name (e.g., 'blender.object.create')
            params: Method parameters

        Returns:
            Result data

        Raises:
            ValueError: For unknown methods
        """
        # Strip service prefix if present
        if method.startswith("blender."):
            method = method[8:]

        # Route to appropriate handler
        parts = method.split(".", 1)
        category = parts[0]
        action = parts[1] if len(parts) > 1 else ""

        # Session methods
        if category == "session":
            return self._dispatch_session(action, params)

        # Job methods
        if category == "job":
            return self._dispatch_job(action, params)

        # Python execution
        if category == "python":
            return self._dispatch_python(action, params)

        # Viewport methods
        if category == "viewport":
            return self._dispatch_viewport(action, params)

        # Handler-based methods
        handlers = {
            "scene": self._scene_handler,
            "object": self._object_handler,
            "mesh": self._mesh_handler,
            "material": self._material_handler,
            "shader": self._material_handler,  # Alias
            "render": self._render_handler,
            "camera": self._camera_handler,
            "light": self._camera_handler,  # Same handler
            "animation": self._animation_handler,
            "armature": self._animation_handler,  # Alias
            "import": self._io_handler,
            "export": self._io_handler,
            "assets": self._io_handler,  # Asset library
            # New handlers
            "physics": self._physics_handler,
            "ai": self._ai_handler,
            "gpencil": self._gpencil_handler,
        }

        handler = handlers.get(category)
        if handler is None:
            raise ValueError(f"Unknown method category: {category}")

        return handler.dispatch(category, action, params)

    def _dispatch_session(self, action: str, params: dict[str, Any]) -> Any:
        """Handle session methods."""
        if action == "new":
            name = params.get("name", "Untitled")
            metadata = params.get("metadata")
            session = self.sessions.create(name, metadata)
            return session.to_dict()

        elif action == "list":
            return {"sessions": [s.to_dict() for s in self.sessions.list()]}

        elif action == "switch":
            session_id = params.get("session_id")
            if not session_id:
                raise ValueError("session_id is required")
            session = self.sessions.switch(session_id)
            return session.to_dict()

        elif action == "current":
            session = self.sessions.get_current()
            return session.to_dict() if session else None

        elif action == "reset":
            session_id = params.get("session_id")
            session = self.sessions.reset(session_id)
            return session.to_dict()

        elif action == "delete":
            session_id = params.get("session_id")
            if not session_id:
                raise ValueError("session_id is required")
            deleted = self.sessions.delete(session_id)
            return {"deleted": deleted}

        else:
            raise ValueError(f"Unknown session action: {action}")

    def _dispatch_job(self, action: str, params: dict[str, Any]) -> Any:
        """Handle job methods."""
        if action == "status":
            job_id = params.get("job_id")
            if not job_id:
                raise ValueError("job_id is required")
            job = self.jobs.get(job_id)
            return job.to_dict() if job else None

        elif action == "list":
            status = params.get("status")
            if status:
                status = JobStatus(status)
            jobs = self.jobs.list(
                status=status,
                limit=params.get("limit", 50),
            )
            return {"jobs": [j.to_dict() for j in jobs]}

        elif action == "cancel":
            job_id = params.get("job_id")
            if not job_id:
                raise ValueError("job_id is required")
            cancelled = self.jobs.cancel(job_id)
            return {"cancelled": cancelled}

        elif action == "wait":
            job_id = params.get("job_id")
            timeout = params.get("timeout", 30.0)
            if not job_id:
                raise ValueError("job_id is required")
            job = self.jobs.wait(job_id, timeout)
            return job.to_dict() if job else None

        else:
            raise ValueError(f"Unknown job action: {action}")

    def _dispatch_python(self, action: str, params: dict[str, Any]) -> Any:
        """Handle Python execution methods."""
        if not self._bpy:
            raise RuntimeError("Blender not available")

        if action == "exec":
            code = params.get("code")
            if not code:
                raise ValueError("code is required")
            # Execute in Blender context
            exec(code, {"bpy": self._bpy})
            return {"executed": True}

        elif action == "eval":
            expr = params.get("expr")
            if not expr:
                raise ValueError("expr is required")
            result = eval(expr, {"bpy": self._bpy})
            # Convert to JSON-safe value
            return {"result": _to_json_safe(result)}

        else:
            raise ValueError(f"Unknown python action: {action}")

    def _dispatch_viewport(self, action: str, params: dict[str, Any]) -> Any:
        """Handle viewport methods."""
        if not self._bpy:
            raise RuntimeError("Blender not available")

        bpy = self._bpy

        if action == "screenshot":
            output = params.get("output", "/tmp/viewport.png")
            # Render viewport to image
            for area in bpy.context.screen.areas:
                if area.type == "VIEW_3D":
                    with bpy.context.temp_override(area=area):
                        bpy.ops.screen.screenshot(filepath=output)
                        return {"saved": True, "path": output}
            raise RuntimeError("No 3D viewport found")

        elif action == "set":
            view = params.get("view", "FRONT")  # FRONT, BACK, LEFT, RIGHT, TOP, BOTTOM, CAMERA
            for area in bpy.context.screen.areas:
                if area.type == "VIEW_3D":
                    for region in area.regions:
                        if region.type == "WINDOW":
                            with bpy.context.temp_override(area=area, region=region):
                                bpy.ops.view3d.view_axis(type=view)
                                return {"view": view}
            raise RuntimeError("No 3D viewport found")

        elif action == "orbit":
            angle = params.get("angle", 45)
            axis = params.get("axis", "Z")  # X, Y, Z
            for area in bpy.context.screen.areas:
                if area.type == "VIEW_3D":
                    for region in area.regions:
                        if region.type == "WINDOW":
                            with bpy.context.temp_override(area=area, region=region):
                                if axis == "Z":
                                    bpy.ops.view3d.view_orbit(angle=angle, type="ORBITRIGHT")
                                elif axis == "X":
                                    bpy.ops.view3d.view_orbit(angle=angle, type="ORBITUP")
                                return {"orbited": True, "angle": angle, "axis": axis}
            raise RuntimeError("No 3D viewport found")

        else:
            raise ValueError(f"Unknown viewport action: {action}")

    def method_list(self) -> list[MethodInfo]:
        """Return list of all available methods."""
        methods: list[MethodInfo] = []

        # Session methods
        methods.extend([
            MethodInfo("session.new", "Create a new session", [
                ParamInfo("name", "string", False, "Untitled"),
                ParamInfo("metadata", "object", False),
            ]),
            MethodInfo("session.list", "List all sessions", []),
            MethodInfo("session.switch", "Switch to a session", [
                ParamInfo("session_id", "string", True),
            ]),
            MethodInfo("session.current", "Get current session", []),
            MethodInfo("session.reset", "Reset session to empty", [
                ParamInfo("session_id", "string", False),
            ]),
            MethodInfo("session.delete", "Delete a session", [
                ParamInfo("session_id", "string", True),
            ]),
        ])

        # Job methods
        methods.extend([
            MethodInfo("job.status", "Get job status", [
                ParamInfo("job_id", "string", True),
            ]),
            MethodInfo("job.list", "List jobs", [
                ParamInfo("status", "string", False),
                ParamInfo("limit", "integer", False, 50),
            ]),
            MethodInfo("job.cancel", "Cancel pending job", [
                ParamInfo("job_id", "string", True),
            ]),
            MethodInfo("job.wait", "Wait for job completion", [
                ParamInfo("job_id", "string", True),
                ParamInfo("timeout", "number", False, 30.0),
            ]),
        ])

        # Python methods
        methods.extend([
            MethodInfo("python.exec", "Execute Python code in Blender", [
                ParamInfo("code", "string", True),
            ]),
            MethodInfo("python.eval", "Evaluate Python expression", [
                ParamInfo("expr", "string", True),
            ]),
        ])

        # Viewport methods
        methods.extend([
            MethodInfo("viewport.screenshot", "Capture viewport screenshot", [
                ParamInfo("output", "string", False, "/tmp/viewport.png"),
            ]),
            MethodInfo("viewport.set", "Set viewport view angle", [
                ParamInfo("view", "string", False, "FRONT"),
            ]),
            MethodInfo("viewport.orbit", "Orbit viewport", [
                ParamInfo("angle", "number", False, 45),
                ParamInfo("axis", "string", False, "Z"),
            ]),
        ])

        # Add handler methods
        for handler in [
            self._scene_handler,
            self._object_handler,
            self._mesh_handler,
            self._material_handler,
            self._render_handler,
            self._camera_handler,
            self._animation_handler,
            self._io_handler,
            self._physics_handler,
            self._ai_handler,
            self._gpencil_handler,
        ]:
            if handler:
                methods.extend(handler.method_list())

        return methods


def _to_json_safe(value: Any) -> Any:
    """Convert Python value to JSON-safe representation."""
    if value is None or isinstance(value, (bool, int, float, str)):
        return value
    if isinstance(value, (list, tuple)):
        return [_to_json_safe(v) for v in value]
    if isinstance(value, dict):
        return {str(k): _to_json_safe(v) for k, v in value.items()}
    # Convert other objects to string representation
    return str(value)
