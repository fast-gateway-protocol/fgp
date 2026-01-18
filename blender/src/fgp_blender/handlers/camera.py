"""
Camera and Light Handler for FGP Blender Daemon.

Handles camera and lighting operations.

CHANGELOG:
01/17/2026 - Initial implementation (Claude)
"""

from __future__ import annotations

import math
from typing import Any

from fgp_daemon import MethodInfo, ParamInfo

from ..session import SessionManager


class CameraHandler:
    """Handler for camera and light operations."""

    def __init__(self, bpy: Any, sessions: SessionManager) -> None:
        """Initialize with Blender module and session manager."""
        self.bpy = bpy
        self.sessions = sessions

    def dispatch(self, category: str, action: str, params: dict[str, Any]) -> Any:
        """Dispatch camera/light methods."""
        if category == "camera":
            return self._dispatch_camera(action, params)
        elif category == "light":
            return self._dispatch_light(action, params)
        else:
            raise ValueError(f"Unknown category: {category}")

    def _dispatch_camera(self, action: str, params: dict[str, Any]) -> Any:
        """Handle camera methods."""
        handlers = {
            "create": self._camera_create,
            "set_active": self._camera_set_active,
            "look_at": self._camera_look_at,
            "set_lens": self._camera_set_lens,
            "list": self._camera_list,
            "info": self._camera_info,
        }

        handler = handlers.get(action)
        if handler is None:
            raise ValueError(f"Unknown camera action: {action}")

        return handler(params)

    def _dispatch_light(self, action: str, params: dict[str, Any]) -> Any:
        """Handle light methods."""
        handlers = {
            "create": self._light_create,
            "set_color": self._light_set_color,
            "set_energy": self._light_set_energy,
            "hdri": self._light_hdri,
            "list": self._light_list,
            "info": self._light_info,
        }

        handler = handlers.get(action)
        if handler is None:
            raise ValueError(f"Unknown light action: {action}")

        return handler(params)

    # =========================================================================
    # Camera Methods
    # =========================================================================

    def _camera_create(self, params: dict[str, Any]) -> dict[str, Any]:
        """Create a new camera."""
        if not self.bpy:
            return {"created": True, "name": "Camera", "mock": True}

        bpy = self.bpy
        name = params.get("name", "Camera")
        location = params.get("location", [0, -10, 5])
        rotation = params.get("rotation", [60, 0, 0])  # Degrees
        lens = params.get("lens", 50)  # mm
        set_active = params.get("set_active", True)

        # Create camera data and object
        cam_data = bpy.data.cameras.new(name=name)
        cam_data.lens = lens

        cam_obj = bpy.data.objects.new(name=name, object_data=cam_data)
        bpy.context.scene.collection.objects.link(cam_obj)

        # Set transform
        cam_obj.location = location
        cam_obj.rotation_euler = [math.radians(r) for r in rotation]

        # Set as active camera
        if set_active:
            bpy.context.scene.camera = cam_obj

        self.sessions.update_modified()

        return {
            "created": True,
            "name": cam_obj.name,
            "location": list(cam_obj.location),
            "rotation": [math.degrees(r) for r in cam_obj.rotation_euler],
            "lens": cam_data.lens,
            "active": set_active,
        }

    def _camera_set_active(self, params: dict[str, Any]) -> dict[str, Any]:
        """Set the active camera."""
        if not self.bpy:
            return {"set": True, "mock": True}

        name = params.get("name")
        if not name:
            raise ValueError("name is required")

        cam = self.bpy.data.objects.get(name)
        if not cam or cam.type != "CAMERA":
            raise ValueError(f"Camera not found: {name}")

        self.bpy.context.scene.camera = cam
        self.sessions.update_modified()

        return {"set": True, "name": name}

    def _camera_look_at(self, params: dict[str, Any]) -> dict[str, Any]:
        """Point camera at a target location or object."""
        if not self.bpy:
            return {"set": True, "mock": True}

        bpy = self.bpy
        camera_name = params.get("camera")
        target = params.get("target")  # [x, y, z] or object name

        if not camera_name:
            # Use active camera
            if bpy.context.scene.camera:
                camera_name = bpy.context.scene.camera.name
            else:
                raise ValueError("No active camera and no camera specified")

        cam = bpy.data.objects.get(camera_name)
        if not cam or cam.type != "CAMERA":
            raise ValueError(f"Camera not found: {camera_name}")

        # Determine target location
        if isinstance(target, list) and len(target) == 3:
            target_loc = target
        elif isinstance(target, str):
            target_obj = bpy.data.objects.get(target)
            if not target_obj:
                raise ValueError(f"Target object not found: {target}")
            target_loc = list(target_obj.location)
        else:
            raise ValueError("target must be [x, y, z] or object name")

        # Calculate direction and rotation
        import mathutils
        direction = mathutils.Vector(target_loc) - cam.location
        rot_quat = direction.to_track_quat("-Z", "Y")
        cam.rotation_euler = rot_quat.to_euler()

        self.sessions.update_modified()

        return {
            "set": True,
            "camera": camera_name,
            "target": target_loc,
            "rotation": [math.degrees(r) for r in cam.rotation_euler],
        }

    def _camera_set_lens(self, params: dict[str, Any]) -> dict[str, Any]:
        """Set camera lens properties."""
        if not self.bpy:
            return {"set": True, "mock": True}

        camera_name = params.get("camera")

        if not camera_name:
            if self.bpy.context.scene.camera:
                camera_name = self.bpy.context.scene.camera.name
            else:
                raise ValueError("No active camera and no camera specified")

        cam = self.bpy.data.objects.get(camera_name)
        if not cam or cam.type != "CAMERA":
            raise ValueError(f"Camera not found: {camera_name}")

        cam_data = cam.data

        if "lens" in params:
            cam_data.lens = params["lens"]
        if "lens_unit" in params:
            cam_data.lens_unit = params["lens_unit"]
        if "sensor_width" in params:
            cam_data.sensor_width = params["sensor_width"]
        if "clip_start" in params:
            cam_data.clip_start = params["clip_start"]
        if "clip_end" in params:
            cam_data.clip_end = params["clip_end"]
        if "dof_focus_distance" in params:
            cam_data.dof.focus_distance = params["dof_focus_distance"]
            cam_data.dof.use_dof = True

        self.sessions.update_modified()

        return {
            "set": True,
            "camera": camera_name,
            "lens": cam_data.lens,
        }

    def _camera_list(self, params: dict[str, Any]) -> dict[str, Any]:
        """List all cameras."""
        if not self.bpy:
            return {"cameras": [], "mock": True}

        active = self.bpy.context.scene.camera

        cameras = [
            {
                "name": obj.name,
                "active": obj == active,
                "location": list(obj.location),
                "lens": obj.data.lens,
            }
            for obj in self.bpy.data.objects
            if obj.type == "CAMERA"
        ]

        return {"cameras": cameras, "count": len(cameras)}

    def _camera_info(self, params: dict[str, Any]) -> dict[str, Any]:
        """Get camera information."""
        if not self.bpy:
            return {"name": "Camera", "mock": True}

        camera_name = params.get("name")

        if not camera_name:
            if self.bpy.context.scene.camera:
                camera_name = self.bpy.context.scene.camera.name
            else:
                raise ValueError("No active camera and no camera specified")

        cam = self.bpy.data.objects.get(camera_name)
        if not cam or cam.type != "CAMERA":
            raise ValueError(f"Camera not found: {camera_name}")

        cam_data = cam.data

        return {
            "name": cam.name,
            "location": list(cam.location),
            "rotation": [math.degrees(r) for r in cam.rotation_euler],
            "lens": cam_data.lens,
            "lens_unit": cam_data.lens_unit,
            "sensor_width": cam_data.sensor_width,
            "clip_start": cam_data.clip_start,
            "clip_end": cam_data.clip_end,
            "active": cam == self.bpy.context.scene.camera,
        }

    # =========================================================================
    # Light Methods
    # =========================================================================

    def _light_create(self, params: dict[str, Any]) -> dict[str, Any]:
        """Create a new light."""
        if not self.bpy:
            return {"created": True, "name": "Light", "mock": True}

        bpy = self.bpy
        light_type = params.get("type", "POINT").upper()
        name = params.get("name", light_type.title())
        location = params.get("location", [0, 0, 5])
        energy = params.get("energy", 1000)
        color = params.get("color", [1, 1, 1])

        # Create light data
        light_data = bpy.data.lights.new(name=name, type=light_type)
        light_data.energy = energy
        light_data.color = color[:3]

        # Type-specific settings
        if light_type == "SUN":
            light_data.energy = params.get("energy", 5)
            if "angle" in params:
                light_data.angle = math.radians(params["angle"])
        elif light_type == "SPOT":
            if "spot_size" in params:
                light_data.spot_size = math.radians(params["spot_size"])
            if "spot_blend" in params:
                light_data.spot_blend = params["spot_blend"]
        elif light_type == "AREA":
            if "size" in params:
                light_data.size = params["size"]

        # Create object
        light_obj = bpy.data.objects.new(name=name, object_data=light_data)
        bpy.context.scene.collection.objects.link(light_obj)

        light_obj.location = location

        if "rotation" in params:
            light_obj.rotation_euler = [math.radians(r) for r in params["rotation"]]

        self.sessions.update_modified()

        return {
            "created": True,
            "name": light_obj.name,
            "type": light_type,
            "location": list(light_obj.location),
            "energy": light_data.energy,
            "color": list(light_data.color),
        }

    def _light_set_color(self, params: dict[str, Any]) -> dict[str, Any]:
        """Set light color."""
        if not self.bpy:
            return {"set": True, "mock": True}

        name = params.get("name")
        color = params.get("color")

        if not name or not color:
            raise ValueError("name and color are required")

        light = self.bpy.data.objects.get(name)
        if not light or light.type != "LIGHT":
            raise ValueError(f"Light not found: {name}")

        light.data.color = color[:3]
        self.sessions.update_modified()

        return {"set": True, "name": name, "color": list(light.data.color)}

    def _light_set_energy(self, params: dict[str, Any]) -> dict[str, Any]:
        """Set light energy/power."""
        if not self.bpy:
            return {"set": True, "mock": True}

        name = params.get("name")
        energy = params.get("energy")

        if not name or energy is None:
            raise ValueError("name and energy are required")

        light = self.bpy.data.objects.get(name)
        if not light or light.type != "LIGHT":
            raise ValueError(f"Light not found: {name}")

        light.data.energy = energy
        self.sessions.update_modified()

        return {"set": True, "name": name, "energy": energy}

    def _light_hdri(self, params: dict[str, Any]) -> dict[str, Any]:
        """Set HDRI environment lighting."""
        if not self.bpy:
            return {"set": True, "mock": True}

        bpy = self.bpy
        filepath = params.get("filepath")
        strength = params.get("strength", 1.0)
        rotation = params.get("rotation", 0)  # Degrees

        if not filepath:
            raise ValueError("filepath is required")

        # Get or create world
        world = bpy.context.scene.world
        if not world:
            world = bpy.data.worlds.new("World")
            bpy.context.scene.world = world

        world.use_nodes = True
        nodes = world.node_tree.nodes
        links = world.node_tree.links

        # Clear existing nodes
        nodes.clear()

        # Create nodes for HDRI
        output = nodes.new(type="ShaderNodeOutputWorld")
        output.location = (300, 0)

        background = nodes.new(type="ShaderNodeBackground")
        background.location = (0, 0)
        background.inputs["Strength"].default_value = strength

        env_texture = nodes.new(type="ShaderNodeTexEnvironment")
        env_texture.location = (-300, 0)

        mapping = nodes.new(type="ShaderNodeMapping")
        mapping.location = (-500, 0)
        mapping.inputs["Rotation"].default_value[2] = math.radians(rotation)

        tex_coord = nodes.new(type="ShaderNodeTexCoord")
        tex_coord.location = (-700, 0)

        # Load HDRI image
        from pathlib import Path
        path = Path(filepath).expanduser()
        if not path.exists():
            raise FileNotFoundError(f"HDRI not found: {filepath}")

        image = bpy.data.images.load(str(path))
        env_texture.image = image

        # Connect nodes
        links.new(tex_coord.outputs["Generated"], mapping.inputs["Vector"])
        links.new(mapping.outputs["Vector"], env_texture.inputs["Vector"])
        links.new(env_texture.outputs["Color"], background.inputs["Color"])
        links.new(background.outputs["Background"], output.inputs["Surface"])

        self.sessions.update_modified()

        return {
            "set": True,
            "filepath": str(path),
            "strength": strength,
            "rotation": rotation,
        }

    def _light_list(self, params: dict[str, Any]) -> dict[str, Any]:
        """List all lights."""
        if not self.bpy:
            return {"lights": [], "mock": True}

        lights = [
            {
                "name": obj.name,
                "type": obj.data.type,
                "location": list(obj.location),
                "energy": obj.data.energy,
                "color": list(obj.data.color),
            }
            for obj in self.bpy.data.objects
            if obj.type == "LIGHT"
        ]

        return {"lights": lights, "count": len(lights)}

    def _light_info(self, params: dict[str, Any]) -> dict[str, Any]:
        """Get light information."""
        if not self.bpy:
            return {"name": "Light", "mock": True}

        name = params.get("name")
        if not name:
            raise ValueError("name is required")

        light = self.bpy.data.objects.get(name)
        if not light or light.type != "LIGHT":
            raise ValueError(f"Light not found: {name}")

        return {
            "name": light.name,
            "type": light.data.type,
            "location": list(light.location),
            "rotation": [math.degrees(r) for r in light.rotation_euler],
            "energy": light.data.energy,
            "color": list(light.data.color),
        }

    def method_list(self) -> list[MethodInfo]:
        """Return available camera/light methods."""
        return [
            MethodInfo("camera.create", "Create a new camera", [
                ParamInfo("name", "string", False, "Camera"),
                ParamInfo("location", "array", False, [0, -10, 5]),
                ParamInfo("rotation", "array", False, [60, 0, 0]),
                ParamInfo("lens", "number", False, 50),
                ParamInfo("set_active", "boolean", False, True),
            ]),
            MethodInfo("camera.set_active", "Set the active camera", [
                ParamInfo("name", "string", True),
            ]),
            MethodInfo("camera.look_at", "Point camera at target", [
                ParamInfo("camera", "string", False),
                ParamInfo("target", "any", True),
            ]),
            MethodInfo("camera.set_lens", "Set camera lens properties", [
                ParamInfo("camera", "string", False),
                ParamInfo("lens", "number", False),
                ParamInfo("clip_start", "number", False),
                ParamInfo("clip_end", "number", False),
                ParamInfo("dof_focus_distance", "number", False),
            ]),
            MethodInfo("camera.list", "List all cameras", []),
            MethodInfo("camera.info", "Get camera information", [
                ParamInfo("name", "string", False),
            ]),
            MethodInfo("light.create", "Create a new light", [
                ParamInfo("type", "string", False, "POINT"),
                ParamInfo("name", "string", False),
                ParamInfo("location", "array", False, [0, 0, 5]),
                ParamInfo("energy", "number", False, 1000),
                ParamInfo("color", "array", False, [1, 1, 1]),
                ParamInfo("rotation", "array", False),
            ]),
            MethodInfo("light.set_color", "Set light color", [
                ParamInfo("name", "string", True),
                ParamInfo("color", "array", True),
            ]),
            MethodInfo("light.set_energy", "Set light energy", [
                ParamInfo("name", "string", True),
                ParamInfo("energy", "number", True),
            ]),
            MethodInfo("light.hdri", "Set HDRI environment", [
                ParamInfo("filepath", "string", True),
                ParamInfo("strength", "number", False, 1.0),
                ParamInfo("rotation", "number", False, 0),
            ]),
            MethodInfo("light.list", "List all lights", []),
            MethodInfo("light.info", "Get light information", [
                ParamInfo("name", "string", True),
            ]),
        ]
