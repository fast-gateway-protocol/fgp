#!/usr/bin/env python3
"""
Benchmark script for FGP Blender daemon.

Measures latency for various operations.
"""

import json
import socket
import time
import uuid
from pathlib import Path
from typing import Any


SOCKET_PATH = Path.home() / ".fgp" / "services" / "blender" / "daemon.sock"


def send_request(method: str, params: dict = None) -> tuple[dict, float]:
    """Send request and return result with latency in ms."""
    sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    sock.settimeout(30.0)
    sock.connect(str(SOCKET_PATH))

    try:
        request = {
            "id": str(uuid.uuid4()),
            "v": 1,
            "method": method,
            "params": params or {},
        }

        start = time.perf_counter()
        sock.sendall((json.dumps(request) + "\n").encode())

        data = b""
        while True:
            chunk = sock.recv(4096)
            if not chunk:
                break
            data += chunk
            if b"\n" in data:
                break

        end = time.perf_counter()
        latency_ms = (end - start) * 1000

        response = json.loads(data.decode().strip())
        return response.get("result", response), latency_ms

    finally:
        sock.close()


def benchmark_operation(name: str, method: str, params: dict = None, iterations: int = 10) -> dict:
    """Benchmark a single operation."""
    latencies = []

    for i in range(iterations):
        try:
            _, latency = send_request(method, params)
            latencies.append(latency)
        except Exception as e:
            print(f"  Error on iteration {i}: {e}")

    if not latencies:
        return {"name": name, "error": "All iterations failed"}

    avg = sum(latencies) / len(latencies)
    min_lat = min(latencies)
    max_lat = max(latencies)

    return {
        "name": name,
        "iterations": len(latencies),
        "avg_ms": round(avg, 2),
        "min_ms": round(min_lat, 2),
        "max_ms": round(max_lat, 2),
    }


def main():
    print("=" * 60)
    print("FGP Blender Daemon Benchmark")
    print("=" * 60)
    print()

    # Check daemon is running
    try:
        result, latency = send_request("health")
        print(f"Daemon status: {result.get('status', 'unknown')}")
        print(f"Health check latency: {latency:.2f}ms")
        print()
    except Exception as e:
        print(f"ERROR: Daemon not running or not accessible: {e}")
        return

    benchmarks = []

    # Benchmark: Health check
    print("Running benchmarks (10 iterations each)...")
    print()

    benchmarks.append(benchmark_operation(
        "health",
        "health",
    ))

    # Benchmark: Scene info
    benchmarks.append(benchmark_operation(
        "scene.info",
        "blender.scene.info",
    ))

    # Benchmark: Object list
    benchmarks.append(benchmark_operation(
        "object.list",
        "blender.object.list",
    ))

    # Benchmark: Object create
    benchmarks.append(benchmark_operation(
        "object.create (CUBE)",
        "blender.object.create",
        {"type": "CUBE", "name": f"BenchCube"},
    ))

    # Benchmark: Object transform
    # First get an object name
    result, _ = send_request("blender.object.list")
    if result.get("objects"):
        obj_name = result["objects"][0]["name"]
        benchmarks.append(benchmark_operation(
            "object.transform",
            "blender.object.transform",
            {"name": obj_name, "location": [1, 2, 3]},
        ))

    # Benchmark: Material create
    benchmarks.append(benchmark_operation(
        "material.create",
        "blender.material.create",
        {"name": "BenchMat", "color": [0.5, 0.5, 0.5]},
    ))

    # Benchmark: Material list
    benchmarks.append(benchmark_operation(
        "material.list",
        "blender.material.list",
    ))

    # Benchmark: Session list
    benchmarks.append(benchmark_operation(
        "session.list",
        "blender.session.list",
    ))

    # Print results
    print()
    print("=" * 60)
    print("Results")
    print("=" * 60)
    print()
    print(f"{'Operation':<25} {'Avg (ms)':<12} {'Min (ms)':<12} {'Max (ms)':<12}")
    print("-" * 60)

    for b in benchmarks:
        if "error" in b:
            print(f"{b['name']:<25} ERROR: {b['error']}")
        else:
            print(f"{b['name']:<25} {b['avg_ms']:<12.2f} {b['min_ms']:<12.2f} {b['max_ms']:<12.2f}")

    print()
    print("=" * 60)
    print("Summary")
    print("=" * 60)

    # Calculate overall average
    successful = [b for b in benchmarks if "avg_ms" in b]
    if successful:
        overall_avg = sum(b["avg_ms"] for b in successful) / len(successful)
        print(f"Average latency across all operations: {overall_avg:.2f}ms")

    # Compare to expected MCP latency
    print()
    print("Comparison to MCP implementations:")
    print("  - MCP cold start: ~4-5 seconds")
    print("  - MCP warm operation: ~200-300ms (estimated)")
    print(f"  - FGP warm operation: ~{overall_avg:.1f}ms")
    print(f"  - Speedup: ~{300/overall_avg:.0f}x faster (vs estimated MCP warm)")
    print()

    # Save results to JSON
    results = {
        "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
        "benchmarks": benchmarks,
        "overall_avg_ms": round(overall_avg, 2) if successful else None,
    }

    output_path = Path("/tmp/fgp_blender_benchmark.json")
    with open(output_path, "w") as f:
        json.dump(results, f, indent=2)
    print(f"Results saved to: {output_path}")


if __name__ == "__main__":
    main()
