"""
Animation Handler for FGP Blender Daemon.

Handles animation keyframes and armatures.

CHANGELOG:
01/17/2026 - Initial implementation (Claude)
"""

from __future__ import annotations

import math
from typing import Any

from fgp_daemon import MethodInfo, ParamInfo

from ..session import SessionManager


class AnimationHandler:
    """Handler for animation operations."""

    def __init__(self, bpy: Any, sessions: SessionManager) -> None:
        """Initialize with Blender module and session manager."""
        self.bpy = bpy
        self.sessions = sessions

    def dispatch(self, category: str, action: str, params: dict[str, Any]) -> Any:
        """Dispatch animation/armature methods."""
        if category == "armature":
            return self._dispatch_armature(action, params)
        else:
            return self._dispatch_animation(action, params)

    def _dispatch_animation(self, action: str, params: dict[str, Any]) -> Any:
        """Handle animation methods."""
        # Handle nested actions like animation.keyframe.insert
        parts = action.split(".")
        if len(parts) > 1:
            sub = parts[0]
            sub_action = parts[1]
        else:
            sub = action
            sub_action = None

        if sub == "keyframe":
            return self._keyframe(sub_action, params)
        elif sub == "set_frame":
            return self._set_frame(params)
        elif sub == "set_range":
            return self._set_range(params)
        elif sub == "info":
            return self._info(params)
        elif sub == "clear":
            return self._clear(params)
        elif sub == "bake":
            return self._bake(params)
        else:
            raise ValueError(f"Unknown animation action: {action}")

    def _dispatch_armature(self, action: str, params: dict[str, Any]) -> Any:
        """Handle armature methods."""
        handlers = {
            "create": self._armature_create,
            "add_bone": self._armature_add_bone,
            "pose": self._armature_pose,
            "list_bones": self._armature_list_bones,
        }

        handler = handlers.get(action)
        if handler is None:
            raise ValueError(f"Unknown armature action: {action}")

        return handler(params)

    # =========================================================================
    # Animation Methods
    # =========================================================================

    def _keyframe(self, action: str, params: dict[str, Any]) -> dict[str, Any]:
        """Handle keyframe operations."""
        if not self.bpy:
            return {"success": True, "mock": True}

        bpy = self.bpy

        if action == "insert":
            object_name = params.get("object")
            data_path = params.get("data_path", "location")
            frame = params.get("frame")
            index = params.get("index", -1)  # -1 means all indices

            if not object_name:
                raise ValueError("object is required")

            obj = bpy.data.objects.get(object_name)
            if not obj:
                raise ValueError(f"Object not found: {object_name}")

            # Set frame if specified
            if frame is not None:
                bpy.context.scene.frame_set(frame)
            else:
                frame = bpy.context.scene.frame_current

            # Insert keyframe
            obj.keyframe_insert(data_path=data_path, index=index, frame=frame)
            self.sessions.update_modified()

            return {
                "inserted": True,
                "object": object_name,
                "data_path": data_path,
                "frame": frame,
            }

        elif action == "delete":
            object_name = params.get("object")
            data_path = params.get("data_path", "location")
            frame = params.get("frame")
            index = params.get("index", -1)

            if not object_name:
                raise ValueError("object is required")

            obj = bpy.data.objects.get(object_name)
            if not obj:
                raise ValueError(f"Object not found: {object_name}")

            if frame is not None:
                bpy.context.scene.frame_set(frame)
            else:
                frame = bpy.context.scene.frame_current

            obj.keyframe_delete(data_path=data_path, index=index, frame=frame)
            self.sessions.update_modified()

            return {
                "deleted": True,
                "object": object_name,
                "data_path": data_path,
                "frame": frame,
            }

        elif action == "list":
            object_name = params.get("object")

            if not object_name:
                raise ValueError("object is required")

            obj = bpy.data.objects.get(object_name)
            if not obj:
                raise ValueError(f"Object not found: {object_name}")

            keyframes = []

            if obj.animation_data and obj.animation_data.action:
                for fcurve in obj.animation_data.action.fcurves:
                    for keyframe in fcurve.keyframe_points:
                        keyframes.append({
                            "frame": int(keyframe.co[0]),
                            "value": keyframe.co[1],
                            "data_path": fcurve.data_path,
                            "index": fcurve.array_index,
                        })

            return {"object": object_name, "keyframes": keyframes}

        else:
            raise ValueError(f"Unknown keyframe action: {action}")

    def _set_frame(self, params: dict[str, Any]) -> dict[str, Any]:
        """Set current frame."""
        if not self.bpy:
            return {"set": True, "mock": True}

        frame = params.get("frame")
        if frame is None:
            raise ValueError("frame is required")

        self.bpy.context.scene.frame_set(frame)

        return {"set": True, "frame": frame}

    def _set_range(self, params: dict[str, Any]) -> dict[str, Any]:
        """Set animation frame range."""
        if not self.bpy:
            return {"set": True, "mock": True}

        scene = self.bpy.context.scene

        if "start" in params:
            scene.frame_start = params["start"]
        if "end" in params:
            scene.frame_end = params["end"]
        if "fps" in params:
            scene.render.fps = params["fps"]

        self.sessions.update_modified()

        return {
            "set": True,
            "start": scene.frame_start,
            "end": scene.frame_end,
            "fps": scene.render.fps,
        }

    def _info(self, params: dict[str, Any]) -> dict[str, Any]:
        """Get animation information."""
        if not self.bpy:
            return {"mock": True}

        scene = self.bpy.context.scene

        return {
            "frame_current": scene.frame_current,
            "frame_start": scene.frame_start,
            "frame_end": scene.frame_end,
            "fps": scene.render.fps,
            "duration_seconds": (scene.frame_end - scene.frame_start + 1) / scene.render.fps,
        }

    def _clear(self, params: dict[str, Any]) -> dict[str, Any]:
        """Clear animation data from object."""
        if not self.bpy:
            return {"cleared": True, "mock": True}

        object_name = params.get("object")
        if not object_name:
            raise ValueError("object is required")

        obj = self.bpy.data.objects.get(object_name)
        if not obj:
            raise ValueError(f"Object not found: {object_name}")

        if obj.animation_data:
            obj.animation_data_clear()

        self.sessions.update_modified()

        return {"cleared": True, "object": object_name}

    def _bake(self, params: dict[str, Any]) -> dict[str, Any]:
        """Bake animation to keyframes."""
        if not self.bpy:
            return {"baked": True, "mock": True}

        bpy = self.bpy
        object_name = params.get("object")
        frame_start = params.get("frame_start", bpy.context.scene.frame_start)
        frame_end = params.get("frame_end", bpy.context.scene.frame_end)

        if not object_name:
            raise ValueError("object is required")

        obj = bpy.data.objects.get(object_name)
        if not obj:
            raise ValueError(f"Object not found: {object_name}")

        # Select object and bake
        bpy.ops.object.select_all(action="DESELECT")
        obj.select_set(True)
        bpy.context.view_layer.objects.active = obj

        bpy.ops.nla.bake(
            frame_start=frame_start,
            frame_end=frame_end,
            only_selected=True,
            visual_keying=True,
            clear_constraints=False,
            bake_types={"OBJECT"},
        )

        self.sessions.update_modified()

        return {
            "baked": True,
            "object": object_name,
            "frame_start": frame_start,
            "frame_end": frame_end,
        }

    # =========================================================================
    # Armature Methods
    # =========================================================================

    def _armature_create(self, params: dict[str, Any]) -> dict[str, Any]:
        """Create a new armature."""
        if not self.bpy:
            return {"created": True, "name": "Armature", "mock": True}

        bpy = self.bpy
        name = params.get("name", "Armature")
        location = params.get("location", [0, 0, 0])

        # Create armature data
        arm_data = bpy.data.armatures.new(name=name)
        arm_obj = bpy.data.objects.new(name=name, object_data=arm_data)

        bpy.context.scene.collection.objects.link(arm_obj)
        arm_obj.location = location

        self.sessions.update_modified()

        return {
            "created": True,
            "name": arm_obj.name,
            "location": list(arm_obj.location),
        }

    def _armature_add_bone(self, params: dict[str, Any]) -> dict[str, Any]:
        """Add a bone to an armature."""
        if not self.bpy:
            return {"added": True, "mock": True}

        bpy = self.bpy
        armature_name = params.get("armature")
        bone_name = params.get("name", "Bone")
        head = params.get("head", [0, 0, 0])
        tail = params.get("tail", [0, 0, 1])
        parent = params.get("parent")

        if not armature_name:
            raise ValueError("armature is required")

        arm_obj = bpy.data.objects.get(armature_name)
        if not arm_obj or arm_obj.type != "ARMATURE":
            raise ValueError(f"Armature not found: {armature_name}")

        # Enter edit mode
        bpy.context.view_layer.objects.active = arm_obj
        bpy.ops.object.mode_set(mode="EDIT")

        try:
            # Create bone
            bone = arm_obj.data.edit_bones.new(bone_name)
            bone.head = head
            bone.tail = tail

            if parent:
                parent_bone = arm_obj.data.edit_bones.get(parent)
                if parent_bone:
                    bone.parent = parent_bone

            bone_name = bone.name  # May be renamed to avoid duplicates

        finally:
            bpy.ops.object.mode_set(mode="OBJECT")

        self.sessions.update_modified()

        return {
            "added": True,
            "armature": armature_name,
            "bone": bone_name,
            "head": head,
            "tail": tail,
        }

    def _armature_pose(self, params: dict[str, Any]) -> dict[str, Any]:
        """Set bone pose."""
        if not self.bpy:
            return {"set": True, "mock": True}

        bpy = self.bpy
        armature_name = params.get("armature")
        bone_name = params.get("bone")
        rotation = params.get("rotation")  # Euler degrees
        location = params.get("location")

        if not armature_name or not bone_name:
            raise ValueError("armature and bone are required")

        arm_obj = bpy.data.objects.get(armature_name)
        if not arm_obj or arm_obj.type != "ARMATURE":
            raise ValueError(f"Armature not found: {armature_name}")

        # Enter pose mode
        bpy.context.view_layer.objects.active = arm_obj
        bpy.ops.object.mode_set(mode="POSE")

        try:
            pose_bone = arm_obj.pose.bones.get(bone_name)
            if not pose_bone:
                raise ValueError(f"Bone not found: {bone_name}")

            if rotation:
                pose_bone.rotation_euler = [math.radians(r) for r in rotation]
            if location:
                pose_bone.location = location

        finally:
            bpy.ops.object.mode_set(mode="OBJECT")

        self.sessions.update_modified()

        return {
            "set": True,
            "armature": armature_name,
            "bone": bone_name,
        }

    def _armature_list_bones(self, params: dict[str, Any]) -> dict[str, Any]:
        """List bones in an armature."""
        if not self.bpy:
            return {"bones": [], "mock": True}

        armature_name = params.get("armature")
        if not armature_name:
            raise ValueError("armature is required")

        arm_obj = self.bpy.data.objects.get(armature_name)
        if not arm_obj or arm_obj.type != "ARMATURE":
            raise ValueError(f"Armature not found: {armature_name}")

        bones = [
            {
                "name": bone.name,
                "head": list(bone.head_local),
                "tail": list(bone.tail_local),
                "parent": bone.parent.name if bone.parent else None,
            }
            for bone in arm_obj.data.bones
        ]

        return {"armature": armature_name, "bones": bones, "count": len(bones)}

    def method_list(self) -> list[MethodInfo]:
        """Return available animation methods."""
        return [
            MethodInfo("animation.keyframe.insert", "Insert keyframe", [
                ParamInfo("object", "string", True),
                ParamInfo("data_path", "string", False, "location"),
                ParamInfo("frame", "integer", False),
                ParamInfo("index", "integer", False, -1),
            ]),
            MethodInfo("animation.keyframe.delete", "Delete keyframe", [
                ParamInfo("object", "string", True),
                ParamInfo("data_path", "string", False, "location"),
                ParamInfo("frame", "integer", False),
                ParamInfo("index", "integer", False, -1),
            ]),
            MethodInfo("animation.keyframe.list", "List keyframes", [
                ParamInfo("object", "string", True),
            ]),
            MethodInfo("animation.set_frame", "Set current frame", [
                ParamInfo("frame", "integer", True),
            ]),
            MethodInfo("animation.set_range", "Set frame range", [
                ParamInfo("start", "integer", False),
                ParamInfo("end", "integer", False),
                ParamInfo("fps", "integer", False),
            ]),
            MethodInfo("animation.info", "Get animation info", []),
            MethodInfo("animation.clear", "Clear animation from object", [
                ParamInfo("object", "string", True),
            ]),
            MethodInfo("animation.bake", "Bake animation to keyframes", [
                ParamInfo("object", "string", True),
                ParamInfo("frame_start", "integer", False),
                ParamInfo("frame_end", "integer", False),
            ]),
            MethodInfo("armature.create", "Create armature", [
                ParamInfo("name", "string", False, "Armature"),
                ParamInfo("location", "array", False, [0, 0, 0]),
            ]),
            MethodInfo("armature.add_bone", "Add bone to armature", [
                ParamInfo("armature", "string", True),
                ParamInfo("name", "string", False, "Bone"),
                ParamInfo("head", "array", False, [0, 0, 0]),
                ParamInfo("tail", "array", False, [0, 0, 1]),
                ParamInfo("parent", "string", False),
            ]),
            MethodInfo("armature.pose", "Set bone pose", [
                ParamInfo("armature", "string", True),
                ParamInfo("bone", "string", True),
                ParamInfo("rotation", "array", False),
                ParamInfo("location", "array", False),
            ]),
            MethodInfo("armature.list_bones", "List bones in armature", [
                ParamInfo("armature", "string", True),
            ]),
        ]
