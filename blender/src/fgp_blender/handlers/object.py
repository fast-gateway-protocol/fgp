"""
Object Handler for FGP Blender Daemon.

Manages Blender objects: create, delete, transform, select.

CHANGELOG:
01/17/2026 - Initial implementation (Claude)
"""

from __future__ import annotations

import math
from typing import Any, Optional

from fgp_daemon import MethodInfo, ParamInfo

from ..session import SessionManager


class ObjectHandler:
    """Handler for object operations."""

    def __init__(self, bpy: Any, sessions: SessionManager) -> None:
        """Initialize with Blender module and session manager."""
        self.bpy = bpy
        self.sessions = sessions

    def dispatch(self, category: str, action: str, params: dict[str, Any]) -> Any:
        """Dispatch object methods."""
        handlers = {
            "create": self._create,
            "delete": self._delete,
            "list": self._list,
            "info": self._info,
            "transform": self._transform,
            "select": self._select,
            "duplicate": self._duplicate,
            "rename": self._rename,
            "parent": self._parent,
            "hide": self._hide,
        }

        handler = handlers.get(action)
        if handler is None:
            raise ValueError(f"Unknown object action: {action}")

        return handler(params)

    def _create(self, params: dict[str, Any]) -> dict[str, Any]:
        """Create a new object."""
        if not self.bpy:
            return {"created": True, "name": "Object", "mock": True}

        bpy = self.bpy
        obj_type = params.get("type", "CUBE").upper()
        name = params.get("name")
        location = params.get("location", [0, 0, 0])
        rotation = params.get("rotation", [0, 0, 0])  # Degrees
        scale = params.get("scale", [1, 1, 1])

        # Map type to creation method
        mesh_types = {
            "CUBE": lambda: bpy.ops.mesh.primitive_cube_add(
                size=params.get("size", 2),
                location=location,
            ),
            "SPHERE": lambda: bpy.ops.mesh.primitive_uv_sphere_add(
                radius=params.get("radius", 1),
                segments=params.get("segments", 32),
                ring_count=params.get("rings", 16),
                location=location,
            ),
            "CYLINDER": lambda: bpy.ops.mesh.primitive_cylinder_add(
                radius=params.get("radius", 1),
                depth=params.get("depth", 2),
                vertices=params.get("vertices", 32),
                location=location,
            ),
            "CONE": lambda: bpy.ops.mesh.primitive_cone_add(
                radius1=params.get("radius", 1),
                depth=params.get("depth", 2),
                vertices=params.get("vertices", 32),
                location=location,
            ),
            "TORUS": lambda: bpy.ops.mesh.primitive_torus_add(
                major_radius=params.get("major_radius", 1),
                minor_radius=params.get("minor_radius", 0.25),
                location=location,
            ),
            "PLANE": lambda: bpy.ops.mesh.primitive_plane_add(
                size=params.get("size", 2),
                location=location,
            ),
            "CIRCLE": lambda: bpy.ops.mesh.primitive_circle_add(
                radius=params.get("radius", 1),
                vertices=params.get("vertices", 32),
                location=location,
            ),
            "MONKEY": lambda: bpy.ops.mesh.primitive_monkey_add(
                size=params.get("size", 2),
                location=location,
            ),
            "EMPTY": lambda: bpy.ops.object.empty_add(
                type=params.get("empty_type", "PLAIN_AXES"),
                location=location,
            ),
            "TEXT": lambda: bpy.ops.object.text_add(location=location),
        }

        creator = mesh_types.get(obj_type)
        if creator is None:
            raise ValueError(f"Unknown object type: {obj_type}")

        creator()
        obj = bpy.context.active_object

        # Apply rotation (convert degrees to radians)
        obj.rotation_euler = [math.radians(r) for r in rotation]
        obj.scale = scale

        # Rename if specified
        if name:
            obj.name = name

        # Update text content if TEXT type
        if obj_type == "TEXT" and params.get("text"):
            obj.data.body = params["text"]

        self.sessions.update_modified()

        return {
            "created": True,
            "name": obj.name,
            "type": obj.type,
            "location": list(obj.location),
            "rotation": [math.degrees(r) for r in obj.rotation_euler],
            "scale": list(obj.scale),
        }

    def _delete(self, params: dict[str, Any]) -> dict[str, Any]:
        """Delete objects by name or selection."""
        if not self.bpy:
            return {"deleted": True, "count": 0, "mock": True}

        bpy = self.bpy
        names = params.get("names", [])
        delete_selected = params.get("selected", False)

        deleted = []

        if delete_selected:
            for obj in list(bpy.context.selected_objects):
                deleted.append(obj.name)
                bpy.data.objects.remove(obj, do_unlink=True)
        elif names:
            for name in names:
                obj = bpy.data.objects.get(name)
                if obj:
                    deleted.append(obj.name)
                    bpy.data.objects.remove(obj, do_unlink=True)
        else:
            raise ValueError("Specify 'names' list or 'selected': true")

        self.sessions.update_modified()

        return {"deleted": True, "names": deleted, "count": len(deleted)}

    def _list(self, params: dict[str, Any]) -> dict[str, Any]:
        """List objects in scene."""
        if not self.bpy:
            return {"objects": [], "count": 0, "mock": True}

        type_filter = params.get("type")
        selected_only = params.get("selected", False)

        if selected_only:
            objects = list(self.bpy.context.selected_objects)
        else:
            objects = list(self.bpy.context.scene.objects)

        if type_filter:
            objects = [o for o in objects if o.type == type_filter.upper()]

        result = [
            {
                "name": obj.name,
                "type": obj.type,
                "location": list(obj.location),
                "visible": obj.visible_get(),
                "selected": obj.select_get(),
            }
            for obj in objects
        ]

        return {"objects": result, "count": len(result)}

    def _info(self, params: dict[str, Any]) -> dict[str, Any]:
        """Get detailed object information."""
        if not self.bpy:
            return {"name": "Object", "mock": True}

        name = params.get("name")
        if not name:
            raise ValueError("name is required")

        obj = self.bpy.data.objects.get(name)
        if not obj:
            raise ValueError(f"Object not found: {name}")

        info = {
            "name": obj.name,
            "type": obj.type,
            "location": list(obj.location),
            "rotation": [math.degrees(r) for r in obj.rotation_euler],
            "scale": list(obj.scale),
            "dimensions": list(obj.dimensions),
            "visible": obj.visible_get(),
            "selected": obj.select_get(),
            "parent": obj.parent.name if obj.parent else None,
            "children": [c.name for c in obj.children],
        }

        # Add mesh-specific info
        if obj.type == "MESH":
            mesh = obj.data
            info["mesh"] = {
                "vertices": len(mesh.vertices),
                "edges": len(mesh.edges),
                "faces": len(mesh.polygons),
                "materials": [m.name for m in mesh.materials if m],
            }

        return info

    def _transform(self, params: dict[str, Any]) -> dict[str, Any]:
        """Transform an object (location, rotation, scale)."""
        if not self.bpy:
            return {"transformed": True, "mock": True}

        name = params.get("name")
        if not name:
            raise ValueError("name is required")

        obj = self.bpy.data.objects.get(name)
        if not obj:
            raise ValueError(f"Object not found: {name}")

        # Apply transforms
        if "location" in params:
            obj.location = params["location"]
        if "rotation" in params:
            obj.rotation_euler = [math.radians(r) for r in params["rotation"]]
        if "scale" in params:
            obj.scale = params["scale"]

        # Relative transforms
        if "translate" in params:
            t = params["translate"]
            obj.location = [obj.location[i] + t[i] for i in range(3)]
        if "rotate" in params:
            r = params["rotate"]
            obj.rotation_euler = [
                obj.rotation_euler[i] + math.radians(r[i]) for i in range(3)
            ]

        self.sessions.update_modified()

        return {
            "transformed": True,
            "name": obj.name,
            "location": list(obj.location),
            "rotation": [math.degrees(r) for r in obj.rotation_euler],
            "scale": list(obj.scale),
        }

    def _select(self, params: dict[str, Any]) -> dict[str, Any]:
        """Select or deselect objects."""
        if not self.bpy:
            return {"selected": True, "mock": True}

        bpy = self.bpy
        names = params.get("names", [])
        add = params.get("add", False)  # Add to selection or replace
        deselect_all = params.get("deselect_all", False)

        if deselect_all or not add:
            bpy.ops.object.select_all(action="DESELECT")

        if deselect_all:
            return {"selected": True, "names": [], "count": 0}

        selected = []
        for name in names:
            obj = bpy.data.objects.get(name)
            if obj:
                obj.select_set(True)
                selected.append(name)
                # Make last one active
                bpy.context.view_layer.objects.active = obj

        return {"selected": True, "names": selected, "count": len(selected)}

    def _duplicate(self, params: dict[str, Any]) -> dict[str, Any]:
        """Duplicate objects."""
        if not self.bpy:
            return {"duplicated": True, "names": [], "mock": True}

        bpy = self.bpy
        names = params.get("names", [])
        linked = params.get("linked", False)  # Link data or make copies

        if not names:
            raise ValueError("names list is required")

        # Select objects to duplicate
        bpy.ops.object.select_all(action="DESELECT")
        for name in names:
            obj = bpy.data.objects.get(name)
            if obj:
                obj.select_set(True)

        # Duplicate
        bpy.ops.object.duplicate(linked=linked)

        new_names = [obj.name for obj in bpy.context.selected_objects]
        self.sessions.update_modified()

        return {"duplicated": True, "names": new_names, "count": len(new_names)}

    def _rename(self, params: dict[str, Any]) -> dict[str, Any]:
        """Rename an object."""
        if not self.bpy:
            return {"renamed": True, "mock": True}

        old_name = params.get("name")
        new_name = params.get("new_name")

        if not old_name or not new_name:
            raise ValueError("name and new_name are required")

        obj = self.bpy.data.objects.get(old_name)
        if not obj:
            raise ValueError(f"Object not found: {old_name}")

        obj.name = new_name
        self.sessions.update_modified()

        return {"renamed": True, "old_name": old_name, "new_name": obj.name}

    def _parent(self, params: dict[str, Any]) -> dict[str, Any]:
        """Set parent-child relationship."""
        if not self.bpy:
            return {"parented": True, "mock": True}

        child_name = params.get("child")
        parent_name = params.get("parent")
        clear = params.get("clear", False)

        if not child_name:
            raise ValueError("child is required")

        child = self.bpy.data.objects.get(child_name)
        if not child:
            raise ValueError(f"Child object not found: {child_name}")

        if clear:
            child.parent = None
            return {"parented": False, "child": child_name, "parent": None}

        if not parent_name:
            raise ValueError("parent is required (or use clear: true)")

        parent = self.bpy.data.objects.get(parent_name)
        if not parent:
            raise ValueError(f"Parent object not found: {parent_name}")

        child.parent = parent
        self.sessions.update_modified()

        return {"parented": True, "child": child_name, "parent": parent_name}

    def _hide(self, params: dict[str, Any]) -> dict[str, Any]:
        """Hide or show objects."""
        if not self.bpy:
            return {"hidden": True, "mock": True}

        names = params.get("names", [])
        hide = params.get("hide", True)

        if not names:
            raise ValueError("names list is required")

        hidden = []
        for name in names:
            obj = self.bpy.data.objects.get(name)
            if obj:
                obj.hide_set(hide)
                hidden.append(name)

        self.sessions.update_modified()

        return {"hidden": hide, "names": hidden, "count": len(hidden)}

    def method_list(self) -> list[MethodInfo]:
        """Return available object methods."""
        return [
            MethodInfo("object.create", "Create a primitive object", [
                ParamInfo("type", "string", False, "CUBE"),
                ParamInfo("name", "string", False),
                ParamInfo("location", "array", False, [0, 0, 0]),
                ParamInfo("rotation", "array", False, [0, 0, 0]),
                ParamInfo("scale", "array", False, [1, 1, 1]),
                ParamInfo("size", "number", False, 2),
                ParamInfo("radius", "number", False, 1),
            ]),
            MethodInfo("object.delete", "Delete objects", [
                ParamInfo("names", "array", False),
                ParamInfo("selected", "boolean", False, False),
            ]),
            MethodInfo("object.list", "List objects in scene", [
                ParamInfo("type", "string", False),
                ParamInfo("selected", "boolean", False, False),
            ]),
            MethodInfo("object.info", "Get object details", [
                ParamInfo("name", "string", True),
            ]),
            MethodInfo("object.transform", "Transform object", [
                ParamInfo("name", "string", True),
                ParamInfo("location", "array", False),
                ParamInfo("rotation", "array", False),
                ParamInfo("scale", "array", False),
                ParamInfo("translate", "array", False),
                ParamInfo("rotate", "array", False),
            ]),
            MethodInfo("object.select", "Select objects", [
                ParamInfo("names", "array", True),
                ParamInfo("add", "boolean", False, False),
                ParamInfo("deselect_all", "boolean", False, False),
            ]),
            MethodInfo("object.duplicate", "Duplicate objects", [
                ParamInfo("names", "array", True),
                ParamInfo("linked", "boolean", False, False),
            ]),
            MethodInfo("object.rename", "Rename object", [
                ParamInfo("name", "string", True),
                ParamInfo("new_name", "string", True),
            ]),
            MethodInfo("object.parent", "Set parent-child relationship", [
                ParamInfo("child", "string", True),
                ParamInfo("parent", "string", False),
                ParamInfo("clear", "boolean", False, False),
            ]),
            MethodInfo("object.hide", "Hide or show objects", [
                ParamInfo("names", "array", True),
                ParamInfo("hide", "boolean", False, True),
            ]),
        ]
