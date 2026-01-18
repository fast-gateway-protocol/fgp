#!/usr/bin/env python3
"""
Physics Playground Showcase - Live Collaborative Scene Building

This script builds a stylized Rube Goldberg physics scene step by step,
demonstrating FGP Blender's conversational workflow.

Usage:
    python3 physics_playground_showcase.py
"""

import socket
import json
import time
import sys

SOCKET_PATH = "/Users/wolfgangschoenberger/.fgp/services/blender/daemon.sock"

# Color palette: "Sunset Playground"
COLORS = {
    "coral": (1.0, 0.4, 0.4, 1.0),      # Coral pink
    "blue": (0.2, 0.5, 1.0, 1.0),        # Electric blue
    "yellow": (1.0, 0.9, 0.2, 1.0),      # Yellow
    "orange": (1.0, 0.6, 0.1, 1.0),      # Orange
    "red": (1.0, 0.3, 0.2, 1.0),         # Red
    "mint": (0.4, 1.0, 0.7, 1.0),        # Mint green
    "gold": (1.0, 0.8, 0.3, 1.0),        # Gold
    "cyan": (0.3, 0.9, 1.0, 1.0),        # Cyan/turquoise
    "purple": (0.5, 0.2, 0.8, 1.0),      # Deep purple
}


def send_command(method: str, params: dict, description: str = None) -> dict:
    """Send a command to the FGP daemon and return the result."""
    request = {
        "id": f"cmd-{time.time()}",
        "v": 1,
        "method": method,
        "params": params
    }

    # Print the conversational command
    if description:
        print(f"\n🎨 {description}")
    print(f"   → {method}", end="", flush=True)

    sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    sock.settimeout(60)

    try:
        sock.connect(SOCKET_PATH)
        sock.sendall((json.dumps(request) + "\n").encode())

        response = b""
        while b"\n" not in response:
            chunk = sock.recv(4096)
            if not chunk:
                break
            response += chunk

        result = json.loads(response.decode())

        if result.get("ok"):
            ms = result.get("meta", {}).get("server_ms", 0)
            print(f" ✓ ({ms:.1f}ms)")
            return result.get("result", {})
        else:
            error = result.get("error", {}).get("message", "Unknown error")
            print(f" ✗ ({error})")
            return None
    except Exception as e:
        print(f" ✗ ({e})")
        return None
    finally:
        sock.close()


def clear_scene():
    """Remove default objects."""
    print("\n" + "="*60)
    print("PHASE 1: Preparing the Canvas")
    print("="*60)

    # Delete the default cube
    send_command("object.delete", {"names": ["Cube"]}, "Clearing the default cube...")
    time.sleep(0.3)


def create_environment():
    """Create gradient background and lighting."""
    print("\n" + "="*60)
    print("PHASE 2: Setting the Stage")
    print("="*60)

    # Create background plane
    send_command("object.create", {
        "type": "plane",
        "name": "Background",
        "location": [0, 10, 0],
        "scale": [30, 1, 20]
    }, "Creating gradient background plane...")
    time.sleep(0.2)

    # Rotate to face camera
    send_command("object.transform", {
        "name": "Background",
        "rotation": [1.5708, 0, 0]  # 90 degrees in radians
    }, "Rotating background to face camera...")
    time.sleep(0.2)

    # Create emission material for background
    send_command("material.create", {
        "name": "BackgroundMat",
        "color": COLORS["purple"],
        "emission_strength": 2.0
    }, "Adding purple emission material...")
    time.sleep(0.2)

    send_command("material.assign", {
        "object": "Background",
        "material": "BackgroundMat"
    }, "Applying to background...")
    time.sleep(0.2)

    # Create key light (warm orange)
    send_command("light.create", {
        "type": "SUN",
        "name": "KeyLight",
        "location": [5, -5, 10],
        "energy": 3.0,
        "color": [1.0, 0.8, 0.6]
    }, "Adding warm key light from above...")
    time.sleep(0.2)

    # Create rim light (cool blue)
    send_command("light.create", {
        "type": "AREA",
        "name": "RimLight",
        "location": [-5, 5, 3],
        "energy": 500.0,
        "color": [0.4, 0.6, 1.0]
    }, "Adding cool blue rim light...")
    time.sleep(0.2)

    # Create fill light (soft purple)
    send_command("light.create", {
        "type": "AREA",
        "name": "FillLight",
        "location": [0, -8, 2],
        "energy": 200.0,
        "color": [0.7, 0.5, 0.9]
    }, "Adding soft purple fill light...")
    time.sleep(0.3)


def create_ramp():
    """Create the starting ramp."""
    print("\n" + "="*60)
    print("PHASE 3: Building the Physics Playground")
    print("="*60)

    # Create ramp (rotated cube)
    send_command("object.create", {
        "type": "cube",
        "name": "Ramp",
        "location": [-6, 0, 3],
        "scale": [0.3, 1.5, 0.1]
    }, "Creating the starting ramp...")
    time.sleep(0.2)

    send_command("object.transform", {
        "name": "Ramp",
        "rotation": [0, 0.4, 0]  # Tilt for slope
    }, "Tilting ramp for the ball to roll...")
    time.sleep(0.2)

    # Coral material
    send_command("material.create", {
        "name": "CoralMat",
        "color": COLORS["coral"],
        "metallic": 0.3,
        "roughness": 0.4
    }, "Adding coral pink glossy material...")
    time.sleep(0.2)

    send_command("material.assign", {
        "object": "Ramp",
        "material": "CoralMat"
    }, "Applying to ramp...")
    time.sleep(0.3)


def create_bowling_ball():
    """Create the bowling ball."""
    send_command("object.create", {
        "type": "sphere",
        "name": "BowlingBall",
        "location": [-6.5, 0, 4],
        "scale": [0.4, 0.4, 0.4]
    }, "Creating bowling ball at top of ramp...")
    time.sleep(0.2)

    # Electric blue metallic material
    send_command("material.create", {
        "name": "BlueMat",
        "color": COLORS["blue"],
        "metallic": 0.9,
        "roughness": 0.1,
        "emission_strength": 0.3
    }, "Adding electric blue metallic material with glow...")
    time.sleep(0.2)

    send_command("material.assign", {
        "object": "BowlingBall",
        "material": "BlueMat"
    }, "Applying to bowling ball...")
    time.sleep(0.3)


def create_dominoes():
    """Create a curved line of dominoes."""
    print("\n   Creating domino chain...")

    # Create dominoes in a curved path
    num_dominoes = 20
    for i in range(num_dominoes):
        # Curved path from ramp end towards the right
        t = i / num_dominoes
        x = -4 + t * 6  # Move from -4 to 2
        y = -1.5 * (1 - (2*t - 1)**2)  # Parabolic curve

        # Interpolate color from yellow to red
        if t < 0.5:
            color = (
                1.0,
                0.9 - t * 1.2,
                0.2,
                1.0
            )
        else:
            color = (
                1.0,
                0.3 - (t - 0.5) * 0.2,
                0.2 - (t - 0.5) * 0.2,
                1.0
            )

        name = f"Domino_{i:02d}"

        # Calculate rotation to face along the curve
        import math
        if i < num_dominoes - 1:
            next_t = (i + 1) / num_dominoes
            next_x = -4 + next_t * 6
            next_y = -1.5 * (1 - (2*next_t - 1)**2)
            angle = math.atan2(next_y - y, next_x - x)
        else:
            angle = 0

        send_command("object.create", {
            "type": "cube",
            "name": name,
            "location": [x, y, 0.5],
            "scale": [0.08, 0.25, 0.5]
        }, f"Domino {i+1}/{num_dominoes}..." if i % 5 == 0 else None)

        send_command("object.transform", {
            "name": name,
            "rotation": [0, 0, angle + 1.5708]  # Rotate to face path
        }, None)

        # Create and assign gradient material
        mat_name = f"DominoMat_{i:02d}"
        send_command("material.create", {
            "name": mat_name,
            "color": color,
            "metallic": 0.1,
            "roughness": 0.3,
            "emission_strength": 0.2
        }, None)

        send_command("material.assign", {
            "object": name,
            "material": mat_name
        }, None)

        time.sleep(0.05)

    print(f"   → Created {num_dominoes} gradient dominoes ✓")
    time.sleep(0.3)


def create_seesaw():
    """Create the seesaw/lever."""
    send_command("object.create", {
        "type": "cube",
        "name": "Seesaw",
        "location": [3, 0, 0.3],
        "scale": [1.5, 0.3, 0.05]
    }, "Creating seesaw lever...")
    time.sleep(0.2)

    # Pivot point (small cylinder)
    send_command("object.create", {
        "type": "cylinder",
        "name": "Pivot",
        "location": [3, 0, 0.15],
        "scale": [0.1, 0.1, 0.1]
    }, "Adding pivot point...")
    time.sleep(0.2)

    # Mint material
    send_command("material.create", {
        "name": "MintMat",
        "color": COLORS["mint"],
        "metallic": 0.5,
        "roughness": 0.2
    }, "Adding mint green glossy material...")
    time.sleep(0.2)

    send_command("material.assign", {
        "object": "Seesaw",
        "material": "MintMat"
    }, "Applying to seesaw...")
    time.sleep(0.2)

    send_command("material.assign", {
        "object": "Pivot",
        "material": "MintMat"
    }, None)
    time.sleep(0.3)


def create_fluid_pool():
    """Create the fluid pool."""
    # Pool container
    send_command("object.create", {
        "type": "cube",
        "name": "Pool",
        "location": [5, 0, -0.5],
        "scale": [1.5, 1.5, 0.5]
    }, "Creating fluid pool container...")
    time.sleep(0.2)

    # Cyan material
    send_command("material.create", {
        "name": "CyanMat",
        "color": COLORS["cyan"],
        "metallic": 0.0,
        "roughness": 0.0,
        "transmission": 0.9
    }, "Adding cyan glass-like material...")
    time.sleep(0.2)

    send_command("material.assign", {
        "object": "Pool",
        "material": "CyanMat"
    }, "Applying to pool...")
    time.sleep(0.3)


def create_trigger_ball():
    """Create a small ball that will fall into the pool."""
    send_command("object.create", {
        "type": "sphere",
        "name": "TriggerBall",
        "location": [4.2, 0, 0.6],
        "scale": [0.2, 0.2, 0.2]
    }, "Creating trigger ball on seesaw...")
    time.sleep(0.2)

    send_command("material.create", {
        "name": "GoldMat",
        "color": COLORS["gold"],
        "metallic": 1.0,
        "roughness": 0.1,
        "emission_strength": 0.5
    }, "Adding shiny gold material...")
    time.sleep(0.2)

    send_command("material.assign", {
        "object": "TriggerBall",
        "material": "GoldMat"
    }, "Applying to trigger ball...")
    time.sleep(0.3)


def create_ground():
    """Create a ground plane."""
    send_command("object.create", {
        "type": "plane",
        "name": "Ground",
        "location": [0, 0, 0],
        "scale": [15, 15, 1]
    }, "Creating ground plane...")
    time.sleep(0.2)

    send_command("material.create", {
        "name": "GroundMat",
        "color": (0.15, 0.1, 0.2, 1.0),  # Dark purple
        "metallic": 0.0,
        "roughness": 0.8
    }, "Adding dark ground material...")
    time.sleep(0.2)

    send_command("material.assign", {
        "object": "Ground",
        "material": "GroundMat"
    }, "Applying to ground...")
    time.sleep(0.3)


def setup_physics():
    """Set up physics for all objects."""
    print("\n" + "="*60)
    print("PHASE 4: Enabling Physics")
    print("="*60)

    # Ground - passive rigid body
    send_command("physics.rigid_body.add", {
        "object": "Ground",
        "type": "PASSIVE"
    }, "Making ground a passive rigid body...")
    time.sleep(0.2)

    # Ramp - passive
    send_command("physics.rigid_body.add", {
        "object": "Ramp",
        "type": "PASSIVE"
    }, "Making ramp passive...")
    time.sleep(0.2)

    # Bowling ball - active
    send_command("physics.rigid_body.add", {
        "object": "BowlingBall",
        "type": "ACTIVE",
        "mass": 5.0
    }, "Making bowling ball active (mass: 5kg)...")
    time.sleep(0.2)

    # Dominoes - active
    print("\n   Enabling physics on dominoes...")
    for i in range(20):
        send_command("physics.rigid_body.add", {
            "object": f"Domino_{i:02d}",
            "type": "ACTIVE",
            "mass": 0.5
        }, None)
        time.sleep(0.02)
    print("   → All 20 dominoes are now physics-enabled ✓")
    time.sleep(0.2)

    # Seesaw - active (will tip when hit)
    send_command("physics.rigid_body.add", {
        "object": "Seesaw",
        "type": "ACTIVE",
        "mass": 1.0
    }, "Making seesaw active...")
    time.sleep(0.2)

    # Pivot - passive
    send_command("physics.rigid_body.add", {
        "object": "Pivot",
        "type": "PASSIVE"
    }, "Making pivot passive...")
    time.sleep(0.2)

    # Trigger ball - active
    send_command("physics.rigid_body.add", {
        "object": "TriggerBall",
        "type": "ACTIVE",
        "mass": 0.3
    }, "Making trigger ball active...")
    time.sleep(0.2)

    # Pool - passive
    send_command("physics.rigid_body.add", {
        "object": "Pool",
        "type": "PASSIVE"
    }, "Making pool passive...")
    time.sleep(0.3)


def set_camera():
    """Position camera for good view."""
    send_command("object.transform", {
        "name": "Camera",
        "location": [0, -12, 6],
        "rotation": [1.2, 0, 0]
    }, "Positioning camera for the best view...")
    time.sleep(0.3)


def run_simulation():
    """Bake and run the physics simulation."""
    print("\n" + "="*60)
    print("PHASE 5: Let's Simulate!")
    print("="*60)

    send_command("physics.simulate", {
        "frame_start": 1,
        "frame_end": 250
    }, "Running physics simulation (250 frames)...")
    time.sleep(0.5)

    print("\n🎬 Simulation complete! The physics playground is ready.")


def take_screenshot():
    """Take a viewport screenshot."""
    send_command("viewport.screenshot", {
        "output_path": "/tmp/physics_playground.png"
    }, "Capturing viewport screenshot...")
    print("\n📸 Screenshot saved to /tmp/physics_playground.png")


def main():
    print("""
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║     🎮 FGP BLENDER - PHYSICS PLAYGROUND SHOWCASE 🎮          ║
║                                                              ║
║     Live Collaborative Scene Building Demo                   ║
║     Watch as we build a Rube Goldberg machine!               ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
    """)

    # Check daemon connection
    print("Connecting to FGP Blender daemon...", end=" ")
    try:
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        sock.connect(SOCKET_PATH)
        sock.close()
        print("✓ Connected!\n")
    except Exception as e:
        print(f"✗ Failed: {e}")
        print("Make sure Blender is running with run_daemon_gui_v2.py")
        sys.exit(1)

    time.sleep(1)

    # Build the scene step by step
    clear_scene()
    time.sleep(0.5)

    create_ground()
    time.sleep(0.3)

    create_environment()
    time.sleep(0.3)

    create_ramp()
    time.sleep(0.3)

    create_bowling_ball()
    time.sleep(0.3)

    create_dominoes()
    time.sleep(0.3)

    create_seesaw()
    time.sleep(0.3)

    create_trigger_ball()
    time.sleep(0.3)

    create_fluid_pool()
    time.sleep(0.3)

    set_camera()
    time.sleep(0.3)

    setup_physics()
    time.sleep(0.5)

    # Take pre-simulation screenshot
    take_screenshot()

    # Run simulation
    run_simulation()

    print("""
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║     ✅ SHOWCASE COMPLETE!                                    ║
║                                                              ║
║     The Blender GUI now shows a stylized physics             ║
║     playground ready to play. Press SPACE in Blender         ║
║     to watch the simulation!                                 ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
    """)


if __name__ == "__main__":
    main()
