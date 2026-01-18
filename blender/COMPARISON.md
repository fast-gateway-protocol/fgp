# FGP Blender vs MCP Alternatives - Feature Comparison

**Generated:** 01/17/2026

## Summary

| Implementation | Total Methods | Architecture | Cold Start | Active Development |
|----------------|---------------|--------------|------------|-------------------|
| **FGP Blender** | **55+** | UNIX socket daemon | **0ms** (warm) | New |
| ahujasid/blender-mcp | 22 | TCP socket + addon | ~4-5s | Active (v1.4.0) |
| poly-mcp/Blender-MCP-Server | ~33 | HTTP server addon | ~4-5s | Active |
| ooMike1/BlenderMCP | ~15 | MCP server | ~4-5s | Moderate |

## Detailed Feature Comparison

### Object Operations

| Feature | FGP Blender | ahujasid | poly-mcp | ooMike1 |
|---------|-------------|----------|----------|---------|
| Create primitives (cube, sphere, etc.) | 10 types | Via code | 5 types | 5 types |
| Delete objects | Yes | Via code | Yes | Yes |
| List objects | Yes | Yes | Yes | - |
| Object info | Yes | Yes | Yes | - |
| Transform (loc/rot/scale) | Yes | Via code | Yes | - |
| Duplicate | Yes | Via code | Yes | - |
| Parent/child | Yes | Via code | - | - |
| Hide/show | Yes | Via code | - | - |
| Select | Yes | Via code | - | - |
| Rename | Yes | Via code | - | - |

### Mesh Operations

| Feature | FGP Blender | ahujasid | poly-mcp | ooMike1 |
|---------|-------------|----------|----------|---------|
| Add modifiers | Yes (all types) | Via code | Yes | Yes |
| Apply modifiers | Yes | Via code | - | - |
| Boolean operations | Yes | Via code | - | Yes |
| Subdivision | Yes | Via code | - | Yes |
| Smooth shading | Yes | Via code | - | Yes |
| Decimate | Yes | Via code | - | - |
| Geometry nodes | Yes | Via code | Yes | - |
| Edit mode ops | Yes | Via code | - | - |

### Materials & Shaders

| Feature | FGP Blender | ahujasid | poly-mcp | ooMike1 |
|---------|-------------|----------|----------|---------|
| Create material | Yes | Via code | Yes | Yes |
| Assign material | Yes | Via code | Yes | Yes |
| Set color | Yes | Via code | Yes | - |
| Set PBR properties | Yes | Via code | Yes | - |
| Shader node add | Yes | Via code | Yes | - |
| Shader node connect | Yes | Via code | Yes | - |
| Procedural materials | - | Via code | Yes | - |

### Rendering

| Feature | FGP Blender | ahujasid | poly-mcp | ooMike1 |
|---------|-------------|----------|----------|---------|
| Render image | Yes (async) | Via code | Yes | - |
| Render animation | Yes (async) | Via code | - | - |
| Set engine | Yes | Via code | Yes | - |
| Set resolution | Yes | Via code | Yes | - |
| Set samples | Yes | Via code | - | - |
| Preview render | Yes | Via code | - | - |
| Job queue | **Yes** | - | - | - |
| Progress tracking | **Yes** | - | - | - |

### Camera & Lighting

| Feature | FGP Blender | ahujasid | poly-mcp | ooMike1 |
|---------|-------------|----------|----------|---------|
| Create camera | Yes | Via code | Yes | - |
| Set active camera | Yes | Via code | - | - |
| Look at target | Yes | Via code | Yes | - |
| Set lens | Yes | Via code | - | - |
| Create light | Yes | Via code | Yes | - |
| Set light color | Yes | Via code | - | - |
| Set light energy | Yes | Via code | - | - |
| HDRI environment | Yes | Yes | Yes | - |

### Animation

| Feature | FGP Blender | ahujasid | poly-mcp | ooMike1 |
|---------|-------------|----------|----------|---------|
| Insert keyframe | Yes | Via code | - | - |
| Delete keyframe | Yes | Via code | - | - |
| List keyframes | Yes | Via code | - | - |
| Set frame | Yes | Via code | - | - |
| Set frame range | Yes | Via code | - | - |
| Bake animation | Yes | Via code | - | - |
| Create armature | Yes | Via code | - | - |
| Add bones | Yes | Via code | - | - |
| Pose bones | Yes | Via code | - | - |

### Import/Export

| Feature | FGP Blender | ahujasid | poly-mcp | ooMike1 |
|---------|-------------|----------|----------|---------|
| FBX | Yes | Via code | - | Yes |
| OBJ | Yes | Via code | - | Yes |
| GLTF/GLB | Yes | Via code | - | - |
| USD | Yes | Via code | - | - |
| STL | Yes | Via code | - | Yes |
| PLY | Yes | Via code | - | Yes |
| Alembic | Yes | Via code | - | - |
| Auto-detect format | **Yes** | - | - | - |

### Asset Integration

| Feature | FGP Blender | ahujasid | poly-mcp | ooMike1 |
|---------|-------------|----------|----------|---------|
| Poly Haven search | Yes | Yes | - | - |
| Poly Haven download | Yes | Yes | - | - |
| Sketchfab search | Yes | Yes | - | - |
| Sketchfab import | Yes | Yes | - | - |
| Hyper3D Rodin | - | Yes | - | - |
| Hunyuan3D | - | Yes | - | - |

### Scene Management

| Feature | FGP Blender | ahujasid | poly-mcp | ooMike1 |
|---------|-------------|----------|----------|---------|
| Get scene info | Yes | Yes | Yes | - |
| Create new scene | Yes | Via code | Yes | - |
| Load .blend file | Yes | Via code | - | - |
| Save scene | Yes | Via code | - | Yes |
| Clear scene | Yes | Via code | Yes | Yes |
| List scenes | Yes | Via code | - | - |

### Session Management

| Feature | FGP Blender | ahujasid | poly-mcp | ooMike1 |
|---------|-------------|----------|----------|---------|
| Multiple sessions | **Yes** | - | - | - |
| Session isolation | **Yes** | - | - | - |
| Session persistence | **Yes** | - | - | - |
| Session reset | **Yes** | - | - | - |

### Advanced Features

| Feature | FGP Blender | ahujasid | poly-mcp | ooMike1 |
|---------|-------------|----------|----------|---------|
| Execute Python | Yes | Yes | - | - |
| Viewport screenshot | Yes | Yes | Yes | - |
| Particle systems | - | Via code | Yes | - |
| Rigid body physics | - | Via code | Yes | - |
| Cloth simulation | - | Via code | Yes | - |
| Fluid simulation | - | Via code | Yes | - |
| Force fields | - | Via code | Yes | - |
| Grease pencil | - | Via code | Yes | - |

## Architecture Comparison

### FGP Blender
- **Protocol**: NDJSON over UNIX socket
- **Daemon**: Persistent process (warm startup)
- **State**: Session-based with .blend file isolation
- **Async**: Built-in job queue for long operations
- **Cold start**: 0ms (daemon already running)

### ahujasid/blender-mcp
- **Protocol**: JSON over TCP socket
- **Addon**: Runs inside Blender as addon
- **State**: Single Blender instance
- **Async**: None
- **Cold start**: ~4-5s (Blender + addon startup)

### poly-mcp/Blender-MCP-Server
- **Protocol**: HTTP endpoints
- **Addon**: Runs inside Blender as addon
- **State**: Single Blender instance
- **Async**: Thread-safe execution queue
- **Cold start**: ~4-5s

## Unique FGP Blender Features

1. **Session Isolation**: Multiple parallel workflows with separate .blend files
2. **Async Job Queue**: Non-blocking renders with progress tracking
3. **Warm Daemon**: Zero cold-start latency
4. **Comprehensive API**: 55+ discrete methods vs code execution
5. **Direct Method Calls**: No need to write Python code
6. **Session Persistence**: Sessions survive daemon restarts

## Unique Competitor Features

### ahujasid/blender-mcp
- AI model generation (Hyper3D Rodin, Hunyuan3D)
- Flexible code execution for any operation

### poly-mcp/Blender-MCP-Server
- Physics simulations (rigid body, cloth, fluid)
- Particle systems
- Grease pencil support
- Thread-safe operation queue

## Sources

- [ahujasid/blender-mcp](https://github.com/ahujasid/blender-mcp)
- [poly-mcp/Blender-MCP-Server](https://github.com/poly-mcp/Blender-MCP-Server)
- [ooMike1/BlenderMCP](https://github.com/ooMike1/BlenderMCP)
- [dhakalnirajan/blender-open-mcp](https://github.com/dhakalnirajan/blender-open-mcp)
