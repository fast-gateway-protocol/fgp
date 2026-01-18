"""
Grease Pencil handler for FGP Blender.

Provides methods for 2D drawing, animation, and effects within 3D space.

CHANGELOG:
01/17/2026 - Initial implementation (Claude)
"""

from __future__ import annotations

import logging
from typing import Any, Optional

from fgp_daemon import MethodInfo, ParamInfo

logger = logging.getLogger(__name__)


class GreasePencilHandler:
    """Handler for Grease Pencil operations."""

    def __init__(self, bpy: Any, sessions: Any) -> None:
        self._bpy = bpy
        self._sessions = sessions

    def dispatch(self, category: str, action: str, params: dict[str, Any]) -> Any:
        """Dispatch grease pencil method calls."""
        method_name = f"{category}.{action}" if action else category

        handlers = {
            # Object management
            "gpencil.create": self._create,
            "gpencil.list": self._list,
            "gpencil.delete": self._delete,
            "gpencil.info": self._info,

            # Layers
            "gpencil.layer.add": self._layer_add,
            "gpencil.layer.remove": self._layer_remove,
            "gpencil.layer.list": self._layer_list,
            "gpencil.layer.settings": self._layer_settings,

            # Drawing
            "gpencil.stroke.add": self._stroke_add,
            "gpencil.stroke.delete": self._stroke_delete,
            "gpencil.fill": self._fill,

            # Materials
            "gpencil.material.add": self._material_add,
            "gpencil.material.assign": self._material_assign,

            # Modifiers
            "gpencil.modifier.add": self._modifier_add,
            "gpencil.modifier.remove": self._modifier_remove,
            "gpencil.modifier.list": self._modifier_list,

            # Effects (visual effects)
            "gpencil.effect.add": self._effect_add,
            "gpencil.effect.remove": self._effect_remove,
            "gpencil.effect.list": self._effect_list,

            # Conversion
            "gpencil.convert_to_mesh": self._convert_to_mesh,
            "gpencil.convert_from_curve": self._convert_from_curve,
        }

        handler = handlers.get(method_name)
        if handler is None:
            raise ValueError(f"Unknown grease pencil method: {method_name}")

        return handler(params)

    # -------------------------------------------------------------------------
    # Object Management
    # -------------------------------------------------------------------------

    def _create(self, params: dict[str, Any]) -> dict:
        """Create a new Grease Pencil object."""
        if not self._bpy:
            return {"created": True, "mock": True}

        bpy = self._bpy
        name = params.get("name", "GPencil")
        location = params.get("location", [0, 0, 0])

        # Create grease pencil data
        gpd = bpy.data.grease_pencils.new(name)

        # Create object
        obj = bpy.data.objects.new(name, gpd)
        obj.location = location

        # Link to scene
        bpy.context.collection.objects.link(obj)

        # Add default layer
        layer = gpd.layers.new("Layer", set_active=True)

        # Add default material
        mat = bpy.data.materials.new(name=f"{name}_Material")
        bpy.data.materials.create_gpencil_data(mat)
        obj.data.materials.append(mat)

        return {
            "created": True,
            "name": obj.name,
            "layer": layer.info,
        }

    def _list(self, params: dict[str, Any]) -> dict:
        """List all Grease Pencil objects."""
        if not self._bpy:
            return {"objects": [], "mock": True}

        bpy = self._bpy
        gp_objects = [
            {
                "name": obj.name,
                "layers": len(obj.data.layers) if obj.data else 0,
                "location": list(obj.location),
            }
            for obj in bpy.data.objects
            if obj.type == "GPENCIL"
        ]

        return {"objects": gp_objects}

    def _delete(self, params: dict[str, Any]) -> dict:
        """Delete a Grease Pencil object."""
        if not self._bpy:
            return {"deleted": True, "mock": True}

        bpy = self._bpy
        name = params.get("name")

        obj = bpy.data.objects.get(name)
        if not obj or obj.type != "GPENCIL":
            raise ValueError(f"Grease Pencil object not found: {name}")

        bpy.data.objects.remove(obj, do_unlink=True)
        return {"deleted": True, "name": name}

    def _info(self, params: dict[str, Any]) -> dict:
        """Get Grease Pencil object info."""
        if not self._bpy:
            return {"mock": True}

        bpy = self._bpy
        name = params.get("name")

        obj = bpy.data.objects.get(name)
        if not obj or obj.type != "GPENCIL":
            raise ValueError(f"Grease Pencil object not found: {name}")

        gpd = obj.data

        return {
            "name": obj.name,
            "location": list(obj.location),
            "layers": [
                {
                    "name": layer.info,
                    "frames": len(layer.frames),
                    "opacity": layer.opacity,
                    "visible": not layer.hide,
                }
                for layer in gpd.layers
            ],
            "materials": [mat.name for mat in obj.data.materials if mat],
            "modifiers": [mod.name for mod in obj.grease_pencil_modifiers],
            "effects": [fx.name for fx in obj.shader_effects],
        }

    # -------------------------------------------------------------------------
    # Layers
    # -------------------------------------------------------------------------

    def _layer_add(self, params: dict[str, Any]) -> dict:
        """Add a layer to Grease Pencil object."""
        if not self._bpy:
            return {"added": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")
        layer_name = params.get("name", "Layer")

        obj = bpy.data.objects.get(obj_name)
        if not obj or obj.type != "GPENCIL":
            raise ValueError(f"Grease Pencil object not found: {obj_name}")

        layer = obj.data.layers.new(layer_name, set_active=True)

        if params.get("opacity") is not None:
            layer.opacity = params["opacity"]
        if params.get("color"):
            layer.channel_color = params["color"]

        return {
            "added": True,
            "object": obj_name,
            "layer": layer.info,
        }

    def _layer_remove(self, params: dict[str, Any]) -> dict:
        """Remove a layer from Grease Pencil object."""
        if not self._bpy:
            return {"removed": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")
        layer_name = params.get("name")

        obj = bpy.data.objects.get(obj_name)
        if not obj or obj.type != "GPENCIL":
            raise ValueError(f"Grease Pencil object not found: {obj_name}")

        layer = obj.data.layers.get(layer_name)
        if not layer:
            raise ValueError(f"Layer not found: {layer_name}")

        obj.data.layers.remove(layer)
        return {"removed": True, "object": obj_name, "layer": layer_name}

    def _layer_list(self, params: dict[str, Any]) -> dict:
        """List layers in Grease Pencil object."""
        if not self._bpy:
            return {"layers": [], "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")

        obj = bpy.data.objects.get(obj_name)
        if not obj or obj.type != "GPENCIL":
            raise ValueError(f"Grease Pencil object not found: {obj_name}")

        layers = [
            {
                "name": layer.info,
                "frames": len(layer.frames),
                "opacity": layer.opacity,
                "visible": not layer.hide,
                "locked": layer.lock,
                "active": layer == obj.data.layers.active,
            }
            for layer in obj.data.layers
        ]

        return {"object": obj_name, "layers": layers}

    def _layer_settings(self, params: dict[str, Any]) -> dict:
        """Update layer settings."""
        if not self._bpy:
            return {"updated": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")
        layer_name = params.get("name")

        obj = bpy.data.objects.get(obj_name)
        if not obj or obj.type != "GPENCIL":
            raise ValueError(f"Grease Pencil object not found: {obj_name}")

        layer = obj.data.layers.get(layer_name)
        if not layer:
            raise ValueError(f"Layer not found: {layer_name}")

        if "opacity" in params:
            layer.opacity = params["opacity"]
        if "visible" in params:
            layer.hide = not params["visible"]
        if "locked" in params:
            layer.lock = params["locked"]
        if "blend_mode" in params:
            layer.blend_mode = params["blend_mode"]
        if "tint_color" in params:
            layer.tint_color = params["tint_color"]
        if "tint_factor" in params:
            layer.tint_factor = params["tint_factor"]

        return {"updated": True, "object": obj_name, "layer": layer_name}

    # -------------------------------------------------------------------------
    # Drawing
    # -------------------------------------------------------------------------

    def _stroke_add(self, params: dict[str, Any]) -> dict:
        """Add a stroke programmatically."""
        if not self._bpy:
            return {"added": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")
        layer_name = params.get("layer")
        points = params.get("points", [])  # [[x,y,z], [x,y,z], ...]
        pressure = params.get("pressure", 1.0)
        strength = params.get("strength", 1.0)
        frame = params.get("frame", bpy.context.scene.frame_current)

        obj = bpy.data.objects.get(obj_name)
        if not obj or obj.type != "GPENCIL":
            raise ValueError(f"Grease Pencil object not found: {obj_name}")

        gpd = obj.data
        layer = gpd.layers.get(layer_name) if layer_name else gpd.layers.active
        if not layer:
            raise ValueError("No layer found")

        # Get or create frame
        gp_frame = layer.frames.get(frame)
        if not gp_frame:
            gp_frame = layer.frames.new(frame)

        # Create stroke
        stroke = gp_frame.strokes.new()
        stroke.display_mode = '3DSPACE'
        stroke.line_width = params.get("line_width", 100)

        # Add points
        stroke.points.add(len(points))
        for i, pt in enumerate(points):
            stroke.points[i].co = pt
            stroke.points[i].pressure = pressure
            stroke.points[i].strength = strength

        # Set material
        if params.get("material_index") is not None:
            stroke.material_index = params["material_index"]

        return {
            "added": True,
            "object": obj_name,
            "layer": layer.info,
            "frame": frame,
            "points": len(points),
        }

    def _stroke_delete(self, params: dict[str, Any]) -> dict:
        """Delete strokes from a frame."""
        if not self._bpy:
            return {"deleted": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")
        layer_name = params.get("layer")
        frame = params.get("frame", bpy.context.scene.frame_current)
        stroke_index = params.get("index")  # None = delete all

        obj = bpy.data.objects.get(obj_name)
        if not obj or obj.type != "GPENCIL":
            raise ValueError(f"Grease Pencil object not found: {obj_name}")

        gpd = obj.data
        layer = gpd.layers.get(layer_name) if layer_name else gpd.layers.active
        if not layer:
            raise ValueError("No layer found")

        gp_frame = layer.frames.get(frame)
        if not gp_frame:
            raise ValueError(f"No frame at {frame}")

        if stroke_index is not None:
            if stroke_index < len(gp_frame.strokes):
                gp_frame.strokes.remove(gp_frame.strokes[stroke_index])
                return {"deleted": True, "count": 1}
            else:
                raise ValueError(f"Stroke index out of range: {stroke_index}")
        else:
            count = len(gp_frame.strokes)
            gp_frame.strokes.clear()
            return {"deleted": True, "count": count}

    def _fill(self, params: dict[str, Any]) -> dict:
        """Add a fill to enclosed strokes."""
        if not self._bpy:
            return {"filled": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")
        layer_name = params.get("layer")
        material_index = params.get("material_index", 0)

        obj = bpy.data.objects.get(obj_name)
        if not obj or obj.type != "GPENCIL":
            raise ValueError(f"Grease Pencil object not found: {obj_name}")

        # Select object and enter draw mode
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.mode_set(mode='PAINT_GPENCIL')

        # Use fill tool
        bpy.ops.gpencil.fill(
            on_back=params.get("on_back", False),
            material_index=material_index,
        )

        bpy.ops.object.mode_set(mode='OBJECT')

        return {"filled": True, "object": obj_name}

    # -------------------------------------------------------------------------
    # Materials
    # -------------------------------------------------------------------------

    def _material_add(self, params: dict[str, Any]) -> dict:
        """Add a Grease Pencil material."""
        if not self._bpy:
            return {"added": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")
        mat_name = params.get("name", "GP_Material")

        obj = bpy.data.objects.get(obj_name)
        if not obj or obj.type != "GPENCIL":
            raise ValueError(f"Grease Pencil object not found: {obj_name}")

        # Create material
        mat = bpy.data.materials.new(name=mat_name)
        bpy.data.materials.create_gpencil_data(mat)
        gpmat = mat.grease_pencil

        # Configure stroke
        if params.get("stroke_color"):
            gpmat.color = params["stroke_color"][:3]
            if len(params["stroke_color"]) > 3:
                gpmat.color[3] = params["stroke_color"][3]
        if params.get("stroke_style"):
            gpmat.stroke_style = params["stroke_style"]  # LINE, DOTS, BOX

        # Configure fill
        if params.get("fill_color"):
            gpmat.fill_color = params["fill_color"][:3]
            if len(params["fill_color"]) > 3:
                gpmat.fill_color[3] = params["fill_color"][3]
        if params.get("fill_style"):
            gpmat.fill_style = params["fill_style"]  # SOLID, GRADIENT, CHECKER, TEXTURE

        if params.get("show_stroke") is not None:
            gpmat.show_stroke = params["show_stroke"]
        if params.get("show_fill") is not None:
            gpmat.show_fill = params["show_fill"]

        obj.data.materials.append(mat)

        return {
            "added": True,
            "object": obj_name,
            "material": mat_name,
            "index": len(obj.data.materials) - 1,
        }

    def _material_assign(self, params: dict[str, Any]) -> dict:
        """Assign material to strokes."""
        if not self._bpy:
            return {"assigned": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")
        material_index = params.get("index", 0)
        layer_name = params.get("layer")
        frame = params.get("frame")

        obj = bpy.data.objects.get(obj_name)
        if not obj or obj.type != "GPENCIL":
            raise ValueError(f"Grease Pencil object not found: {obj_name}")

        gpd = obj.data
        layer = gpd.layers.get(layer_name) if layer_name else gpd.layers.active
        if not layer:
            raise ValueError("No layer found")

        frames_to_update = [layer.frames.get(frame)] if frame else list(layer.frames)

        count = 0
        for gp_frame in frames_to_update:
            if gp_frame:
                for stroke in gp_frame.strokes:
                    stroke.material_index = material_index
                    count += 1

        return {"assigned": True, "strokes": count}

    # -------------------------------------------------------------------------
    # Modifiers
    # -------------------------------------------------------------------------

    def _modifier_add(self, params: dict[str, Any]) -> dict:
        """Add Grease Pencil modifier."""
        if not self._bpy:
            return {"added": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")
        mod_type = params.get("type")  # NOISE, SMOOTH, SUBDIV, SIMPLIFY, THICK, TINT, OFFSET, etc.
        mod_name = params.get("name")

        obj = bpy.data.objects.get(obj_name)
        if not obj or obj.type != "GPENCIL":
            raise ValueError(f"Grease Pencil object not found: {obj_name}")

        mod = obj.grease_pencil_modifiers.new(mod_name or mod_type, mod_type)

        # Apply common settings
        if params.get("factor"):
            if hasattr(mod, "factor"):
                mod.factor = params["factor"]
        if params.get("strength"):
            if hasattr(mod, "strength"):
                mod.strength = params["strength"]

        return {
            "added": True,
            "object": obj_name,
            "modifier": mod.name,
            "type": mod_type,
        }

    def _modifier_remove(self, params: dict[str, Any]) -> dict:
        """Remove Grease Pencil modifier."""
        if not self._bpy:
            return {"removed": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")
        mod_name = params.get("name")

        obj = bpy.data.objects.get(obj_name)
        if not obj or obj.type != "GPENCIL":
            raise ValueError(f"Grease Pencil object not found: {obj_name}")

        mod = obj.grease_pencil_modifiers.get(mod_name)
        if not mod:
            raise ValueError(f"Modifier not found: {mod_name}")

        obj.grease_pencil_modifiers.remove(mod)
        return {"removed": True, "object": obj_name, "modifier": mod_name}

    def _modifier_list(self, params: dict[str, Any]) -> dict:
        """List Grease Pencil modifiers."""
        if not self._bpy:
            return {"modifiers": [], "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")

        obj = bpy.data.objects.get(obj_name)
        if not obj or obj.type != "GPENCIL":
            raise ValueError(f"Grease Pencil object not found: {obj_name}")

        mods = [
            {
                "name": mod.name,
                "type": mod.type,
                "show_viewport": mod.show_viewport,
                "show_render": mod.show_render,
            }
            for mod in obj.grease_pencil_modifiers
        ]

        return {"object": obj_name, "modifiers": mods}

    # -------------------------------------------------------------------------
    # Effects (Visual Effects)
    # -------------------------------------------------------------------------

    def _effect_add(self, params: dict[str, Any]) -> dict:
        """Add visual effect to Grease Pencil."""
        if not self._bpy:
            return {"added": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")
        fx_type = params.get("type")  # BLUR, COLORIZE, FLIP, GLOW, LIGHT, PIXELATE, RIM, SHADOW, SWIRL, WAVE
        fx_name = params.get("name")

        obj = bpy.data.objects.get(obj_name)
        if not obj or obj.type != "GPENCIL":
            raise ValueError(f"Grease Pencil object not found: {obj_name}")

        fx = obj.shader_effects.new(fx_name or fx_type, fx_type)

        # Type-specific settings
        if fx_type == "BLUR" and params.get("size"):
            fx.size = params["size"]
        elif fx_type == "GLOW":
            if params.get("color"):
                fx.glow_color = params["color"][:3]
            if params.get("radius"):
                fx.radius = params["radius"]
        elif fx_type == "SHADOW":
            if params.get("color"):
                fx.shadow_color = params["color"][:3]
            if params.get("offset"):
                fx.offset = params["offset"]

        return {
            "added": True,
            "object": obj_name,
            "effect": fx.name,
            "type": fx_type,
        }

    def _effect_remove(self, params: dict[str, Any]) -> dict:
        """Remove visual effect."""
        if not self._bpy:
            return {"removed": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")
        fx_name = params.get("name")

        obj = bpy.data.objects.get(obj_name)
        if not obj or obj.type != "GPENCIL":
            raise ValueError(f"Grease Pencil object not found: {obj_name}")

        fx = obj.shader_effects.get(fx_name)
        if not fx:
            raise ValueError(f"Effect not found: {fx_name}")

        obj.shader_effects.remove(fx)
        return {"removed": True, "object": obj_name, "effect": fx_name}

    def _effect_list(self, params: dict[str, Any]) -> dict:
        """List visual effects."""
        if not self._bpy:
            return {"effects": [], "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")

        obj = bpy.data.objects.get(obj_name)
        if not obj or obj.type != "GPENCIL":
            raise ValueError(f"Grease Pencil object not found: {obj_name}")

        effects = [
            {
                "name": fx.name,
                "type": fx.type,
                "show_viewport": fx.show_viewport,
                "show_render": fx.show_render,
            }
            for fx in obj.shader_effects
        ]

        return {"object": obj_name, "effects": effects}

    # -------------------------------------------------------------------------
    # Conversion
    # -------------------------------------------------------------------------

    def _convert_to_mesh(self, params: dict[str, Any]) -> dict:
        """Convert Grease Pencil to mesh."""
        if not self._bpy:
            return {"converted": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")
        keep_original = params.get("keep_original", True)

        obj = bpy.data.objects.get(obj_name)
        if not obj or obj.type != "GPENCIL":
            raise ValueError(f"Grease Pencil object not found: {obj_name}")

        bpy.context.view_layer.objects.active = obj
        obj.select_set(True)

        bpy.ops.gpencil.convert(type='POLY', use_timing_data=False)

        # New mesh object
        new_obj = bpy.context.active_object

        if not keep_original:
            bpy.data.objects.remove(obj, do_unlink=True)

        return {
            "converted": True,
            "original": obj_name,
            "mesh": new_obj.name,
        }

    def _convert_from_curve(self, params: dict[str, Any]) -> dict:
        """Convert curve to Grease Pencil."""
        if not self._bpy:
            return {"converted": True, "mock": True}

        bpy = self._bpy
        curve_name = params.get("curve")
        keep_original = params.get("keep_original", True)

        curve = bpy.data.objects.get(curve_name)
        if not curve or curve.type != "CURVE":
            raise ValueError(f"Curve not found: {curve_name}")

        bpy.context.view_layer.objects.active = curve
        curve.select_set(True)

        bpy.ops.gpencil.bake_mesh_animation(
            target='NEW',
            frame_start=bpy.context.scene.frame_start,
            frame_end=bpy.context.scene.frame_start,  # Single frame for static curve
        )

        gp_obj = bpy.context.active_object

        if not keep_original:
            bpy.data.objects.remove(curve, do_unlink=True)

        return {
            "converted": True,
            "curve": curve_name,
            "gpencil": gp_obj.name,
        }

    def method_list(self) -> list[MethodInfo]:
        """Return list of Grease Pencil methods."""
        return [
            # Object management
            MethodInfo("gpencil.create", "Create Grease Pencil object", [
                ParamInfo("name", "string", False, "GPencil"),
                ParamInfo("location", "array", False, [0, 0, 0]),
            ]),
            MethodInfo("gpencil.list", "List Grease Pencil objects", []),
            MethodInfo("gpencil.delete", "Delete Grease Pencil object", [
                ParamInfo("name", "string", True),
            ]),
            MethodInfo("gpencil.info", "Get Grease Pencil info", [
                ParamInfo("name", "string", True),
            ]),

            # Layers
            MethodInfo("gpencil.layer.add", "Add layer", [
                ParamInfo("object", "string", True),
                ParamInfo("name", "string", False, "Layer"),
                ParamInfo("opacity", "number", False),
                ParamInfo("color", "array", False, description="Channel color [r,g,b]"),
            ]),
            MethodInfo("gpencil.layer.remove", "Remove layer", [
                ParamInfo("object", "string", True),
                ParamInfo("name", "string", True),
            ]),
            MethodInfo("gpencil.layer.list", "List layers", [
                ParamInfo("object", "string", True),
            ]),
            MethodInfo("gpencil.layer.settings", "Update layer settings", [
                ParamInfo("object", "string", True),
                ParamInfo("name", "string", True),
                ParamInfo("opacity", "number", False),
                ParamInfo("visible", "boolean", False),
                ParamInfo("locked", "boolean", False),
                ParamInfo("blend_mode", "string", False),
            ]),

            # Drawing
            MethodInfo("gpencil.stroke.add", "Add stroke programmatically", [
                ParamInfo("object", "string", True),
                ParamInfo("points", "array", True, description="Array of [x,y,z] coordinates"),
                ParamInfo("layer", "string", False, description="Layer name (default: active)"),
                ParamInfo("frame", "integer", False, description="Frame number"),
                ParamInfo("pressure", "number", False, 1.0),
                ParamInfo("strength", "number", False, 1.0),
                ParamInfo("line_width", "integer", False, 100),
                ParamInfo("material_index", "integer", False),
            ]),
            MethodInfo("gpencil.stroke.delete", "Delete strokes", [
                ParamInfo("object", "string", True),
                ParamInfo("layer", "string", False),
                ParamInfo("frame", "integer", False),
                ParamInfo("index", "integer", False, description="Stroke index (all if not specified)"),
            ]),
            MethodInfo("gpencil.fill", "Fill enclosed area", [
                ParamInfo("object", "string", True),
                ParamInfo("layer", "string", False),
                ParamInfo("material_index", "integer", False, 0),
                ParamInfo("on_back", "boolean", False, False),
            ]),

            # Materials
            MethodInfo("gpencil.material.add", "Add GP material", [
                ParamInfo("object", "string", True),
                ParamInfo("name", "string", False, "GP_Material"),
                ParamInfo("stroke_color", "array", False, description="[r,g,b,a]"),
                ParamInfo("fill_color", "array", False),
                ParamInfo("stroke_style", "string", False, description="LINE, DOTS, BOX"),
                ParamInfo("fill_style", "string", False, description="SOLID, GRADIENT, CHECKER, TEXTURE"),
                ParamInfo("show_stroke", "boolean", False),
                ParamInfo("show_fill", "boolean", False),
            ]),
            MethodInfo("gpencil.material.assign", "Assign material to strokes", [
                ParamInfo("object", "string", True),
                ParamInfo("index", "integer", False, 0),
                ParamInfo("layer", "string", False),
                ParamInfo("frame", "integer", False),
            ]),

            # Modifiers
            MethodInfo("gpencil.modifier.add", "Add GP modifier", [
                ParamInfo("object", "string", True),
                ParamInfo("type", "string", True, description="NOISE, SMOOTH, SUBDIV, SIMPLIFY, THICK, TINT, OFFSET"),
                ParamInfo("name", "string", False),
                ParamInfo("factor", "number", False),
                ParamInfo("strength", "number", False),
            ]),
            MethodInfo("gpencil.modifier.remove", "Remove GP modifier", [
                ParamInfo("object", "string", True),
                ParamInfo("name", "string", True),
            ]),
            MethodInfo("gpencil.modifier.list", "List GP modifiers", [
                ParamInfo("object", "string", True),
            ]),

            # Effects
            MethodInfo("gpencil.effect.add", "Add visual effect", [
                ParamInfo("object", "string", True),
                ParamInfo("type", "string", True, description="BLUR, COLORIZE, FLIP, GLOW, LIGHT, PIXELATE, RIM, SHADOW, SWIRL, WAVE"),
                ParamInfo("name", "string", False),
                ParamInfo("size", "array", False, description="For BLUR"),
                ParamInfo("color", "array", False, description="For GLOW/SHADOW"),
                ParamInfo("radius", "number", False, description="For GLOW"),
                ParamInfo("offset", "array", False, description="For SHADOW"),
            ]),
            MethodInfo("gpencil.effect.remove", "Remove visual effect", [
                ParamInfo("object", "string", True),
                ParamInfo("name", "string", True),
            ]),
            MethodInfo("gpencil.effect.list", "List visual effects", [
                ParamInfo("object", "string", True),
            ]),

            # Conversion
            MethodInfo("gpencil.convert_to_mesh", "Convert to mesh", [
                ParamInfo("object", "string", True),
                ParamInfo("keep_original", "boolean", False, True),
            ]),
            MethodInfo("gpencil.convert_from_curve", "Convert curve to GP", [
                ParamInfo("curve", "string", True),
                ParamInfo("keep_original", "boolean", False, True),
            ]),
        ]
