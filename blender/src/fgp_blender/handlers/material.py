"""
Material Handler for FGP Blender Daemon.

Handles materials and shader nodes.

CHANGELOG:
01/17/2026 - Initial implementation (Claude)
"""

from __future__ import annotations

from typing import Any

from fgp_daemon import MethodInfo, ParamInfo

from ..session import SessionManager


class MaterialHandler:
    """Handler for material and shader operations."""

    def __init__(self, bpy: Any, sessions: SessionManager) -> None:
        """Initialize with Blender module and session manager."""
        self.bpy = bpy
        self.sessions = sessions

    def dispatch(self, category: str, action: str, params: dict[str, Any]) -> Any:
        """Dispatch material/shader methods."""
        # Handle material.* and shader.*
        if category == "shader":
            return self._dispatch_shader(action, params)
        else:
            return self._dispatch_material(action, params)

    def _dispatch_material(self, action: str, params: dict[str, Any]) -> Any:
        """Handle material methods."""
        handlers = {
            "create": self._create,
            "delete": self._delete,
            "assign": self._assign,
            "list": self._list,
            "info": self._info,
            "set_color": self._set_color,
            "set_property": self._set_property,
            "duplicate": self._duplicate,
            # Procedural materials
            "procedural.create": self._procedural_create,
            "procedural.noise": self._procedural_noise,
            "procedural.gradient": self._procedural_gradient,
            "procedural.checker": self._procedural_checker,
            "procedural.brick": self._procedural_brick,
            "procedural.wave": self._procedural_wave,
            "procedural.voronoi": self._procedural_voronoi,
            # Batch operations
            "batch.create": self._batch_create,
            "batch.assign": self._batch_assign,
        }

        handler = handlers.get(action)
        if handler is None:
            raise ValueError(f"Unknown material action: {action}")

        return handler(params)

    def _dispatch_shader(self, action: str, params: dict[str, Any]) -> Any:
        """Handle shader node methods."""
        # Parse nested action like shader.node.add
        parts = action.split(".")
        if len(parts) > 1:
            sub = parts[0]
            sub_action = parts[1]
        else:
            sub = action
            sub_action = None

        if sub == "node":
            return self._shader_node(sub_action, params)
        else:
            raise ValueError(f"Unknown shader action: {action}")

    def _create(self, params: dict[str, Any]) -> dict[str, Any]:
        """Create a new material."""
        if not self.bpy:
            return {"created": True, "name": "Material", "mock": True}

        bpy = self.bpy
        name = params.get("name", "Material")
        use_nodes = params.get("use_nodes", True)

        material = bpy.data.materials.new(name=name)
        material.use_nodes = use_nodes

        # Set default color if specified
        if "color" in params and use_nodes:
            color = params["color"]
            if len(color) == 3:
                color = color + [1.0]  # Add alpha

            principled = material.node_tree.nodes.get("Principled BSDF")
            if principled:
                principled.inputs["Base Color"].default_value = color

        # Set metallic/roughness
        if use_nodes:
            principled = material.node_tree.nodes.get("Principled BSDF")
            if principled:
                if "metallic" in params:
                    principled.inputs["Metallic"].default_value = params["metallic"]
                if "roughness" in params:
                    principled.inputs["Roughness"].default_value = params["roughness"]

        self.sessions.update_modified()

        return {
            "created": True,
            "name": material.name,
            "use_nodes": material.use_nodes,
        }

    def _delete(self, params: dict[str, Any]) -> dict[str, Any]:
        """Delete a material."""
        if not self.bpy:
            return {"deleted": True, "mock": True}

        name = params.get("name")
        if not name:
            raise ValueError("name is required")

        material = self.bpy.data.materials.get(name)
        if not material:
            raise ValueError(f"Material not found: {name}")

        self.bpy.data.materials.remove(material)
        self.sessions.update_modified()

        return {"deleted": True, "name": name}

    def _assign(self, params: dict[str, Any]) -> dict[str, Any]:
        """Assign material to object."""
        if not self.bpy:
            return {"assigned": True, "mock": True}

        object_name = params.get("object")
        material_name = params.get("material")
        slot = params.get("slot", 0)

        if not object_name or not material_name:
            raise ValueError("object and material are required")

        obj = self.bpy.data.objects.get(object_name)
        if not obj:
            raise ValueError(f"Object not found: {object_name}")

        material = self.bpy.data.materials.get(material_name)
        if not material:
            raise ValueError(f"Material not found: {material_name}")

        # Ensure object has material slots
        if not obj.data.materials:
            obj.data.materials.append(material)
        elif slot < len(obj.data.materials):
            obj.data.materials[slot] = material
        else:
            obj.data.materials.append(material)

        self.sessions.update_modified()

        return {
            "assigned": True,
            "object": object_name,
            "material": material.name,
            "slot": slot,
        }

    def _list(self, params: dict[str, Any]) -> dict[str, Any]:
        """List all materials."""
        if not self.bpy:
            return {"materials": [], "mock": True}

        materials = [
            {
                "name": m.name,
                "use_nodes": m.use_nodes,
                "users": m.users,
            }
            for m in self.bpy.data.materials
        ]

        return {"materials": materials, "count": len(materials)}

    def _info(self, params: dict[str, Any]) -> dict[str, Any]:
        """Get material information."""
        if not self.bpy:
            return {"name": "Material", "mock": True}

        name = params.get("name")
        if not name:
            raise ValueError("name is required")

        material = self.bpy.data.materials.get(name)
        if not material:
            raise ValueError(f"Material not found: {name}")

        info = {
            "name": material.name,
            "use_nodes": material.use_nodes,
            "users": material.users,
        }

        if material.use_nodes:
            principled = material.node_tree.nodes.get("Principled BSDF")
            if principled:
                info["base_color"] = list(principled.inputs["Base Color"].default_value)
                info["metallic"] = principled.inputs["Metallic"].default_value
                info["roughness"] = principled.inputs["Roughness"].default_value

            info["nodes"] = [
                {"name": n.name, "type": n.type}
                for n in material.node_tree.nodes
            ]

        return info

    def _set_color(self, params: dict[str, Any]) -> dict[str, Any]:
        """Set material base color."""
        if not self.bpy:
            return {"set": True, "mock": True}

        name = params.get("name")
        color = params.get("color")

        if not name or not color:
            raise ValueError("name and color are required")

        material = self.bpy.data.materials.get(name)
        if not material:
            raise ValueError(f"Material not found: {name}")

        if len(color) == 3:
            color = color + [1.0]

        if material.use_nodes:
            principled = material.node_tree.nodes.get("Principled BSDF")
            if principled:
                principled.inputs["Base Color"].default_value = color

        self.sessions.update_modified()

        return {"set": True, "name": name, "color": color}

    def _set_property(self, params: dict[str, Any]) -> dict[str, Any]:
        """Set material property (metallic, roughness, etc.)."""
        if not self.bpy:
            return {"set": True, "mock": True}

        name = params.get("name")
        prop = params.get("property")
        value = params.get("value")

        if not name or not prop or value is None:
            raise ValueError("name, property, and value are required")

        material = self.bpy.data.materials.get(name)
        if not material:
            raise ValueError(f"Material not found: {name}")

        if material.use_nodes:
            principled = material.node_tree.nodes.get("Principled BSDF")
            if principled:
                # Map property names to Principled BSDF inputs
                prop_map = {
                    "metallic": "Metallic",
                    "roughness": "Roughness",
                    "specular": "Specular IOR Level",
                    "ior": "IOR",
                    "transmission": "Transmission Weight",
                    "emission": "Emission Strength",
                    "alpha": "Alpha",
                    "subsurface": "Subsurface Weight",
                }

                input_name = prop_map.get(prop.lower(), prop)
                if input_name in principled.inputs:
                    principled.inputs[input_name].default_value = value

        self.sessions.update_modified()

        return {"set": True, "name": name, "property": prop, "value": value}

    def _duplicate(self, params: dict[str, Any]) -> dict[str, Any]:
        """Duplicate a material."""
        if not self.bpy:
            return {"duplicated": True, "mock": True}

        name = params.get("name")
        new_name = params.get("new_name")

        if not name:
            raise ValueError("name is required")

        material = self.bpy.data.materials.get(name)
        if not material:
            raise ValueError(f"Material not found: {name}")

        new_material = material.copy()
        if new_name:
            new_material.name = new_name

        self.sessions.update_modified()

        return {
            "duplicated": True,
            "original": name,
            "new_name": new_material.name,
        }

    def _shader_node(self, action: str, params: dict[str, Any]) -> dict[str, Any]:
        """Handle shader node operations."""
        if not self.bpy:
            return {"success": True, "mock": True}

        bpy = self.bpy
        material_name = params.get("material")
        if not material_name:
            raise ValueError("material is required")

        material = bpy.data.materials.get(material_name)
        if not material or not material.use_nodes:
            raise ValueError(f"Material with nodes not found: {material_name}")

        node_tree = material.node_tree

        if action == "add":
            node_type = params.get("type")
            name = params.get("name")
            location = params.get("location", [0, 0])

            if not node_type:
                raise ValueError("type is required")

            node = node_tree.nodes.new(type=node_type)
            if name:
                node.name = name
            node.location = location

            self.sessions.update_modified()

            return {
                "added": True,
                "material": material_name,
                "node": node.name,
                "type": node_type,
            }

        elif action == "remove":
            node_name = params.get("name")
            if not node_name:
                raise ValueError("name is required")

            node = node_tree.nodes.get(node_name)
            if not node:
                raise ValueError(f"Node not found: {node_name}")

            node_tree.nodes.remove(node)
            self.sessions.update_modified()

            return {"removed": True, "material": material_name, "node": node_name}

        elif action == "connect":
            from_node = params.get("from_node")
            from_socket = params.get("from_socket")
            to_node = params.get("to_node")
            to_socket = params.get("to_socket")

            if not all([from_node, from_socket, to_node, to_socket]):
                raise ValueError("from_node, from_socket, to_node, to_socket are required")

            source = node_tree.nodes.get(from_node)
            target = node_tree.nodes.get(to_node)

            if not source:
                raise ValueError(f"Source node not found: {from_node}")
            if not target:
                raise ValueError(f"Target node not found: {to_node}")

            # Find sockets by name or index
            out_socket = None
            if isinstance(from_socket, int):
                out_socket = source.outputs[from_socket]
            else:
                out_socket = source.outputs.get(from_socket)

            in_socket = None
            if isinstance(to_socket, int):
                in_socket = target.inputs[to_socket]
            else:
                in_socket = target.inputs.get(to_socket)

            if not out_socket:
                raise ValueError(f"Output socket not found: {from_socket}")
            if not in_socket:
                raise ValueError(f"Input socket not found: {to_socket}")

            node_tree.links.new(out_socket, in_socket)
            self.sessions.update_modified()

            return {
                "connected": True,
                "material": material_name,
                "from": f"{from_node}.{from_socket}",
                "to": f"{to_node}.{to_socket}",
            }

        elif action == "disconnect":
            node_name = params.get("node")
            socket_name = params.get("socket")
            input_socket = params.get("input", True)

            if not node_name or not socket_name:
                raise ValueError("node and socket are required")

            node = node_tree.nodes.get(node_name)
            if not node:
                raise ValueError(f"Node not found: {node_name}")

            sockets = node.inputs if input_socket else node.outputs
            socket = sockets.get(socket_name)
            if not socket:
                raise ValueError(f"Socket not found: {socket_name}")

            for link in list(socket.links):
                node_tree.links.remove(link)

            self.sessions.update_modified()

            return {"disconnected": True, "material": material_name, "node": node_name}

        elif action == "list":
            nodes = [
                {
                    "name": n.name,
                    "type": n.type,
                    "location": list(n.location),
                    "inputs": [s.name for s in n.inputs],
                    "outputs": [s.name for s in n.outputs],
                }
                for n in node_tree.nodes
            ]

            links = [
                {
                    "from_node": l.from_node.name,
                    "from_socket": l.from_socket.name,
                    "to_node": l.to_node.name,
                    "to_socket": l.to_socket.name,
                }
                for l in node_tree.links
            ]

            return {
                "material": material_name,
                "nodes": nodes,
                "links": links,
            }

        elif action == "set":
            node_name = params.get("node")
            input_name = params.get("input")
            value = params.get("value")

            if not node_name or not input_name or value is None:
                raise ValueError("node, input, and value are required")

            node = node_tree.nodes.get(node_name)
            if not node:
                raise ValueError(f"Node not found: {node_name}")

            if input_name in node.inputs:
                node.inputs[input_name].default_value = value
            else:
                raise ValueError(f"Input not found: {input_name}")

            self.sessions.update_modified()

            return {
                "set": True,
                "material": material_name,
                "node": node_name,
                "input": input_name,
            }

        else:
            raise ValueError(f"Unknown shader node action: {action}")

    # -------------------------------------------------------------------------
    # Procedural Materials
    # -------------------------------------------------------------------------

    def _procedural_create(self, params: dict[str, Any]) -> dict[str, Any]:
        """Create a procedural material from preset."""
        if not self.bpy:
            return {"created": True, "mock": True}

        bpy = self.bpy
        name = params.get("name", "Procedural")
        preset = params.get("preset", "noise")  # noise, gradient, checker, brick, wave, voronoi, marble, wood
        color1 = params.get("color1", [0.8, 0.8, 0.8, 1.0])
        color2 = params.get("color2", [0.2, 0.2, 0.2, 1.0])
        scale = params.get("scale", 5.0)

        material = bpy.data.materials.new(name=name)
        material.use_nodes = True
        nodes = material.node_tree.nodes
        links = material.node_tree.links

        # Clear default nodes except output
        output = nodes.get("Material Output")
        for node in list(nodes):
            if node != output:
                nodes.remove(node)

        # Create Principled BSDF
        principled = nodes.new("ShaderNodeBsdfPrincipled")
        principled.location = (0, 0)
        links.new(principled.outputs["BSDF"], output.inputs["Surface"])

        # Create texture coordinate and mapping
        tex_coord = nodes.new("ShaderNodeTexCoord")
        tex_coord.location = (-800, 0)
        mapping = nodes.new("ShaderNodeMapping")
        mapping.location = (-600, 0)
        mapping.inputs["Scale"].default_value = (scale, scale, scale)
        links.new(tex_coord.outputs["Generated"], mapping.inputs["Vector"])

        # Create color ramp
        ramp = nodes.new("ShaderNodeValToRGB")
        ramp.location = (-200, 0)
        ramp.color_ramp.elements[0].color = color1 if len(color1) == 4 else color1 + [1.0]
        ramp.color_ramp.elements[1].color = color2 if len(color2) == 4 else color2 + [1.0]
        links.new(ramp.outputs["Color"], principled.inputs["Base Color"])

        # Create texture based on preset
        if preset == "noise" or preset == "marble":
            tex = nodes.new("ShaderNodeTexNoise")
            tex.location = (-400, 0)
            tex.inputs["Scale"].default_value = scale
            if preset == "marble":
                tex.inputs["Distortion"].default_value = 5.0
            links.new(mapping.outputs["Vector"], tex.inputs["Vector"])
            links.new(tex.outputs["Fac"], ramp.inputs["Fac"])

        elif preset == "voronoi":
            tex = nodes.new("ShaderNodeTexVoronoi")
            tex.location = (-400, 0)
            tex.inputs["Scale"].default_value = scale
            links.new(mapping.outputs["Vector"], tex.inputs["Vector"])
            links.new(tex.outputs["Distance"], ramp.inputs["Fac"])

        elif preset == "checker":
            tex = nodes.new("ShaderNodeTexChecker")
            tex.location = (-400, 0)
            tex.inputs["Scale"].default_value = scale
            links.new(mapping.outputs["Vector"], tex.inputs["Vector"])
            links.new(tex.outputs["Fac"], ramp.inputs["Fac"])

        elif preset == "brick":
            tex = nodes.new("ShaderNodeTexBrick")
            tex.location = (-400, 0)
            tex.inputs["Scale"].default_value = scale
            links.new(mapping.outputs["Vector"], tex.inputs["Vector"])
            links.new(tex.outputs["Fac"], ramp.inputs["Fac"])

        elif preset == "wave":
            tex = nodes.new("ShaderNodeTexWave")
            tex.location = (-400, 0)
            tex.inputs["Scale"].default_value = scale
            links.new(mapping.outputs["Vector"], tex.inputs["Vector"])
            links.new(tex.outputs["Fac"], ramp.inputs["Fac"])

        elif preset == "gradient":
            tex = nodes.new("ShaderNodeTexGradient")
            tex.location = (-400, 0)
            links.new(mapping.outputs["Vector"], tex.inputs["Vector"])
            links.new(tex.outputs["Fac"], ramp.inputs["Fac"])

        elif preset == "wood":
            tex = nodes.new("ShaderNodeTexWave")
            tex.location = (-400, 0)
            tex.wave_type = 'RINGS'
            tex.inputs["Scale"].default_value = scale
            tex.inputs["Distortion"].default_value = 10.0
            links.new(mapping.outputs["Vector"], tex.inputs["Vector"])
            links.new(tex.outputs["Fac"], ramp.inputs["Fac"])

        self.sessions.update_modified()

        return {
            "created": True,
            "name": material.name,
            "preset": preset,
        }

    def _procedural_noise(self, params: dict[str, Any]) -> dict[str, Any]:
        """Add noise texture to material."""
        return self._add_procedural_texture(params, "noise")

    def _procedural_gradient(self, params: dict[str, Any]) -> dict[str, Any]:
        """Add gradient texture to material."""
        return self._add_procedural_texture(params, "gradient")

    def _procedural_checker(self, params: dict[str, Any]) -> dict[str, Any]:
        """Add checker texture to material."""
        return self._add_procedural_texture(params, "checker")

    def _procedural_brick(self, params: dict[str, Any]) -> dict[str, Any]:
        """Add brick texture to material."""
        return self._add_procedural_texture(params, "brick")

    def _procedural_wave(self, params: dict[str, Any]) -> dict[str, Any]:
        """Add wave texture to material."""
        return self._add_procedural_texture(params, "wave")

    def _procedural_voronoi(self, params: dict[str, Any]) -> dict[str, Any]:
        """Add voronoi texture to material."""
        return self._add_procedural_texture(params, "voronoi")

    def _add_procedural_texture(self, params: dict[str, Any], tex_type: str) -> dict[str, Any]:
        """Add procedural texture to existing material."""
        if not self.bpy:
            return {"added": True, "mock": True}

        bpy = self.bpy
        material_name = params.get("material")
        target_input = params.get("input", "Base Color")  # Which input to connect to
        scale = params.get("scale", 5.0)

        material = bpy.data.materials.get(material_name)
        if not material or not material.use_nodes:
            raise ValueError(f"Material with nodes not found: {material_name}")

        nodes = material.node_tree.nodes
        links = material.node_tree.links

        # Find principled BSDF
        principled = nodes.get("Principled BSDF")
        if not principled:
            raise ValueError("No Principled BSDF node found")

        # Create texture coordinate if not exists
        tex_coord = None
        for node in nodes:
            if node.type == 'TEX_COORD':
                tex_coord = node
                break
        if not tex_coord:
            tex_coord = nodes.new("ShaderNodeTexCoord")
            tex_coord.location = (-800, 0)

        # Create texture node
        tex_types = {
            "noise": "ShaderNodeTexNoise",
            "voronoi": "ShaderNodeTexVoronoi",
            "checker": "ShaderNodeTexChecker",
            "brick": "ShaderNodeTexBrick",
            "wave": "ShaderNodeTexWave",
            "gradient": "ShaderNodeTexGradient",
        }

        tex = nodes.new(tex_types[tex_type])
        tex.location = (-400, 200)
        if tex_type != "gradient" and "Scale" in tex.inputs:
            tex.inputs["Scale"].default_value = scale

        links.new(tex_coord.outputs["Generated"], tex.inputs["Vector"])

        # Connect to target input
        output_name = "Fac" if tex_type != "voronoi" else "Distance"
        if target_input in principled.inputs:
            links.new(tex.outputs[output_name], principled.inputs[target_input])

        self.sessions.update_modified()

        return {
            "added": True,
            "material": material_name,
            "texture": tex_type,
            "input": target_input,
        }

    # -------------------------------------------------------------------------
    # Batch Operations
    # -------------------------------------------------------------------------

    def _batch_create(self, params: dict[str, Any]) -> dict[str, Any]:
        """Create multiple materials at once."""
        if not self.bpy:
            return {"created": 0, "mock": True}

        materials = params.get("materials", [])
        # Each item: {name, color, metallic, roughness}

        created = []
        for mat_def in materials:
            result = self._create(mat_def)
            created.append(result["name"])

        return {
            "created": len(created),
            "materials": created,
        }

    def _batch_assign(self, params: dict[str, Any]) -> dict[str, Any]:
        """Assign materials to multiple objects."""
        if not self.bpy:
            return {"assigned": 0, "mock": True}

        assignments = params.get("assignments", [])
        # Each item: {object, material, slot}

        assigned = []
        for assignment in assignments:
            try:
                self._assign(assignment)
                assigned.append(assignment.get("object"))
            except Exception as e:
                pass  # Skip failures in batch

        return {
            "assigned": len(assigned),
            "objects": assigned,
        }

    def method_list(self) -> list[MethodInfo]:
        """Return available material/shader methods."""
        return [
            MethodInfo("material.create", "Create a new material", [
                ParamInfo("name", "string", False, "Material"),
                ParamInfo("use_nodes", "boolean", False, True),
                ParamInfo("color", "array", False),
                ParamInfo("metallic", "number", False),
                ParamInfo("roughness", "number", False),
            ]),
            MethodInfo("material.delete", "Delete a material", [
                ParamInfo("name", "string", True),
            ]),
            MethodInfo("material.assign", "Assign material to object", [
                ParamInfo("object", "string", True),
                ParamInfo("material", "string", True),
                ParamInfo("slot", "integer", False, 0),
            ]),
            MethodInfo("material.list", "List all materials", []),
            MethodInfo("material.info", "Get material information", [
                ParamInfo("name", "string", True),
            ]),
            MethodInfo("material.set_color", "Set base color", [
                ParamInfo("name", "string", True),
                ParamInfo("color", "array", True),
            ]),
            MethodInfo("material.set_property", "Set material property", [
                ParamInfo("name", "string", True),
                ParamInfo("property", "string", True),
                ParamInfo("value", "number", True),
            ]),
            MethodInfo("material.duplicate", "Duplicate a material", [
                ParamInfo("name", "string", True),
                ParamInfo("new_name", "string", False),
            ]),
            MethodInfo("shader.node.add", "Add shader node", [
                ParamInfo("material", "string", True),
                ParamInfo("type", "string", True),
                ParamInfo("name", "string", False),
                ParamInfo("location", "array", False, [0, 0]),
            ]),
            MethodInfo("shader.node.remove", "Remove shader node", [
                ParamInfo("material", "string", True),
                ParamInfo("name", "string", True),
            ]),
            MethodInfo("shader.node.connect", "Connect shader nodes", [
                ParamInfo("material", "string", True),
                ParamInfo("from_node", "string", True),
                ParamInfo("from_socket", "string", True),
                ParamInfo("to_node", "string", True),
                ParamInfo("to_socket", "string", True),
            ]),
            MethodInfo("shader.node.list", "List shader nodes", [
                ParamInfo("material", "string", True),
            ]),
            MethodInfo("shader.node.set", "Set node input value", [
                ParamInfo("material", "string", True),
                ParamInfo("node", "string", True),
                ParamInfo("input", "string", True),
                ParamInfo("value", "any", True),
            ]),
            # Procedural materials
            MethodInfo("material.procedural.create", "Create procedural material from preset", [
                ParamInfo("name", "string", False, "Procedural"),
                ParamInfo("preset", "string", False, "noise", description="noise, gradient, checker, brick, wave, voronoi, marble, wood"),
                ParamInfo("color1", "array", False, description="Primary color [r,g,b,a]"),
                ParamInfo("color2", "array", False, description="Secondary color [r,g,b,a]"),
                ParamInfo("scale", "number", False, 5.0),
            ]),
            MethodInfo("material.procedural.noise", "Add noise texture to material", [
                ParamInfo("material", "string", True),
                ParamInfo("scale", "number", False, 5.0),
                ParamInfo("input", "string", False, "Base Color", description="Target BSDF input"),
            ]),
            MethodInfo("material.procedural.gradient", "Add gradient texture to material", [
                ParamInfo("material", "string", True),
                ParamInfo("input", "string", False, "Base Color"),
            ]),
            MethodInfo("material.procedural.checker", "Add checker texture to material", [
                ParamInfo("material", "string", True),
                ParamInfo("scale", "number", False, 5.0),
                ParamInfo("input", "string", False, "Base Color"),
            ]),
            MethodInfo("material.procedural.brick", "Add brick texture to material", [
                ParamInfo("material", "string", True),
                ParamInfo("scale", "number", False, 5.0),
                ParamInfo("input", "string", False, "Base Color"),
            ]),
            MethodInfo("material.procedural.wave", "Add wave texture to material", [
                ParamInfo("material", "string", True),
                ParamInfo("scale", "number", False, 5.0),
                ParamInfo("input", "string", False, "Base Color"),
            ]),
            MethodInfo("material.procedural.voronoi", "Add voronoi texture to material", [
                ParamInfo("material", "string", True),
                ParamInfo("scale", "number", False, 5.0),
                ParamInfo("input", "string", False, "Base Color"),
            ]),
            # Batch operations
            MethodInfo("material.batch.create", "Create multiple materials at once", [
                ParamInfo("materials", "array", True, description="Array of {name, color, metallic, roughness}"),
            ]),
            MethodInfo("material.batch.assign", "Assign materials to multiple objects", [
                ParamInfo("assignments", "array", True, description="Array of {object, material, slot}"),
            ]),
        ]
