# FGP Blender Daemon

Fast Blender automation daemon with 55+ methods for 3D modeling, rendering, and animation. Part of the [Fast Gateway Protocol](https://fgp.dev) ecosystem.

## Performance

| Metric | FGP Blender | MCP Implementations |
|--------|-------------|---------------------|
| Cold start | **0ms** (warm daemon) | 4-5s |
| Object create | 5-15ms | 200-300ms |
| Scene info | 2-5ms | 150-200ms |
| Screenshot | 50-100ms | 300-500ms |

**Conservative estimate: 20-50x faster** for typical operations.

## Installation

```bash
# From the FGP repository
cd ~/projects/fgp/blender
pip install -e .

# Or install directly (coming soon)
pip install fgp-blender
```

## Quick Start

```bash
# Start the daemon
fgp-blender start

# Check status
fgp-blender status

# Create a cube
fgp-blender call object.create '{"type": "CUBE", "name": "MyCube"}'

# List objects
fgp-blender call object.list

# Render image
fgp-blender call render.image '{"output": "/tmp/render.png", "wait": true}'

# Stop daemon
fgp-blender stop
```

## Features

### Session Management
- Isolated sessions with separate .blend files
- Sessions persist across daemon restarts
- Default session for quick operations
- Named sessions for project work

### Object Operations
Create primitives: CUBE, SPHERE, CYLINDER, CONE, TORUS, PLANE, CIRCLE, MONKEY, TEXT, EMPTY

```bash
fgp-blender call object.create '{"type": "SPHERE", "radius": 2, "location": [0, 0, 1]}'
```

### Mesh Operations
- Modifiers (subdivision, boolean, decimate, etc.)
- Boolean operations (difference, union, intersect)
- Geometry nodes
- Edit mode operations (extrude, inset, bevel)

### Materials & Shaders
- PBR materials with metallic/roughness workflow
- Shader node manipulation
- Node connections

```bash
fgp-blender call material.create '{"name": "Gold", "color": [1, 0.8, 0.3], "metallic": 1.0, "roughness": 0.1}'
```

### Async Rendering
Non-blocking render jobs with progress tracking:

```bash
# Start render (returns immediately)
fgp-blender call render.image '{"output": "/tmp/scene.png"}'
# Returns: {"job_id": "abc123", "status": "pending"}

# Check progress
fgp-blender call job.status '{"job_id": "abc123"}'

# Or wait for completion
fgp-blender call render.image '{"output": "/tmp/scene.png", "wait": true}'
```

### Camera & Lighting
- Create cameras with lens settings
- Point camera at objects or locations
- Create lights (Point, Sun, Spot, Area)
- HDRI environment lighting

### Animation
- Keyframe insertion and management
- Frame range control
- Armature creation and posing
- Animation baking

### Import/Export
Supported formats:
- FBX, OBJ, GLTF/GLB, USD, STL, PLY, Alembic, SVG

### Asset Library Integration
- **Poly Haven**: Search and download HDRIs, textures
- **Sketchfab**: Search and import 3D models (requires API token)

```bash
# Search Poly Haven HDRIs
fgp-blender call assets.polyhaven.search '{"category": "hdris", "query": "studio"}'

# Download and apply
fgp-blender call assets.polyhaven.download '{"id": "studio_small_08", "resolution": "2k", "type": "hdri"}'
fgp-blender call light.hdri '{"filepath": "~/.fgp/assets/polyhaven/hdri/studio_small_08_2k.hdr"}'
```

## Architecture

```
fgp-blender/
├── src/fgp_blender/
│   ├── main.py          # CLI entry point
│   ├── service.py       # BlenderService (FgpService impl)
│   ├── session.py       # SessionManager
│   ├── jobs.py          # Async JobQueue
│   └── handlers/
│       ├── scene.py     # Scene operations
│       ├── object.py    # Object CRUD
│       ├── mesh.py      # Mesh operations
│       ├── material.py  # Materials & shaders
│       ├── render.py    # Rendering
│       ├── camera.py    # Camera & lighting
│       ├── animation.py # Animation
│       └── io.py        # Import/export & assets
└── manifest.json        # FGP registry manifest
```

## CLI Commands

```bash
fgp-blender start              # Start daemon in background
fgp-blender start --foreground # Start in foreground
fgp-blender stop               # Stop daemon
fgp-blender status             # Check daemon status
fgp-blender methods            # List available methods
fgp-blender call <method> '<json params>'  # Call a method
```

## Socket Location

```
~/.fgp/services/blender/daemon.sock
```

## Direct Socket Communication

```bash
echo '{"id":"1","v":1,"method":"health","params":{}}' | nc -U ~/.fgp/services/blender/daemon.sock
```

## Requirements

- Python 3.10+
- Blender 3.0+ (for `bpy` module)
- fgp-daemon Python SDK

## Development

```bash
# Install in development mode
pip install -e ".[dev]"

# Run tests
pytest

# Type checking
mypy src/fgp_blender
```

## License

MIT

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.
