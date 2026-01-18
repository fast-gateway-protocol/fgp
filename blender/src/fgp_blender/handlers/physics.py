"""
Physics simulation handler for FGP Blender.

Provides methods for rigid body, cloth, fluid, soft body,
particle systems, and force fields.

CHANGELOG:
01/17/2026 - Initial implementation (Claude)
"""

from __future__ import annotations

import logging
from typing import Any, Optional

from fgp_daemon import MethodInfo, ParamInfo

logger = logging.getLogger(__name__)


class PhysicsHandler:
    """Handler for physics simulation operations."""

    def __init__(self, bpy: Any, sessions: Any) -> None:
        self._bpy = bpy
        self._sessions = sessions

    def dispatch(self, category: str, action: str, params: dict[str, Any]) -> Any:
        """Dispatch physics method calls."""
        method_name = f"{category}.{action}" if action else category

        handlers = {
            # Rigid body
            "physics.rigid_body.add": self._rigid_body_add,
            "physics.rigid_body.remove": self._rigid_body_remove,
            "physics.rigid_body.settings": self._rigid_body_settings,
            # Cloth
            "physics.cloth.add": self._cloth_add,
            "physics.cloth.remove": self._cloth_remove,
            "physics.cloth.settings": self._cloth_settings,
            # Fluid
            "physics.fluid.add": self._fluid_add,
            "physics.fluid.remove": self._fluid_remove,
            "physics.fluid.settings": self._fluid_settings,
            # Soft body
            "physics.soft_body.add": self._soft_body_add,
            "physics.soft_body.remove": self._soft_body_remove,
            "physics.soft_body.settings": self._soft_body_settings,
            # Particles
            "physics.particles.add": self._particles_add,
            "physics.particles.remove": self._particles_remove,
            "physics.particles.settings": self._particles_settings,
            # Force fields
            "physics.force_field.add": self._force_field_add,
            "physics.force_field.remove": self._force_field_remove,
            "physics.force_field.settings": self._force_field_settings,
            # Simulation control
            "physics.simulate": self._simulate,
            "physics.bake": self._bake,
            "physics.clear_cache": self._clear_cache,
        }

        handler = handlers.get(method_name)
        if handler is None:
            raise ValueError(f"Unknown physics method: {method_name}")

        return handler(params)

    # -------------------------------------------------------------------------
    # Rigid Body
    # -------------------------------------------------------------------------

    def _rigid_body_add(self, params: dict[str, Any]) -> dict:
        """Add rigid body physics to an object."""
        if not self._bpy:
            return {"added": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")
        rb_type = params.get("type", "ACTIVE")  # ACTIVE or PASSIVE

        obj = bpy.data.objects.get(obj_name)
        if not obj:
            raise ValueError(f"Object not found: {obj_name}")

        # Ensure rigid body world exists
        if bpy.context.scene.rigidbody_world is None:
            bpy.ops.rigidbody.world_add()

        # Add rigid body
        bpy.context.view_layer.objects.active = obj
        bpy.ops.rigidbody.object_add(type=rb_type)

        # Apply settings
        rb = obj.rigid_body
        if params.get("mass"):
            rb.mass = params["mass"]
        if params.get("friction"):
            rb.friction = params["friction"]
        if params.get("restitution"):
            rb.restitution = params["restitution"]
        if params.get("collision_shape"):
            rb.collision_shape = params["collision_shape"]

        return {
            "added": True,
            "object": obj_name,
            "type": rb_type,
            "mass": rb.mass,
            "friction": rb.friction,
            "restitution": rb.restitution,
        }

    def _rigid_body_remove(self, params: dict[str, Any]) -> dict:
        """Remove rigid body physics from an object."""
        if not self._bpy:
            return {"removed": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")

        obj = bpy.data.objects.get(obj_name)
        if not obj:
            raise ValueError(f"Object not found: {obj_name}")

        bpy.context.view_layer.objects.active = obj
        bpy.ops.rigidbody.object_remove()

        return {"removed": True, "object": obj_name}

    def _rigid_body_settings(self, params: dict[str, Any]) -> dict:
        """Update rigid body settings."""
        if not self._bpy:
            return {"updated": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")

        obj = bpy.data.objects.get(obj_name)
        if not obj or not obj.rigid_body:
            raise ValueError(f"Object has no rigid body: {obj_name}")

        rb = obj.rigid_body

        if "type" in params:
            rb.type = params["type"]
        if "mass" in params:
            rb.mass = params["mass"]
        if "friction" in params:
            rb.friction = params["friction"]
        if "restitution" in params:
            rb.restitution = params["restitution"]
        if "collision_shape" in params:
            rb.collision_shape = params["collision_shape"]
        if "kinematic" in params:
            rb.kinematic = params["kinematic"]
        if "linear_damping" in params:
            rb.linear_damping = params["linear_damping"]
        if "angular_damping" in params:
            rb.angular_damping = params["angular_damping"]

        return {
            "updated": True,
            "object": obj_name,
            "type": rb.type,
            "mass": rb.mass,
        }

    # -------------------------------------------------------------------------
    # Cloth
    # -------------------------------------------------------------------------

    def _cloth_add(self, params: dict[str, Any]) -> dict:
        """Add cloth simulation to an object."""
        if not self._bpy:
            return {"added": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")

        obj = bpy.data.objects.get(obj_name)
        if not obj:
            raise ValueError(f"Object not found: {obj_name}")

        # Add cloth modifier
        cloth_mod = obj.modifiers.new(name="Cloth", type='CLOTH')
        cloth = cloth_mod.settings

        # Apply presets/settings
        if params.get("preset"):
            preset = params["preset"].upper()
            if preset == "COTTON":
                cloth.mass = 0.3
                cloth.air_damping = 1.0
            elif preset == "SILK":
                cloth.mass = 0.15
                cloth.air_damping = 0.5
            elif preset == "LEATHER":
                cloth.mass = 0.4
                cloth.air_damping = 1.0
            elif preset == "RUBBER":
                cloth.mass = 3.0
                cloth.air_damping = 1.0

        if params.get("mass"):
            cloth.mass = params["mass"]
        if params.get("air_damping"):
            cloth.air_damping = params["air_damping"]
        if params.get("quality"):
            cloth.quality = params["quality"]

        return {
            "added": True,
            "object": obj_name,
            "mass": cloth.mass,
            "air_damping": cloth.air_damping,
        }

    def _cloth_remove(self, params: dict[str, Any]) -> dict:
        """Remove cloth simulation from an object."""
        if not self._bpy:
            return {"removed": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")

        obj = bpy.data.objects.get(obj_name)
        if not obj:
            raise ValueError(f"Object not found: {obj_name}")

        for mod in obj.modifiers:
            if mod.type == 'CLOTH':
                obj.modifiers.remove(mod)
                return {"removed": True, "object": obj_name}

        raise ValueError(f"No cloth modifier on object: {obj_name}")

    def _cloth_settings(self, params: dict[str, Any]) -> dict:
        """Update cloth simulation settings."""
        if not self._bpy:
            return {"updated": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")

        obj = bpy.data.objects.get(obj_name)
        if not obj:
            raise ValueError(f"Object not found: {obj_name}")

        cloth_mod = None
        for mod in obj.modifiers:
            if mod.type == 'CLOTH':
                cloth_mod = mod
                break

        if not cloth_mod:
            raise ValueError(f"No cloth modifier on object: {obj_name}")

        cloth = cloth_mod.settings

        if "mass" in params:
            cloth.mass = params["mass"]
        if "air_damping" in params:
            cloth.air_damping = params["air_damping"]
        if "quality" in params:
            cloth.quality = params["quality"]
        if "tension_stiffness" in params:
            cloth.tension_stiffness = params["tension_stiffness"]
        if "compression_stiffness" in params:
            cloth.compression_stiffness = params["compression_stiffness"]
        if "bending_stiffness" in params:
            cloth.bending_stiffness = params["bending_stiffness"]

        return {"updated": True, "object": obj_name}

    # -------------------------------------------------------------------------
    # Fluid
    # -------------------------------------------------------------------------

    def _fluid_add(self, params: dict[str, Any]) -> dict:
        """Add fluid simulation to an object."""
        if not self._bpy:
            return {"added": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")
        fluid_type = params.get("type", "DOMAIN")  # DOMAIN, FLOW, EFFECTOR

        obj = bpy.data.objects.get(obj_name)
        if not obj:
            raise ValueError(f"Object not found: {obj_name}")

        # Add fluid modifier
        fluid_mod = obj.modifiers.new(name="Fluid", type='FLUID')
        fluid_mod.fluid_type = fluid_type

        settings = fluid_mod.domain_settings if fluid_type == "DOMAIN" else \
                   fluid_mod.flow_settings if fluid_type == "FLOW" else \
                   fluid_mod.effector_settings

        # Domain-specific settings
        if fluid_type == "DOMAIN":
            if params.get("domain_type"):
                settings.domain_type = params["domain_type"]  # GAS or LIQUID
            if params.get("resolution_max"):
                settings.resolution_max = params["resolution_max"]

        # Flow-specific settings
        elif fluid_type == "FLOW":
            if params.get("flow_type"):
                settings.flow_type = params["flow_type"]  # SMOKE, FIRE, BOTH, LIQUID
            if params.get("flow_behavior"):
                settings.flow_behavior = params["flow_behavior"]  # INFLOW, OUTFLOW, GEOMETRY

        return {
            "added": True,
            "object": obj_name,
            "fluid_type": fluid_type,
        }

    def _fluid_remove(self, params: dict[str, Any]) -> dict:
        """Remove fluid simulation from an object."""
        if not self._bpy:
            return {"removed": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")

        obj = bpy.data.objects.get(obj_name)
        if not obj:
            raise ValueError(f"Object not found: {obj_name}")

        for mod in obj.modifiers:
            if mod.type == 'FLUID':
                obj.modifiers.remove(mod)
                return {"removed": True, "object": obj_name}

        raise ValueError(f"No fluid modifier on object: {obj_name}")

    def _fluid_settings(self, params: dict[str, Any]) -> dict:
        """Update fluid simulation settings."""
        if not self._bpy:
            return {"updated": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")

        obj = bpy.data.objects.get(obj_name)
        if not obj:
            raise ValueError(f"Object not found: {obj_name}")

        fluid_mod = None
        for mod in obj.modifiers:
            if mod.type == 'FLUID':
                fluid_mod = mod
                break

        if not fluid_mod:
            raise ValueError(f"No fluid modifier on object: {obj_name}")

        # Update based on fluid type
        if fluid_mod.fluid_type == "DOMAIN":
            settings = fluid_mod.domain_settings
            if "resolution_max" in params:
                settings.resolution_max = params["resolution_max"]
            if "time_scale" in params:
                settings.time_scale = params["time_scale"]
            if "use_adaptive_domain" in params:
                settings.use_adaptive_domain = params["use_adaptive_domain"]

        return {"updated": True, "object": obj_name}

    # -------------------------------------------------------------------------
    # Soft Body
    # -------------------------------------------------------------------------

    def _soft_body_add(self, params: dict[str, Any]) -> dict:
        """Add soft body physics to an object."""
        if not self._bpy:
            return {"added": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")

        obj = bpy.data.objects.get(obj_name)
        if not obj:
            raise ValueError(f"Object not found: {obj_name}")

        # Add soft body modifier
        sb_mod = obj.modifiers.new(name="Softbody", type='SOFT_BODY')
        sb = sb_mod.settings

        if params.get("mass"):
            sb.mass = params["mass"]
        if params.get("friction"):
            sb.friction = params["friction"]
        if params.get("goal_strength"):
            sb.goal_default = params["goal_strength"]

        return {
            "added": True,
            "object": obj_name,
            "mass": sb.mass,
        }

    def _soft_body_remove(self, params: dict[str, Any]) -> dict:
        """Remove soft body physics from an object."""
        if not self._bpy:
            return {"removed": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")

        obj = bpy.data.objects.get(obj_name)
        if not obj:
            raise ValueError(f"Object not found: {obj_name}")

        for mod in obj.modifiers:
            if mod.type == 'SOFT_BODY':
                obj.modifiers.remove(mod)
                return {"removed": True, "object": obj_name}

        raise ValueError(f"No soft body modifier on object: {obj_name}")

    def _soft_body_settings(self, params: dict[str, Any]) -> dict:
        """Update soft body settings."""
        if not self._bpy:
            return {"updated": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")

        obj = bpy.data.objects.get(obj_name)
        if not obj:
            raise ValueError(f"Object not found: {obj_name}")

        sb_mod = None
        for mod in obj.modifiers:
            if mod.type == 'SOFT_BODY':
                sb_mod = mod
                break

        if not sb_mod:
            raise ValueError(f"No soft body modifier on object: {obj_name}")

        sb = sb_mod.settings

        if "mass" in params:
            sb.mass = params["mass"]
        if "friction" in params:
            sb.friction = params["friction"]
        if "speed" in params:
            sb.speed = params["speed"]

        return {"updated": True, "object": obj_name}

    # -------------------------------------------------------------------------
    # Particle Systems
    # -------------------------------------------------------------------------

    def _particles_add(self, params: dict[str, Any]) -> dict:
        """Add particle system to an object."""
        if not self._bpy:
            return {"added": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")
        name = params.get("name", "Particles")

        obj = bpy.data.objects.get(obj_name)
        if not obj:
            raise ValueError(f"Object not found: {obj_name}")

        # Add particle system
        obj.modifiers.new(name=name, type='PARTICLE_SYSTEM')
        ps = obj.particle_systems[-1]
        settings = ps.settings

        # Configure
        if params.get("count"):
            settings.count = params["count"]
        if params.get("type"):
            settings.type = params["type"]  # EMITTER or HAIR
        if params.get("lifetime"):
            settings.lifetime = params["lifetime"]
        if params.get("emit_from"):
            settings.emit_from = params["emit_from"]  # VERT, FACE, VOLUME
        if params.get("physics_type"):
            settings.physics_type = params["physics_type"]  # NO, NEWTON, KEYED, BOIDS, FLUID

        return {
            "added": True,
            "object": obj_name,
            "name": ps.name,
            "count": settings.count,
            "type": settings.type,
        }

    def _particles_remove(self, params: dict[str, Any]) -> dict:
        """Remove particle system from an object."""
        if not self._bpy:
            return {"removed": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")
        ps_name = params.get("name")

        obj = bpy.data.objects.get(obj_name)
        if not obj:
            raise ValueError(f"Object not found: {obj_name}")

        if ps_name:
            # Remove specific particle system
            for i, ps in enumerate(obj.particle_systems):
                if ps.name == ps_name:
                    obj.particle_systems.active_index = i
                    bpy.ops.object.particle_system_remove()
                    return {"removed": True, "object": obj_name, "name": ps_name}
            raise ValueError(f"Particle system not found: {ps_name}")
        else:
            # Remove all
            while obj.particle_systems:
                obj.particle_systems.active_index = 0
                bpy.ops.object.particle_system_remove()
            return {"removed": True, "object": obj_name, "all": True}

    def _particles_settings(self, params: dict[str, Any]) -> dict:
        """Update particle system settings."""
        if not self._bpy:
            return {"updated": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")
        ps_name = params.get("name")

        obj = bpy.data.objects.get(obj_name)
        if not obj:
            raise ValueError(f"Object not found: {obj_name}")

        ps = None
        if ps_name:
            for p in obj.particle_systems:
                if p.name == ps_name:
                    ps = p
                    break
        else:
            ps = obj.particle_systems.active if obj.particle_systems else None

        if not ps:
            raise ValueError(f"No particle system on object: {obj_name}")

        settings = ps.settings

        if "count" in params:
            settings.count = params["count"]
        if "lifetime" in params:
            settings.lifetime = params["lifetime"]
        if "lifetime_random" in params:
            settings.lifetime_random = params["lifetime_random"]
        if "velocity_factor" in params:
            settings.normal_factor = params["velocity_factor"]
        if "gravity" in params:
            settings.effector_weights.gravity = params["gravity"]
        if "size" in params:
            settings.particle_size = params["size"]
        if "render_type" in params:
            settings.render_type = params["render_type"]  # NONE, HALO, LINE, PATH, OBJECT, COLLECTION

        return {"updated": True, "object": obj_name, "name": ps.name}

    # -------------------------------------------------------------------------
    # Force Fields
    # -------------------------------------------------------------------------

    def _force_field_add(self, params: dict[str, Any]) -> dict:
        """Add force field to scene."""
        if not self._bpy:
            return {"added": True, "mock": True}

        bpy = self._bpy
        field_type = params.get("type", "FORCE")  # FORCE, WIND, VORTEX, MAGNET, HARMONIC, CHARGE, TURBULENCE, DRAG, FLUID_FLOW
        location = params.get("location", [0, 0, 0])
        name = params.get("name")

        # Create empty for force field
        bpy.ops.object.empty_add(type='PLAIN_AXES', location=location)
        obj = bpy.context.active_object
        if name:
            obj.name = name

        # Add force field
        obj.field.type = field_type

        if params.get("strength"):
            obj.field.strength = params["strength"]
        if params.get("flow"):
            obj.field.flow = params["flow"]
        if params.get("noise"):
            obj.field.noise = params["noise"]
        if params.get("falloff_type"):
            obj.field.falloff_type = params["falloff_type"]

        return {
            "added": True,
            "name": obj.name,
            "type": field_type,
            "strength": obj.field.strength,
        }

    def _force_field_remove(self, params: dict[str, Any]) -> dict:
        """Remove force field."""
        if not self._bpy:
            return {"removed": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")

        obj = bpy.data.objects.get(obj_name)
        if not obj:
            raise ValueError(f"Object not found: {obj_name}")

        if obj.field.type == 'NONE':
            raise ValueError(f"Object is not a force field: {obj_name}")

        obj.field.type = 'NONE'
        return {"removed": True, "object": obj_name}

    def _force_field_settings(self, params: dict[str, Any]) -> dict:
        """Update force field settings."""
        if not self._bpy:
            return {"updated": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")

        obj = bpy.data.objects.get(obj_name)
        if not obj:
            raise ValueError(f"Object not found: {obj_name}")

        if obj.field.type == 'NONE':
            raise ValueError(f"Object is not a force field: {obj_name}")

        field = obj.field

        if "type" in params:
            field.type = params["type"]
        if "strength" in params:
            field.strength = params["strength"]
        if "flow" in params:
            field.flow = params["flow"]
        if "noise" in params:
            field.noise = params["noise"]
        if "seed" in params:
            field.seed = params["seed"]
        if "falloff_type" in params:
            field.falloff_type = params["falloff_type"]
        if "falloff_power" in params:
            field.falloff_power = params["falloff_power"]

        return {"updated": True, "object": obj_name}

    # -------------------------------------------------------------------------
    # Simulation Control
    # -------------------------------------------------------------------------

    def _simulate(self, params: dict[str, Any]) -> dict:
        """Run physics simulation for specified frames."""
        if not self._bpy:
            return {"simulated": True, "mock": True}

        bpy = self._bpy
        start_frame = params.get("start", bpy.context.scene.frame_start)
        end_frame = params.get("end", bpy.context.scene.frame_end)

        # Step through frames to simulate
        original_frame = bpy.context.scene.frame_current

        for frame in range(start_frame, end_frame + 1):
            bpy.context.scene.frame_set(frame)

        # Return to original frame
        bpy.context.scene.frame_set(original_frame)

        return {
            "simulated": True,
            "start": start_frame,
            "end": end_frame,
            "frames": end_frame - start_frame + 1,
        }

    def _bake(self, params: dict[str, Any]) -> dict:
        """Bake physics simulation."""
        if not self._bpy:
            return {"baked": True, "mock": True}

        bpy = self._bpy
        obj_name = params.get("object")
        physics_type = params.get("type")  # RIGID_BODY, CLOTH, FLUID, SOFT_BODY, PARTICLES

        if obj_name:
            obj = bpy.data.objects.get(obj_name)
            if not obj:
                raise ValueError(f"Object not found: {obj_name}")
            bpy.context.view_layer.objects.active = obj

        # Bake based on type
        if physics_type == "RIGID_BODY" or not physics_type:
            if bpy.context.scene.rigidbody_world:
                bpy.ops.rigidbody.bake_to_keyframes()
        elif physics_type == "CLOTH":
            bpy.ops.ptcache.bake_all(bake=True)
        elif physics_type == "FLUID":
            bpy.ops.fluid.bake_all()
        elif physics_type == "PARTICLES":
            bpy.ops.ptcache.bake_all(bake=True)
        else:
            bpy.ops.ptcache.bake_all(bake=True)

        return {"baked": True, "type": physics_type}

    def _clear_cache(self, params: dict[str, Any]) -> dict:
        """Clear physics cache."""
        if not self._bpy:
            return {"cleared": True, "mock": True}

        bpy = self._bpy
        physics_type = params.get("type")

        if physics_type == "RIGID_BODY":
            if bpy.context.scene.rigidbody_world:
                bpy.ops.rigidbody.bake_to_keyframes(frame_start=1, frame_end=1)
        elif physics_type == "FLUID":
            bpy.ops.fluid.free_all()
        else:
            bpy.ops.ptcache.free_bake_all()

        return {"cleared": True, "type": physics_type}

    def method_list(self) -> list[MethodInfo]:
        """Return list of physics methods."""
        return [
            # Rigid Body
            MethodInfo("physics.rigid_body.add", "Add rigid body physics to object", [
                ParamInfo("object", "string", True, description="Object name"),
                ParamInfo("type", "string", False, "ACTIVE", description="ACTIVE or PASSIVE"),
                ParamInfo("mass", "number", False, description="Object mass"),
                ParamInfo("friction", "number", False, description="Friction coefficient"),
                ParamInfo("restitution", "number", False, description="Bounciness"),
                ParamInfo("collision_shape", "string", False, description="CONVEX_HULL, MESH, BOX, SPHERE, etc."),
            ]),
            MethodInfo("physics.rigid_body.remove", "Remove rigid body from object", [
                ParamInfo("object", "string", True, description="Object name"),
            ]),
            MethodInfo("physics.rigid_body.settings", "Update rigid body settings", [
                ParamInfo("object", "string", True, description="Object name"),
                ParamInfo("type", "string", False),
                ParamInfo("mass", "number", False),
                ParamInfo("friction", "number", False),
                ParamInfo("restitution", "number", False),
                ParamInfo("kinematic", "boolean", False),
                ParamInfo("linear_damping", "number", False),
                ParamInfo("angular_damping", "number", False),
            ]),

            # Cloth
            MethodInfo("physics.cloth.add", "Add cloth simulation to object", [
                ParamInfo("object", "string", True, description="Object name"),
                ParamInfo("preset", "string", False, description="COTTON, SILK, LEATHER, RUBBER"),
                ParamInfo("mass", "number", False),
                ParamInfo("air_damping", "number", False),
                ParamInfo("quality", "integer", False),
            ]),
            MethodInfo("physics.cloth.remove", "Remove cloth simulation", [
                ParamInfo("object", "string", True),
            ]),
            MethodInfo("physics.cloth.settings", "Update cloth settings", [
                ParamInfo("object", "string", True),
                ParamInfo("mass", "number", False),
                ParamInfo("air_damping", "number", False),
                ParamInfo("tension_stiffness", "number", False),
                ParamInfo("compression_stiffness", "number", False),
                ParamInfo("bending_stiffness", "number", False),
            ]),

            # Fluid
            MethodInfo("physics.fluid.add", "Add fluid simulation", [
                ParamInfo("object", "string", True),
                ParamInfo("type", "string", False, "DOMAIN", description="DOMAIN, FLOW, or EFFECTOR"),
                ParamInfo("domain_type", "string", False, description="GAS or LIQUID (for domains)"),
                ParamInfo("flow_type", "string", False, description="SMOKE, FIRE, BOTH, LIQUID (for flows)"),
                ParamInfo("resolution_max", "integer", False),
            ]),
            MethodInfo("physics.fluid.remove", "Remove fluid simulation", [
                ParamInfo("object", "string", True),
            ]),
            MethodInfo("physics.fluid.settings", "Update fluid settings", [
                ParamInfo("object", "string", True),
                ParamInfo("resolution_max", "integer", False),
                ParamInfo("time_scale", "number", False),
                ParamInfo("use_adaptive_domain", "boolean", False),
            ]),

            # Soft Body
            MethodInfo("physics.soft_body.add", "Add soft body physics", [
                ParamInfo("object", "string", True),
                ParamInfo("mass", "number", False),
                ParamInfo("friction", "number", False),
                ParamInfo("goal_strength", "number", False),
            ]),
            MethodInfo("physics.soft_body.remove", "Remove soft body", [
                ParamInfo("object", "string", True),
            ]),
            MethodInfo("physics.soft_body.settings", "Update soft body settings", [
                ParamInfo("object", "string", True),
                ParamInfo("mass", "number", False),
                ParamInfo("friction", "number", False),
                ParamInfo("speed", "number", False),
            ]),

            # Particles
            MethodInfo("physics.particles.add", "Add particle system", [
                ParamInfo("object", "string", True),
                ParamInfo("name", "string", False),
                ParamInfo("count", "integer", False, 1000),
                ParamInfo("type", "string", False, "EMITTER", description="EMITTER or HAIR"),
                ParamInfo("lifetime", "number", False),
                ParamInfo("emit_from", "string", False, description="VERT, FACE, VOLUME"),
                ParamInfo("physics_type", "string", False, description="NO, NEWTON, KEYED, BOIDS, FLUID"),
            ]),
            MethodInfo("physics.particles.remove", "Remove particle system", [
                ParamInfo("object", "string", True),
                ParamInfo("name", "string", False, description="Remove specific system, or all if not specified"),
            ]),
            MethodInfo("physics.particles.settings", "Update particle settings", [
                ParamInfo("object", "string", True),
                ParamInfo("name", "string", False),
                ParamInfo("count", "integer", False),
                ParamInfo("lifetime", "number", False),
                ParamInfo("lifetime_random", "number", False),
                ParamInfo("velocity_factor", "number", False),
                ParamInfo("gravity", "number", False),
                ParamInfo("size", "number", False),
                ParamInfo("render_type", "string", False),
            ]),

            # Force Fields
            MethodInfo("physics.force_field.add", "Add force field to scene", [
                ParamInfo("type", "string", False, "FORCE", description="FORCE, WIND, VORTEX, MAGNET, HARMONIC, CHARGE, TURBULENCE, DRAG"),
                ParamInfo("location", "array", False, [0, 0, 0]),
                ParamInfo("name", "string", False),
                ParamInfo("strength", "number", False),
                ParamInfo("flow", "number", False),
                ParamInfo("noise", "number", False),
            ]),
            MethodInfo("physics.force_field.remove", "Remove force field", [
                ParamInfo("object", "string", True),
            ]),
            MethodInfo("physics.force_field.settings", "Update force field settings", [
                ParamInfo("object", "string", True),
                ParamInfo("type", "string", False),
                ParamInfo("strength", "number", False),
                ParamInfo("flow", "number", False),
                ParamInfo("noise", "number", False),
                ParamInfo("falloff_type", "string", False),
                ParamInfo("falloff_power", "number", False),
            ]),

            # Simulation Control
            MethodInfo("physics.simulate", "Run physics simulation", [
                ParamInfo("start", "integer", False, description="Start frame"),
                ParamInfo("end", "integer", False, description="End frame"),
            ]),
            MethodInfo("physics.bake", "Bake physics to keyframes", [
                ParamInfo("object", "string", False, description="Object to bake (or all)"),
                ParamInfo("type", "string", False, description="RIGID_BODY, CLOTH, FLUID, SOFT_BODY, PARTICLES"),
            ]),
            MethodInfo("physics.clear_cache", "Clear physics cache", [
                ParamInfo("type", "string", False, description="Physics type to clear"),
            ]),
        ]
