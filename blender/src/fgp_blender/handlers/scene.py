"""
Scene Handler for FGP Blender Daemon.

Manages Blender scenes: create, load, save, info.

CHANGELOG:
01/17/2026 - Initial implementation (Claude)
"""

from __future__ import annotations

from pathlib import Path
from typing import Any, Optional

from fgp_daemon import MethodInfo, ParamInfo

from ..session import SessionManager


class SceneHandler:
    """Handler for scene operations."""

    def __init__(self, bpy: Any, sessions: SessionManager) -> None:
        """Initialize with Blender module and session manager."""
        self.bpy = bpy
        self.sessions = sessions

    def dispatch(self, category: str, action: str, params: dict[str, Any]) -> Any:
        """Dispatch scene methods."""
        handlers = {
            "new": self._new,
            "load": self._load,
            "save": self._save,
            "info": self._info,
            "clear": self._clear,
            "list": self._list,
        }

        handler = handlers.get(action)
        if handler is None:
            raise ValueError(f"Unknown scene action: {action}")

        return handler(params)

    def _new(self, params: dict[str, Any]) -> dict[str, Any]:
        """Create a new scene."""
        if not self.bpy:
            return {"created": True, "name": "Scene", "mock": True}

        name = params.get("name", "Scene")
        copy_settings = params.get("copy_settings", False)

        if copy_settings and self.bpy.context.scene:
            new_scene = self.bpy.data.scenes.new(name)
            # Copy render settings from active scene
            source = self.bpy.context.scene
            new_scene.render.resolution_x = source.render.resolution_x
            new_scene.render.resolution_y = source.render.resolution_y
            new_scene.render.fps = source.render.fps
        else:
            new_scene = self.bpy.data.scenes.new(name)

        # Make it active
        self.bpy.context.window.scene = new_scene
        self.sessions.update_modified()

        return {
            "created": True,
            "name": new_scene.name,
            "object_count": len(new_scene.objects),
        }

    def _load(self, params: dict[str, Any]) -> dict[str, Any]:
        """Load a .blend file."""
        if not self.bpy:
            return {"loaded": True, "mock": True}

        filepath = params.get("filepath")
        if not filepath:
            raise ValueError("filepath is required")

        path = Path(filepath).expanduser()
        if not path.exists():
            raise FileNotFoundError(f"File not found: {filepath}")

        self.bpy.ops.wm.open_mainfile(filepath=str(path))
        self.sessions.update_modified()

        scene = self.bpy.context.scene
        return {
            "loaded": True,
            "filepath": str(path),
            "scene": scene.name,
            "object_count": len(scene.objects),
        }

    def _save(self, params: dict[str, Any]) -> dict[str, Any]:
        """Save current scene to .blend file."""
        if not self.bpy:
            return {"saved": True, "mock": True}

        filepath = params.get("filepath")

        if filepath:
            path = Path(filepath).expanduser()
            path.parent.mkdir(parents=True, exist_ok=True)
            self.bpy.ops.wm.save_as_mainfile(filepath=str(path))
        else:
            # Save to current file
            if not self.bpy.data.is_saved:
                # Save to session blend file
                session_file = self.sessions.get_blend_file()
                Path(session_file).parent.mkdir(parents=True, exist_ok=True)
                self.bpy.ops.wm.save_as_mainfile(filepath=session_file)
            else:
                self.bpy.ops.wm.save_mainfile()

        self.sessions.update_modified()

        return {
            "saved": True,
            "filepath": self.bpy.data.filepath or self.sessions.get_blend_file(),
        }

    def _info(self, params: dict[str, Any]) -> dict[str, Any]:
        """Get current scene information."""
        if not self.bpy:
            return {
                "name": "Scene",
                "objects": [],
                "mock": True,
            }

        scene = self.bpy.context.scene

        return {
            "name": scene.name,
            "frame_start": scene.frame_start,
            "frame_end": scene.frame_end,
            "frame_current": scene.frame_current,
            "fps": scene.render.fps,
            "resolution": [scene.render.resolution_x, scene.render.resolution_y],
            "objects": [
                {
                    "name": obj.name,
                    "type": obj.type,
                    "visible": obj.visible_get(),
                }
                for obj in scene.objects
            ],
            "object_count": len(scene.objects),
            "world": scene.world.name if scene.world else None,
            "camera": scene.camera.name if scene.camera else None,
        }

    def _clear(self, params: dict[str, Any]) -> dict[str, Any]:
        """Clear all objects from scene."""
        if not self.bpy:
            return {"cleared": True, "mock": True}

        keep_camera = params.get("keep_camera", False)
        keep_lights = params.get("keep_lights", False)

        bpy = self.bpy
        deleted = 0

        for obj in list(bpy.context.scene.objects):
            if keep_camera and obj.type == "CAMERA":
                continue
            if keep_lights and obj.type == "LIGHT":
                continue

            bpy.data.objects.remove(obj, do_unlink=True)
            deleted += 1

        self.sessions.update_modified()

        return {"cleared": True, "deleted": deleted}

    def _list(self, params: dict[str, Any]) -> dict[str, Any]:
        """List all scenes in the file."""
        if not self.bpy:
            return {"scenes": [{"name": "Scene", "active": True}], "mock": True}

        active_scene = self.bpy.context.scene

        scenes = [
            {
                "name": scene.name,
                "object_count": len(scene.objects),
                "active": scene == active_scene,
            }
            for scene in self.bpy.data.scenes
        ]

        return {"scenes": scenes}

    def method_list(self) -> list[MethodInfo]:
        """Return available scene methods."""
        return [
            MethodInfo("scene.new", "Create a new scene", [
                ParamInfo("name", "string", False, "Scene"),
                ParamInfo("copy_settings", "boolean", False, False),
            ]),
            MethodInfo("scene.load", "Load a .blend file", [
                ParamInfo("filepath", "string", True),
            ]),
            MethodInfo("scene.save", "Save current scene", [
                ParamInfo("filepath", "string", False),
            ]),
            MethodInfo("scene.info", "Get scene information", []),
            MethodInfo("scene.clear", "Clear all objects from scene", [
                ParamInfo("keep_camera", "boolean", False, False),
                ParamInfo("keep_lights", "boolean", False, False),
            ]),
            MethodInfo("scene.list", "List all scenes", []),
        ]
