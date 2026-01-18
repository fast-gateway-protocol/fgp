#!/usr/bin/env python3
"""
FGP Blender Showcase Demo

Demonstrates all the capabilities of FGP Blender daemon visually.
Run in Blender: blender --python showcase_demo.py

This script simulates what the FGP daemon can do by calling the same
underlying operations.
"""

import bpy
import math
import time
import random

# Clear the default scene
def clear_scene():
    """Clear all objects from scene."""
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.delete(use_global=False)

    # Clear materials
    for mat in list(bpy.data.materials):
        bpy.data.materials.remove(mat)

def pause(seconds=0.5):
    """Pause for visual effect."""
    bpy.context.view_layer.update()
    time.sleep(seconds)

def set_viewport_shading(mode='MATERIAL'):
    """Set viewport shading mode."""
    for area in bpy.context.screen.areas:
        if area.type == 'VIEW_3D':
            for space in area.spaces:
                if space.type == 'VIEW_3D':
                    space.shading.type = mode

def frame_all():
    """Frame all objects in viewport."""
    for area in bpy.context.screen.areas:
        if area.type == 'VIEW_3D':
            for region in area.regions:
                if region.type == 'WINDOW':
                    with bpy.context.temp_override(area=area, region=region):
                        bpy.ops.view3d.view_all()

print("\n" + "="*60)
print("FGP Blender Showcase - 170 Methods Demo")
print("="*60 + "\n")

# ============================================================================
# PHASE 1: Object Creation
# ============================================================================
print("[1/8] Object Creation - 10 primitive types...")

clear_scene()
set_viewport_shading('SOLID')

primitives = [
    ('CUBE', (-4, -4, 0), "Cube"),
    ('SPHERE', (0, -4, 0), "Sphere"),
    ('CYLINDER', (4, -4, 0), "Cylinder"),
    ('CONE', (-4, 0, 0), "Cone"),
    ('TORUS', (0, 0, 0), "Torus"),
    ('PLANE', (4, 0, 1), "Plane"),
    ('CIRCLE', (-4, 4, 0), "Circle"),
    ('GRID', (0, 4, 0), "Grid"),
    ('MONKEY', (4, 4, 0), "Suzanne"),
]

for prim_type, location, name in primitives:
    if prim_type == 'CUBE':
        bpy.ops.mesh.primitive_cube_add(location=location)
    elif prim_type == 'SPHERE':
        bpy.ops.mesh.primitive_uv_sphere_add(location=location)
    elif prim_type == 'CYLINDER':
        bpy.ops.mesh.primitive_cylinder_add(location=location)
    elif prim_type == 'CONE':
        bpy.ops.mesh.primitive_cone_add(location=location)
    elif prim_type == 'TORUS':
        bpy.ops.mesh.primitive_torus_add(location=location)
    elif prim_type == 'PLANE':
        bpy.ops.mesh.primitive_plane_add(location=location, size=2)
    elif prim_type == 'CIRCLE':
        bpy.ops.mesh.primitive_circle_add(location=location)
    elif prim_type == 'GRID':
        bpy.ops.mesh.primitive_grid_add(location=location)
    elif prim_type == 'MONKEY':
        bpy.ops.mesh.primitive_monkey_add(location=location)

    bpy.context.active_object.name = name

frame_all()
pause(1)
print("  ✓ Created 9 different primitives")

# ============================================================================
# PHASE 2: Materials & Procedural Textures
# ============================================================================
print("\n[2/8] Materials & Procedural Textures...")

set_viewport_shading('MATERIAL')

# Create colorful materials for each object
colors = [
    ("Red", (1, 0.1, 0.1, 1)),
    ("Orange", (1, 0.5, 0.1, 1)),
    ("Yellow", (1, 1, 0.1, 1)),
    ("Green", (0.1, 0.8, 0.2, 1)),
    ("Cyan", (0.1, 0.8, 0.8, 1)),
    ("Blue", (0.2, 0.3, 1, 1)),
    ("Purple", (0.6, 0.2, 0.8, 1)),
    ("Pink", (1, 0.4, 0.6, 1)),
    ("White", (0.9, 0.9, 0.9, 1)),
]

for i, obj in enumerate(bpy.data.objects):
    if obj.type == 'MESH' and i < len(colors):
        mat = bpy.data.materials.new(name=colors[i][0])
        mat.use_nodes = True
        principled = mat.node_tree.nodes.get("Principled BSDF")
        if principled:
            principled.inputs["Base Color"].default_value = colors[i][1]
            principled.inputs["Metallic"].default_value = 0.5
            principled.inputs["Roughness"].default_value = 0.3
        obj.data.materials.append(mat)

pause(1)
print("  ✓ Applied 9 colorful materials")

# Create procedural material demo
print("  Creating procedural materials demo...")

# Clear and create procedural material showcase
clear_scene()

# Create planes for procedural material showcase
procedural_presets = ['noise', 'checker', 'brick', 'wave', 'voronoi', 'marble']
for i, preset in enumerate(procedural_presets):
    bpy.ops.mesh.primitive_plane_add(size=3, location=(i * 3.5 - 8, 0, 0))
    plane = bpy.context.active_object
    plane.name = f"Procedural_{preset}"

    # Create procedural material
    mat = bpy.data.materials.new(name=f"Mat_{preset}")
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    links = mat.node_tree.links

    # Get output and principled
    output = nodes.get("Material Output")
    principled = nodes.get("Principled BSDF")

    # Add texture coordinate
    tex_coord = nodes.new("ShaderNodeTexCoord")
    tex_coord.location = (-600, 0)

    # Add color ramp
    ramp = nodes.new("ShaderNodeValToRGB")
    ramp.location = (-200, 0)
    ramp.color_ramp.elements[0].color = (0.1, 0.3, 0.6, 1)  # Dark blue
    ramp.color_ramp.elements[1].color = (0.9, 0.7, 0.3, 1)  # Gold
    links.new(ramp.outputs["Color"], principled.inputs["Base Color"])

    # Add texture based on preset
    if preset == 'noise' or preset == 'marble':
        tex = nodes.new("ShaderNodeTexNoise")
        tex.inputs["Scale"].default_value = 5
        if preset == 'marble':
            tex.inputs["Distortion"].default_value = 5
        links.new(tex_coord.outputs["Generated"], tex.inputs["Vector"])
        links.new(tex.outputs["Fac"], ramp.inputs["Fac"])
    elif preset == 'checker':
        tex = nodes.new("ShaderNodeTexChecker")
        tex.inputs["Scale"].default_value = 8
        links.new(tex_coord.outputs["Generated"], tex.inputs["Vector"])
        links.new(tex.outputs["Fac"], ramp.inputs["Fac"])
    elif preset == 'brick':
        tex = nodes.new("ShaderNodeTexBrick")
        tex.inputs["Scale"].default_value = 5
        links.new(tex_coord.outputs["Generated"], tex.inputs["Vector"])
        links.new(tex.outputs["Fac"], ramp.inputs["Fac"])
    elif preset == 'wave':
        tex = nodes.new("ShaderNodeTexWave")
        tex.inputs["Scale"].default_value = 3
        links.new(tex_coord.outputs["Generated"], tex.inputs["Vector"])
        links.new(tex.outputs["Fac"], ramp.inputs["Fac"])
    elif preset == 'voronoi':
        tex = nodes.new("ShaderNodeTexVoronoi")
        tex.inputs["Scale"].default_value = 5
        links.new(tex_coord.outputs["Generated"], tex.inputs["Vector"])
        links.new(tex.outputs["Distance"], ramp.inputs["Fac"])

    tex.location = (-400, 0)
    plane.data.materials.append(mat)

frame_all()
pause(1.5)
print("  ✓ Created 6 procedural material presets (noise, checker, brick, wave, voronoi, marble)")

# ============================================================================
# PHASE 3: Mesh Modifiers
# ============================================================================
print("\n[3/8] Mesh Modifiers Demo...")

clear_scene()

# Create objects to showcase modifiers
modifier_demos = [
    ('Array', 'ARRAY'),
    ('Mirror', 'MIRROR'),
    ('Bevel', 'BEVEL'),
    ('Subdivision', 'SUBSURF'),
    ('Solidify', 'SOLIDIFY'),
    ('Boolean', 'BOOLEAN'),
]

for i, (name, mod_type) in enumerate(modifier_demos):
    bpy.ops.mesh.primitive_cube_add(size=1.5, location=(i * 3 - 7.5, 0, 0))
    obj = bpy.context.active_object
    obj.name = f"Modifier_{name}"

    # Create material
    mat = bpy.data.materials.new(name=f"Mat_{name}")
    mat.use_nodes = True
    principled = mat.node_tree.nodes.get("Principled BSDF")
    hue = i / len(modifier_demos)
    principled.inputs["Base Color"].default_value = (
        0.5 + 0.5 * math.cos(hue * 2 * math.pi),
        0.5 + 0.5 * math.cos(hue * 2 * math.pi + 2.1),
        0.5 + 0.5 * math.cos(hue * 2 * math.pi + 4.2),
        1
    )
    obj.data.materials.append(mat)

    # Add modifier
    if mod_type == 'ARRAY':
        mod = obj.modifiers.new(name, 'ARRAY')
        mod.count = 4
        mod.relative_offset_displace = (1.1, 0, 0)
    elif mod_type == 'MIRROR':
        mod = obj.modifiers.new(name, 'MIRROR')
        mod.use_axis[0] = True
    elif mod_type == 'BEVEL':
        mod = obj.modifiers.new(name, 'BEVEL')
        mod.segments = 3
        mod.width = 0.1
    elif mod_type == 'SUBSURF':
        mod = obj.modifiers.new(name, 'SUBSURF')
        mod.levels = 2
    elif mod_type == 'SOLIDIFY':
        bpy.ops.mesh.primitive_plane_add(size=1.5, location=(i * 3 - 7.5, 0, 0))
        obj = bpy.context.active_object
        obj.name = f"Modifier_{name}"
        mod = obj.modifiers.new(name, 'SOLIDIFY')
        mod.thickness = 0.2
        obj.data.materials.append(mat)
    elif mod_type == 'BOOLEAN':
        # Create a sphere to boolean with
        bpy.ops.mesh.primitive_uv_sphere_add(radius=0.8, location=(i * 3 - 7.5, 0, 0))
        sphere = bpy.context.active_object
        sphere.name = "Boolean_Cutter"
        sphere.hide_viewport = True
        sphere.hide_render = True
        mod = obj.modifiers.new(name, 'BOOLEAN')
        mod.operation = 'DIFFERENCE'
        mod.object = sphere

frame_all()
pause(1.5)
print("  ✓ Demonstrated 6 modifiers (Array, Mirror, Bevel, Subdivision, Solidify, Boolean)")

# ============================================================================
# PHASE 4: Camera & Lighting
# ============================================================================
print("\n[4/8] Camera & Lighting Setup...")

# Add camera
bpy.ops.object.camera_add(location=(15, -15, 10))
camera = bpy.context.active_object
camera.name = "Main_Camera"

# Point camera at center
direction = bpy.data.objects["Main_Camera"].location
bpy.context.scene.camera = camera

# Add constraint to look at center
bpy.ops.object.constraint_add(type='TRACK_TO')
camera.constraints["Track To"].target = bpy.data.objects.get("Modifier_Bevel") or bpy.data.objects[0]
camera.constraints["Track To"].track_axis = 'TRACK_NEGATIVE_Z'
camera.constraints["Track To"].up_axis = 'UP_Y'

# Add lights
light_configs = [
    ("Key_Light", 'SUN', (5, -5, 10), 3),
    ("Fill_Light", 'AREA', (-8, -5, 5), 200),
    ("Rim_Light", 'SPOT', (0, 10, 8), 500),
]

for name, light_type, location, energy in light_configs:
    bpy.ops.object.light_add(type=light_type, location=location)
    light = bpy.context.active_object
    light.name = name
    light.data.energy = energy
    if light_type == 'SPOT':
        light.data.spot_size = 1.2

pause(1)
print("  ✓ Added camera and 3 lights (Sun, Area, Spot)")

# ============================================================================
# PHASE 5: Physics Simulation
# ============================================================================
print("\n[5/8] Physics Simulation Demo...")

clear_scene()

# Create ground plane
bpy.ops.mesh.primitive_plane_add(size=20, location=(0, 0, 0))
ground = bpy.context.active_object
ground.name = "Ground"
mat = bpy.data.materials.new(name="Ground_Mat")
mat.use_nodes = True
mat.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (0.2, 0.2, 0.2, 1)
ground.data.materials.append(mat)

# Add rigid body world
bpy.ops.rigidbody.world_add()

# Make ground passive rigid body
ground.select_set(True)
bpy.context.view_layer.objects.active = ground
bpy.ops.rigidbody.object_add(type='PASSIVE')

# Create falling objects
for i in range(5):
    for j in range(3):
        bpy.ops.mesh.primitive_cube_add(size=0.8, location=(i - 2, j - 1, 3 + i * 0.5 + j * 0.3))
        cube = bpy.context.active_object
        cube.name = f"RigidBody_{i}_{j}"

        # Random rotation
        cube.rotation_euler = (random.random(), random.random(), random.random())

        # Material
        mat = bpy.data.materials.new(name=f"RB_Mat_{i}_{j}")
        mat.use_nodes = True
        principled = mat.node_tree.nodes["Principled BSDF"]
        principled.inputs["Base Color"].default_value = (
            random.random() * 0.5 + 0.5,
            random.random() * 0.5 + 0.5,
            random.random() * 0.5 + 0.5,
            1
        )
        principled.inputs["Metallic"].default_value = 0.8
        principled.inputs["Roughness"].default_value = 0.2
        cube.data.materials.append(mat)

        # Make active rigid body
        cube.select_set(True)
        bpy.context.view_layer.objects.active = cube
        bpy.ops.rigidbody.object_add(type='ACTIVE')
        cube.rigid_body.mass = 1.0
        cube.rigid_body.friction = 0.5
        cube.rigid_body.restitution = 0.3

# Add camera and light
bpy.ops.object.camera_add(location=(10, -10, 8))
camera = bpy.context.active_object
camera.name = "Physics_Camera"
bpy.context.scene.camera = camera
bpy.ops.object.constraint_add(type='TRACK_TO')
camera.constraints["Track To"].track_axis = 'TRACK_NEGATIVE_Z'
camera.constraints["Track To"].up_axis = 'UP_Y'

bpy.ops.object.light_add(type='SUN', location=(5, -5, 10))
bpy.context.active_object.data.energy = 3

frame_all()

# Run a few frames of simulation
print("  Running physics simulation...")
bpy.context.scene.frame_start = 1
bpy.context.scene.frame_end = 60
for frame in range(1, 30):
    bpy.context.scene.frame_set(frame)

pause(1)
print("  ✓ Set up rigid body physics with 15 falling cubes")

# ============================================================================
# PHASE 6: Grease Pencil
# ============================================================================
print("\n[6/8] Grease Pencil 2D Drawing...")

# Create grease pencil object
bpy.ops.object.gpencil_add(type='EMPTY', location=(0, 5, 5))
gp = bpy.context.active_object
gp.name = "GP_Drawing"

# Add layer
layer = gp.data.layers.new("Strokes", set_active=True)

# Add frame
frame = layer.frames.new(1)

# Add material
mat = bpy.data.materials.new(name="GP_Material")
bpy.data.materials.create_gpencil_data(mat)
mat.grease_pencil.color = (1, 0.3, 0.1, 1)  # Orange stroke
mat.grease_pencil.fill_color = (1, 0.8, 0.2, 0.5)  # Yellow fill
mat.grease_pencil.show_fill = True
gp.data.materials.append(mat)

# Draw a star shape
import math
star_points = []
for i in range(10):
    angle = i * math.pi / 5 - math.pi / 2
    radius = 2 if i % 2 == 0 else 1
    x = radius * math.cos(angle)
    z = radius * math.sin(angle) + 5
    star_points.append((x, 5, z))

stroke = frame.strokes.new()
stroke.display_mode = '3DSPACE'
stroke.line_width = 50
stroke.points.add(len(star_points) + 1)
for i, pt in enumerate(star_points):
    stroke.points[i].co = pt
    stroke.points[i].pressure = 1.0
    stroke.points[i].strength = 1.0
stroke.points[len(star_points)].co = star_points[0]  # Close the shape

# Add a visual effect (glow)
gp.shader_effects.new("Glow", 'GLOW')
glow = gp.shader_effects["Glow"]
glow.glow_color = (1, 0.5, 0, 1)
glow.radius = 20

print("  ✓ Created Grease Pencil drawing with glow effect")

# ============================================================================
# PHASE 7: Animation
# ============================================================================
print("\n[7/8] Animation & Keyframes...")

# Animate the grease pencil object
gp.keyframe_insert(data_path="rotation_euler", frame=1)
gp.rotation_euler.z = math.pi * 2
gp.keyframe_insert(data_path="rotation_euler", frame=60)

# Make it spin smoothly
for fcurve in gp.animation_data.action.fcurves:
    for kf in fcurve.keyframe_points:
        kf.interpolation = 'LINEAR'

print("  ✓ Added rotation animation to Grease Pencil star")

# ============================================================================
# PHASE 8: Rendering Setup
# ============================================================================
print("\n[8/8] Rendering Setup...")

# Set render engine to Cycles for better quality
bpy.context.scene.render.engine = 'CYCLES'
bpy.context.scene.cycles.device = 'GPU'
bpy.context.scene.cycles.samples = 64

# Set resolution
bpy.context.scene.render.resolution_x = 1920
bpy.context.scene.render.resolution_y = 1080

print("  ✓ Configured Cycles renderer at 1920x1080")

# ============================================================================
# SUMMARY
# ============================================================================
print("\n" + "="*60)
print("FGP Blender Showcase Complete!")
print("="*60)
print("""
Demonstrated capabilities:

📦 Object Operations (10 methods)
   - Created 9 primitive types

🎨 Materials (22 methods)
   - Applied colorful PBR materials
   - Created 6 procedural presets

🔧 Mesh Modifiers (13 methods)
   - Array, Mirror, Bevel, Subdivision, Solidify, Boolean

📷 Camera & Lighting (12 methods)
   - Camera with tracking
   - Sun, Area, Spot lights

⚡ Physics (21 methods)
   - Rigid body simulation with 15 objects

✏️ Grease Pencil (21 methods)
   - 2D star drawing with glow effect

🎬 Animation (12 methods)
   - Rotation keyframes

🖼️ Rendering (7 methods)
   - Cycles GPU at 1920x1080

Total: 170 discrete methods available!

The scene is ready - explore it in the viewport!
Press SPACE to play the physics simulation.
""")

# Reset to frame 1
bpy.context.scene.frame_set(1)
frame_all()
