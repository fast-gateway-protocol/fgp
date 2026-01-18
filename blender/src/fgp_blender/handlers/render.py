"""
Render Handler for FGP Blender Daemon.

Handles rendering with async job queue for non-blocking operations.

CHANGELOG:
01/17/2026 - Initial implementation (Claude)
"""

from __future__ import annotations

from pathlib import Path
from typing import Any, Callable

from fgp_daemon import MethodInfo, ParamInfo

from ..session import SessionManager
from ..jobs import JobQueue, JobType


class RenderHandler:
    """Handler for render operations."""

    def __init__(self, bpy: Any, sessions: SessionManager, jobs: JobQueue) -> None:
        """Initialize with Blender module, session manager, and job queue."""
        self.bpy = bpy
        self.sessions = sessions
        self.jobs = jobs

    def dispatch(self, category: str, action: str, params: dict[str, Any]) -> Any:
        """Dispatch render methods."""
        handlers = {
            "image": self._render_image,
            "animation": self._render_animation,
            "settings": self._settings,
            "set_engine": self._set_engine,
            "set_resolution": self._set_resolution,
            "set_samples": self._set_samples,
            "preview": self._preview,
        }

        handler = handlers.get(action)
        if handler is None:
            raise ValueError(f"Unknown render action: {action}")

        return handler(params)

    def _render_image(self, params: dict[str, Any]) -> dict[str, Any]:
        """Render a single image (async)."""
        output = params.get("output", "/tmp/render.png")
        wait = params.get("wait", False)

        # Create render job
        def render_handler(
            job_params: dict[str, Any],
            progress_cb: Callable[[float, str], None],
        ) -> dict[str, Any]:
            if not self.bpy:
                return {"rendered": True, "output": output, "mock": True}

            bpy = self.bpy

            # Set output path
            bpy.context.scene.render.filepath = output

            # Ensure output directory exists
            Path(output).parent.mkdir(parents=True, exist_ok=True)

            # Set up progress handler
            def frame_handler(scene, depsgraph):
                # Blender doesn't provide granular progress, estimate based on samples
                progress_cb(0.5, "Rendering...")

            # Render
            progress_cb(0.1, "Starting render...")
            bpy.ops.render.render(write_still=True)
            progress_cb(1.0, "Complete")

            return {
                "rendered": True,
                "output": output,
                "resolution": [
                    bpy.context.scene.render.resolution_x,
                    bpy.context.scene.render.resolution_y,
                ],
            }

        job_id = self.jobs.submit(
            job_type=JobType.RENDER_IMAGE,
            params={"output": output},
            handler=render_handler,
            session_id=self.sessions.current_session_id,
        )

        if wait:
            job = self.jobs.wait(job_id, timeout=params.get("timeout", 300))
            if job:
                return job.to_dict()
            return {"job_id": job_id, "status": "timeout"}

        return {"job_id": job_id, "status": "pending"}

    def _render_animation(self, params: dict[str, Any]) -> dict[str, Any]:
        """Render animation frames (async)."""
        output = params.get("output", "/tmp/render_")
        frame_start = params.get("frame_start")
        frame_end = params.get("frame_end")
        wait = params.get("wait", False)

        def animation_handler(
            job_params: dict[str, Any],
            progress_cb: Callable[[float, str], None],
        ) -> dict[str, Any]:
            if not self.bpy:
                return {"rendered": True, "mock": True}

            bpy = self.bpy
            scene = bpy.context.scene

            # Set frame range
            if frame_start is not None:
                scene.frame_start = frame_start
            if frame_end is not None:
                scene.frame_end = frame_end

            total_frames = scene.frame_end - scene.frame_start + 1

            # Set output path
            scene.render.filepath = output

            # Ensure output directory exists
            Path(output).parent.mkdir(parents=True, exist_ok=True)

            # Render each frame
            for frame in range(scene.frame_start, scene.frame_end + 1):
                scene.frame_set(frame)
                progress = (frame - scene.frame_start + 1) / total_frames
                progress_cb(progress, f"Frame {frame}/{scene.frame_end}")

                bpy.context.scene.render.filepath = f"{output}{frame:04d}.png"
                bpy.ops.render.render(write_still=True)

            return {
                "rendered": True,
                "output": output,
                "frames": total_frames,
                "frame_start": scene.frame_start,
                "frame_end": scene.frame_end,
            }

        job_id = self.jobs.submit(
            job_type=JobType.RENDER_ANIMATION,
            params={"output": output, "frame_start": frame_start, "frame_end": frame_end},
            handler=animation_handler,
            session_id=self.sessions.current_session_id,
        )

        if wait:
            job = self.jobs.wait(job_id, timeout=params.get("timeout", 3600))
            if job:
                return job.to_dict()
            return {"job_id": job_id, "status": "timeout"}

        return {"job_id": job_id, "status": "pending"}

    def _settings(self, params: dict[str, Any]) -> dict[str, Any]:
        """Get or set render settings."""
        if not self.bpy:
            return {"engine": "CYCLES", "mock": True}

        render = self.bpy.context.scene.render
        cycles = self.bpy.context.scene.cycles if hasattr(self.bpy.context.scene, "cycles") else None

        # Return current settings if no params to set
        if not any(k in params for k in ["engine", "resolution", "samples", "device"]):
            settings = {
                "engine": render.engine,
                "resolution": [render.resolution_x, render.resolution_y],
                "resolution_percentage": render.resolution_percentage,
                "film_transparent": render.film_transparent,
            }

            if cycles:
                settings["samples"] = cycles.samples
                settings["device"] = cycles.device

            return settings

        # Apply settings
        if "engine" in params:
            render.engine = params["engine"].upper()

        if "resolution" in params:
            res = params["resolution"]
            render.resolution_x = res[0]
            render.resolution_y = res[1]

        if "resolution_percentage" in params:
            render.resolution_percentage = params["resolution_percentage"]

        if "film_transparent" in params:
            render.film_transparent = params["film_transparent"]

        if cycles:
            if "samples" in params:
                cycles.samples = params["samples"]
            if "device" in params:
                cycles.device = params["device"].upper()

        self.sessions.update_modified()

        return {"updated": True}

    def _set_engine(self, params: dict[str, Any]) -> dict[str, Any]:
        """Set render engine."""
        if not self.bpy:
            return {"set": True, "mock": True}

        engine = params.get("engine", "CYCLES").upper()
        self.bpy.context.scene.render.engine = engine
        self.sessions.update_modified()

        return {"set": True, "engine": engine}

    def _set_resolution(self, params: dict[str, Any]) -> dict[str, Any]:
        """Set render resolution."""
        if not self.bpy:
            return {"set": True, "mock": True}

        render = self.bpy.context.scene.render

        if "width" in params:
            render.resolution_x = params["width"]
        if "height" in params:
            render.resolution_y = params["height"]
        if "percentage" in params:
            render.resolution_percentage = params["percentage"]

        self.sessions.update_modified()

        return {
            "set": True,
            "resolution": [render.resolution_x, render.resolution_y],
            "percentage": render.resolution_percentage,
        }

    def _set_samples(self, params: dict[str, Any]) -> dict[str, Any]:
        """Set render samples (for Cycles)."""
        if not self.bpy:
            return {"set": True, "mock": True}

        samples = params.get("samples", 128)

        if hasattr(self.bpy.context.scene, "cycles"):
            self.bpy.context.scene.cycles.samples = samples
            self.sessions.update_modified()
            return {"set": True, "samples": samples}
        else:
            return {"set": False, "error": "Cycles not available"}

    def _preview(self, params: dict[str, Any]) -> dict[str, Any]:
        """Quick preview render with reduced settings."""
        if not self.bpy:
            return {"rendered": True, "mock": True}

        bpy = self.bpy
        render = bpy.context.scene.render
        output = params.get("output", "/tmp/preview.png")

        # Store original settings
        orig_samples = None
        orig_percentage = render.resolution_percentage

        # Reduce settings for preview
        render.resolution_percentage = params.get("percentage", 50)

        if hasattr(bpy.context.scene, "cycles"):
            orig_samples = bpy.context.scene.cycles.samples
            bpy.context.scene.cycles.samples = params.get("samples", 32)

        try:
            # Ensure output directory exists
            Path(output).parent.mkdir(parents=True, exist_ok=True)

            render.filepath = output
            bpy.ops.render.render(write_still=True)

            return {
                "rendered": True,
                "output": output,
                "resolution": [
                    int(render.resolution_x * render.resolution_percentage / 100),
                    int(render.resolution_y * render.resolution_percentage / 100),
                ],
            }
        finally:
            # Restore settings
            render.resolution_percentage = orig_percentage
            if orig_samples is not None:
                bpy.context.scene.cycles.samples = orig_samples

    def method_list(self) -> list[MethodInfo]:
        """Return available render methods."""
        return [
            MethodInfo("render.image", "Render single image (async)", [
                ParamInfo("output", "string", False, "/tmp/render.png"),
                ParamInfo("wait", "boolean", False, False),
                ParamInfo("timeout", "number", False, 300),
            ]),
            MethodInfo("render.animation", "Render animation (async)", [
                ParamInfo("output", "string", False, "/tmp/render_"),
                ParamInfo("frame_start", "integer", False),
                ParamInfo("frame_end", "integer", False),
                ParamInfo("wait", "boolean", False, False),
                ParamInfo("timeout", "number", False, 3600),
            ]),
            MethodInfo("render.settings", "Get or set render settings", [
                ParamInfo("engine", "string", False),
                ParamInfo("resolution", "array", False),
                ParamInfo("samples", "integer", False),
                ParamInfo("device", "string", False),
                ParamInfo("film_transparent", "boolean", False),
            ]),
            MethodInfo("render.set_engine", "Set render engine", [
                ParamInfo("engine", "string", False, "CYCLES"),
            ]),
            MethodInfo("render.set_resolution", "Set render resolution", [
                ParamInfo("width", "integer", False),
                ParamInfo("height", "integer", False),
                ParamInfo("percentage", "integer", False, 100),
            ]),
            MethodInfo("render.set_samples", "Set Cycles samples", [
                ParamInfo("samples", "integer", False, 128),
            ]),
            MethodInfo("render.preview", "Quick preview render", [
                ParamInfo("output", "string", False, "/tmp/preview.png"),
                ParamInfo("percentage", "integer", False, 50),
                ParamInfo("samples", "integer", False, 32),
            ]),
        ]
