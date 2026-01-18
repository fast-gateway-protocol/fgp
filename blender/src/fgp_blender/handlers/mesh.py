"""
Mesh Handler for FGP Blender Daemon.

Handles mesh operations: modifiers, boolean, geometry nodes.

CHANGELOG:
01/17/2026 - Initial implementation (Claude)
"""

from __future__ import annotations

from typing import Any

from fgp_daemon import MethodInfo, ParamInfo

from ..session import SessionManager


class MeshHandler:
    """Handler for mesh operations."""

    def __init__(self, bpy: Any, sessions: SessionManager) -> None:
        """Initialize with Blender module and session manager."""
        self.bpy = bpy
        self.sessions = sessions

    def dispatch(self, category: str, action: str, params: dict[str, Any]) -> Any:
        """Dispatch mesh methods."""
        # Handle nested actions like mesh.modifier.add
        parts = action.split(".")
        if len(parts) > 1:
            sub_category = parts[0]
            sub_action = parts[1]
        else:
            sub_category = action
            sub_action = None

        handlers = {
            "modifier": self._modifier,
            "boolean": self._boolean,
            "subdivide": self._subdivide,
            "smooth": self._smooth,
            "decimate": self._decimate,
            "geometry_nodes": self._geometry_nodes,
            "apply_modifiers": self._apply_modifiers,
            "edit": self._edit,
        }

        handler = handlers.get(sub_category)
        if handler is None:
            raise ValueError(f"Unknown mesh action: {action}")

        return handler(sub_action, params)

    def _modifier(self, action: str, params: dict[str, Any]) -> dict[str, Any]:
        """Handle modifier operations."""
        if not self.bpy:
            return {"success": True, "mock": True}

        bpy = self.bpy
        object_name = params.get("object")
        if not object_name:
            raise ValueError("object is required")

        obj = bpy.data.objects.get(object_name)
        if not obj or obj.type != "MESH":
            raise ValueError(f"Mesh object not found: {object_name}")

        if action == "add":
            mod_type = params.get("type", "SUBSURF").upper()
            mod_name = params.get("name", mod_type)

            modifier = obj.modifiers.new(name=mod_name, type=mod_type)

            # Apply common modifier settings
            settings = params.get("settings", {})
            for key, value in settings.items():
                if hasattr(modifier, key):
                    setattr(modifier, key, value)

            self.sessions.update_modified()

            return {
                "added": True,
                "object": object_name,
                "modifier": modifier.name,
                "type": mod_type,
            }

        elif action == "remove":
            mod_name = params.get("name")
            if not mod_name:
                raise ValueError("name is required")

            modifier = obj.modifiers.get(mod_name)
            if not modifier:
                raise ValueError(f"Modifier not found: {mod_name}")

            obj.modifiers.remove(modifier)
            self.sessions.update_modified()

            return {"removed": True, "object": object_name, "modifier": mod_name}

        elif action == "apply":
            mod_name = params.get("name")
            if not mod_name:
                raise ValueError("name is required")

            modifier = obj.modifiers.get(mod_name)
            if not modifier:
                raise ValueError(f"Modifier not found: {mod_name}")

            bpy.context.view_layer.objects.active = obj
            bpy.ops.object.modifier_apply(modifier=mod_name)
            self.sessions.update_modified()

            return {"applied": True, "object": object_name, "modifier": mod_name}

        elif action == "list":
            modifiers = [
                {"name": m.name, "type": m.type, "show_viewport": m.show_viewport}
                for m in obj.modifiers
            ]
            return {"object": object_name, "modifiers": modifiers}

        else:
            raise ValueError(f"Unknown modifier action: {action}")

    def _boolean(self, action: str, params: dict[str, Any]) -> dict[str, Any]:
        """Apply boolean operation between meshes."""
        if not self.bpy:
            return {"success": True, "mock": True}

        bpy = self.bpy
        object_name = params.get("object")
        target_name = params.get("target")
        operation = params.get("operation", "DIFFERENCE").upper()

        if not object_name or not target_name:
            raise ValueError("object and target are required")

        obj = bpy.data.objects.get(object_name)
        target = bpy.data.objects.get(target_name)

        if not obj or obj.type != "MESH":
            raise ValueError(f"Mesh object not found: {object_name}")
        if not target or target.type != "MESH":
            raise ValueError(f"Target mesh not found: {target_name}")

        # Add boolean modifier
        modifier = obj.modifiers.new(name="Boolean", type="BOOLEAN")
        modifier.operation = operation
        modifier.object = target

        # Optionally apply immediately
        if params.get("apply", False):
            bpy.context.view_layer.objects.active = obj
            bpy.ops.object.modifier_apply(modifier="Boolean")

        # Optionally hide/delete target
        if params.get("hide_target", False):
            target.hide_set(True)
        if params.get("delete_target", False):
            bpy.data.objects.remove(target, do_unlink=True)

        self.sessions.update_modified()

        return {
            "success": True,
            "object": object_name,
            "target": target_name,
            "operation": operation,
        }

    def _subdivide(self, action: str, params: dict[str, Any]) -> dict[str, Any]:
        """Add subdivision surface modifier."""
        if not self.bpy:
            return {"success": True, "mock": True}

        object_name = params.get("object")
        levels = params.get("levels", 2)
        render_levels = params.get("render_levels", levels)

        if not object_name:
            raise ValueError("object is required")

        obj = self.bpy.data.objects.get(object_name)
        if not obj or obj.type != "MESH":
            raise ValueError(f"Mesh object not found: {object_name}")

        modifier = obj.modifiers.new(name="Subdivision", type="SUBSURF")
        modifier.levels = levels
        modifier.render_levels = render_levels

        if params.get("apply", False):
            self.bpy.context.view_layer.objects.active = obj
            self.bpy.ops.object.modifier_apply(modifier="Subdivision")

        self.sessions.update_modified()

        return {
            "success": True,
            "object": object_name,
            "levels": levels,
            "render_levels": render_levels,
        }

    def _smooth(self, action: str, params: dict[str, Any]) -> dict[str, Any]:
        """Apply smooth shading to mesh."""
        if not self.bpy:
            return {"success": True, "mock": True}

        bpy = self.bpy
        object_name = params.get("object")
        auto_smooth = params.get("auto_smooth", True)
        angle = params.get("angle", 30)  # Degrees

        if not object_name:
            raise ValueError("object is required")

        obj = bpy.data.objects.get(object_name)
        if not obj or obj.type != "MESH":
            raise ValueError(f"Mesh object not found: {object_name}")

        # Set smooth shading
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.shade_smooth()

        # Set auto smooth if available
        if hasattr(obj.data, "use_auto_smooth"):
            import math
            obj.data.use_auto_smooth = auto_smooth
            obj.data.auto_smooth_angle = math.radians(angle)

        self.sessions.update_modified()

        return {"success": True, "object": object_name, "auto_smooth": auto_smooth}

    def _decimate(self, action: str, params: dict[str, Any]) -> dict[str, Any]:
        """Add decimate modifier to reduce polygon count."""
        if not self.bpy:
            return {"success": True, "mock": True}

        object_name = params.get("object")
        ratio = params.get("ratio", 0.5)
        decimate_type = params.get("type", "COLLAPSE").upper()

        if not object_name:
            raise ValueError("object is required")

        obj = self.bpy.data.objects.get(object_name)
        if not obj or obj.type != "MESH":
            raise ValueError(f"Mesh object not found: {object_name}")

        modifier = obj.modifiers.new(name="Decimate", type="DECIMATE")
        modifier.decimate_type = decimate_type
        modifier.ratio = ratio

        if params.get("apply", False):
            self.bpy.context.view_layer.objects.active = obj
            self.bpy.ops.object.modifier_apply(modifier="Decimate")

        self.sessions.update_modified()

        return {
            "success": True,
            "object": object_name,
            "ratio": ratio,
            "type": decimate_type,
        }

    def _geometry_nodes(self, action: str, params: dict[str, Any]) -> dict[str, Any]:
        """Add or modify geometry nodes modifier."""
        if not self.bpy:
            return {"success": True, "mock": True}

        bpy = self.bpy
        object_name = params.get("object")

        if not object_name:
            raise ValueError("object is required")

        obj = bpy.data.objects.get(object_name)
        if not obj or obj.type != "MESH":
            raise ValueError(f"Mesh object not found: {object_name}")

        if action == "add":
            # Create new geometry nodes modifier
            modifier = obj.modifiers.new(name="GeometryNodes", type="NODES")

            # Create new node tree if specified
            if params.get("create_tree", True):
                node_tree = bpy.data.node_groups.new(
                    name=params.get("tree_name", "Geometry Nodes"),
                    type="GeometryNodeTree"
                )
                modifier.node_group = node_tree

                # Add basic input/output nodes
                input_node = node_tree.nodes.new("NodeGroupInput")
                input_node.location = (-200, 0)
                output_node = node_tree.nodes.new("NodeGroupOutput")
                output_node.location = (200, 0)

                # Create geometry socket
                node_tree.interface.new_socket(
                    name="Geometry",
                    in_out="INPUT",
                    socket_type="NodeSocketGeometry"
                )
                node_tree.interface.new_socket(
                    name="Geometry",
                    in_out="OUTPUT",
                    socket_type="NodeSocketGeometry"
                )

                # Link input to output
                node_tree.links.new(
                    input_node.outputs["Geometry"],
                    output_node.inputs["Geometry"]
                )

            self.sessions.update_modified()

            return {
                "added": True,
                "object": object_name,
                "modifier": modifier.name,
                "node_tree": modifier.node_group.name if modifier.node_group else None,
            }

        elif action == "set_tree":
            tree_name = params.get("tree_name")
            modifier_name = params.get("modifier", "GeometryNodes")

            modifier = obj.modifiers.get(modifier_name)
            if not modifier or modifier.type != "NODES":
                raise ValueError(f"Geometry nodes modifier not found: {modifier_name}")

            node_tree = bpy.data.node_groups.get(tree_name)
            if not node_tree:
                raise ValueError(f"Node tree not found: {tree_name}")

            modifier.node_group = node_tree
            self.sessions.update_modified()

            return {"set": True, "object": object_name, "tree_name": tree_name}

        else:
            raise ValueError(f"Unknown geometry_nodes action: {action}")

    def _apply_modifiers(self, action: str, params: dict[str, Any]) -> dict[str, Any]:
        """Apply all modifiers to mesh."""
        if not self.bpy:
            return {"success": True, "mock": True}

        bpy = self.bpy
        object_name = params.get("object")

        if not object_name:
            raise ValueError("object is required")

        obj = bpy.data.objects.get(object_name)
        if not obj or obj.type != "MESH":
            raise ValueError(f"Mesh object not found: {object_name}")

        bpy.context.view_layer.objects.active = obj
        applied = []

        for modifier in list(obj.modifiers):
            try:
                bpy.ops.object.modifier_apply(modifier=modifier.name)
                applied.append(modifier.name)
            except Exception as e:
                pass  # Some modifiers can't be applied

        self.sessions.update_modified()

        return {"applied": True, "object": object_name, "modifiers": applied}

    def _edit(self, action: str, params: dict[str, Any]) -> dict[str, Any]:
        """Perform edit mode operations."""
        if not self.bpy:
            return {"success": True, "mock": True}

        bpy = self.bpy
        object_name = params.get("object")

        if not object_name:
            raise ValueError("object is required")

        obj = bpy.data.objects.get(object_name)
        if not obj or obj.type != "MESH":
            raise ValueError(f"Mesh object not found: {object_name}")

        # Enter edit mode
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.mode_set(mode="EDIT")

        try:
            if action == "extrude":
                amount = params.get("amount", 1.0)
                bpy.ops.mesh.select_all(action="SELECT")
                bpy.ops.mesh.extrude_region_move(
                    TRANSFORM_OT_translate={"value": (0, 0, amount)}
                )

            elif action == "inset":
                thickness = params.get("thickness", 0.1)
                depth = params.get("depth", 0.0)
                bpy.ops.mesh.select_all(action="SELECT")
                bpy.ops.mesh.inset(thickness=thickness, depth=depth)

            elif action == "bevel":
                amount = params.get("amount", 0.1)
                segments = params.get("segments", 1)
                bpy.ops.mesh.select_all(action="SELECT")
                bpy.ops.mesh.bevel(offset=amount, segments=segments)

            elif action == "loop_cut":
                cuts = params.get("cuts", 1)
                bpy.ops.mesh.loopcut_slide(
                    MESH_OT_loopcut={"number_cuts": cuts}
                )

            else:
                raise ValueError(f"Unknown edit action: {action}")

        finally:
            # Return to object mode
            bpy.ops.object.mode_set(mode="OBJECT")

        self.sessions.update_modified()

        return {"success": True, "object": object_name, "action": action}

    def method_list(self) -> list[MethodInfo]:
        """Return available mesh methods."""
        return [
            MethodInfo("mesh.modifier.add", "Add modifier to mesh", [
                ParamInfo("object", "string", True),
                ParamInfo("type", "string", False, "SUBSURF"),
                ParamInfo("name", "string", False),
                ParamInfo("settings", "object", False),
            ]),
            MethodInfo("mesh.modifier.remove", "Remove modifier", [
                ParamInfo("object", "string", True),
                ParamInfo("name", "string", True),
            ]),
            MethodInfo("mesh.modifier.apply", "Apply modifier", [
                ParamInfo("object", "string", True),
                ParamInfo("name", "string", True),
            ]),
            MethodInfo("mesh.modifier.list", "List modifiers on object", [
                ParamInfo("object", "string", True),
            ]),
            MethodInfo("mesh.boolean", "Boolean operation between meshes", [
                ParamInfo("object", "string", True),
                ParamInfo("target", "string", True),
                ParamInfo("operation", "string", False, "DIFFERENCE"),
                ParamInfo("apply", "boolean", False, False),
                ParamInfo("hide_target", "boolean", False, False),
                ParamInfo("delete_target", "boolean", False, False),
            ]),
            MethodInfo("mesh.subdivide", "Add subdivision surface", [
                ParamInfo("object", "string", True),
                ParamInfo("levels", "integer", False, 2),
                ParamInfo("render_levels", "integer", False),
                ParamInfo("apply", "boolean", False, False),
            ]),
            MethodInfo("mesh.smooth", "Apply smooth shading", [
                ParamInfo("object", "string", True),
                ParamInfo("auto_smooth", "boolean", False, True),
                ParamInfo("angle", "number", False, 30),
            ]),
            MethodInfo("mesh.decimate", "Reduce polygon count", [
                ParamInfo("object", "string", True),
                ParamInfo("ratio", "number", False, 0.5),
                ParamInfo("type", "string", False, "COLLAPSE"),
                ParamInfo("apply", "boolean", False, False),
            ]),
            MethodInfo("mesh.geometry_nodes.add", "Add geometry nodes modifier", [
                ParamInfo("object", "string", True),
                ParamInfo("create_tree", "boolean", False, True),
                ParamInfo("tree_name", "string", False),
            ]),
            MethodInfo("mesh.apply_modifiers", "Apply all modifiers", [
                ParamInfo("object", "string", True),
            ]),
            MethodInfo("mesh.edit.extrude", "Extrude mesh", [
                ParamInfo("object", "string", True),
                ParamInfo("amount", "number", False, 1.0),
            ]),
            MethodInfo("mesh.edit.inset", "Inset faces", [
                ParamInfo("object", "string", True),
                ParamInfo("thickness", "number", False, 0.1),
                ParamInfo("depth", "number", False, 0.0),
            ]),
            MethodInfo("mesh.edit.bevel", "Bevel edges", [
                ParamInfo("object", "string", True),
                ParamInfo("amount", "number", False, 0.1),
                ParamInfo("segments", "integer", False, 1),
            ]),
        ]
