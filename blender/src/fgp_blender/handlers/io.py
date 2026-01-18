"""
I/O Handler for FGP Blender Daemon.

Handles import/export and asset library operations.

CHANGELOG:
01/17/2026 - Initial implementation (Claude)
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

from fgp_daemon import MethodInfo, ParamInfo

from ..session import SessionManager


class IOHandler:
    """Handler for import/export and asset operations."""

    def __init__(self, bpy: Any, sessions: SessionManager) -> None:
        """Initialize with Blender module and session manager."""
        self.bpy = bpy
        self.sessions = sessions

    def dispatch(self, category: str, action: str, params: dict[str, Any]) -> Any:
        """Dispatch I/O methods."""
        if category == "import":
            return self._import(action, params)
        elif category == "export":
            return self._export(action, params)
        elif category == "assets":
            return self._assets(action, params)
        else:
            raise ValueError(f"Unknown I/O category: {category}")

    def _import(self, format_type: str, params: dict[str, Any]) -> dict[str, Any]:
        """Import a file."""
        if not self.bpy:
            return {"imported": True, "mock": True}

        bpy = self.bpy
        filepath = params.get("filepath")
        if not filepath:
            raise ValueError("filepath is required")

        path = Path(filepath).expanduser()
        if not path.exists():
            raise FileNotFoundError(f"File not found: {filepath}")

        # Auto-detect format if not specified
        if not format_type or format_type == "auto":
            ext = path.suffix.lower()
            format_map = {
                ".fbx": "fbx",
                ".obj": "obj",
                ".gltf": "gltf",
                ".glb": "gltf",
                ".usd": "usd",
                ".usda": "usd",
                ".usdc": "usd",
                ".stl": "stl",
                ".ply": "ply",
                ".abc": "alembic",
                ".svg": "svg",
            }
            format_type = format_map.get(ext)
            if not format_type:
                raise ValueError(f"Unknown file format: {ext}")

        format_type = format_type.lower()

        # Import based on format
        if format_type == "fbx":
            bpy.ops.import_scene.fbx(filepath=str(path))
        elif format_type == "obj":
            bpy.ops.wm.obj_import(filepath=str(path))
        elif format_type in ("gltf", "glb"):
            bpy.ops.import_scene.gltf(filepath=str(path))
        elif format_type == "usd":
            bpy.ops.wm.usd_import(filepath=str(path))
        elif format_type == "stl":
            bpy.ops.wm.stl_import(filepath=str(path))
        elif format_type == "ply":
            bpy.ops.wm.ply_import(filepath=str(path))
        elif format_type == "alembic":
            bpy.ops.wm.alembic_import(filepath=str(path))
        elif format_type == "svg":
            bpy.ops.import_curve.svg(filepath=str(path))
        else:
            raise ValueError(f"Unsupported import format: {format_type}")

        # Get imported objects
        imported = [obj.name for obj in bpy.context.selected_objects]

        self.sessions.update_modified()

        return {
            "imported": True,
            "filepath": str(path),
            "format": format_type,
            "objects": imported,
            "count": len(imported),
        }

    def _export(self, format_type: str, params: dict[str, Any]) -> dict[str, Any]:
        """Export to file."""
        if not self.bpy:
            return {"exported": True, "mock": True}

        bpy = self.bpy
        filepath = params.get("filepath")
        selected_only = params.get("selected_only", False)

        if not filepath:
            raise ValueError("filepath is required")

        path = Path(filepath).expanduser()
        path.parent.mkdir(parents=True, exist_ok=True)

        # Auto-detect format from extension if not specified
        if not format_type or format_type == "auto":
            ext = path.suffix.lower()
            format_map = {
                ".fbx": "fbx",
                ".obj": "obj",
                ".gltf": "gltf",
                ".glb": "glb",
                ".usd": "usd",
                ".usda": "usda",
                ".usdc": "usdc",
                ".stl": "stl",
                ".ply": "ply",
                ".abc": "alembic",
            }
            format_type = format_map.get(ext)
            if not format_type:
                raise ValueError(f"Unknown file format: {ext}")

        format_type = format_type.lower()

        # Export based on format
        if format_type == "fbx":
            bpy.ops.export_scene.fbx(
                filepath=str(path),
                use_selection=selected_only,
            )
        elif format_type == "obj":
            bpy.ops.wm.obj_export(
                filepath=str(path),
                export_selected_objects=selected_only,
            )
        elif format_type == "gltf":
            bpy.ops.export_scene.gltf(
                filepath=str(path),
                export_format="GLTF_SEPARATE",
                use_selection=selected_only,
            )
        elif format_type == "glb":
            bpy.ops.export_scene.gltf(
                filepath=str(path),
                export_format="GLB",
                use_selection=selected_only,
            )
        elif format_type in ("usd", "usda", "usdc"):
            bpy.ops.wm.usd_export(
                filepath=str(path),
                selected_objects_only=selected_only,
            )
        elif format_type == "stl":
            bpy.ops.wm.stl_export(
                filepath=str(path),
                export_selected_objects=selected_only,
            )
        elif format_type == "ply":
            bpy.ops.wm.ply_export(
                filepath=str(path),
                export_selected_objects=selected_only,
            )
        elif format_type == "alembic":
            bpy.ops.wm.alembic_export(
                filepath=str(path),
                selected=selected_only,
            )
        else:
            raise ValueError(f"Unsupported export format: {format_type}")

        return {
            "exported": True,
            "filepath": str(path),
            "format": format_type,
        }

    def _assets(self, action: str, params: dict[str, Any]) -> dict[str, Any]:
        """Handle asset library operations."""
        # Parse nested action like assets.polyhaven.search
        parts = action.split(".")
        if len(parts) > 1:
            source = parts[0]
            sub_action = parts[1]
        else:
            source = action
            sub_action = None

        if source == "polyhaven":
            return self._polyhaven(sub_action, params)
        elif source == "sketchfab":
            return self._sketchfab(sub_action, params)
        else:
            raise ValueError(f"Unknown asset source: {source}")

    def _polyhaven(self, action: str, params: dict[str, Any]) -> dict[str, Any]:
        """Handle Poly Haven asset operations."""
        try:
            import requests
        except ImportError:
            raise RuntimeError("requests package required: pip install requests")

        base_url = "https://api.polyhaven.com"

        if action == "search":
            query = params.get("query", "")
            category = params.get("category", "all")  # hdris, textures, models
            limit = params.get("limit", 20)

            # Get asset list
            if category == "all":
                response = requests.get(f"{base_url}/assets")
            else:
                response = requests.get(f"{base_url}/assets", params={"t": category})

            response.raise_for_status()
            assets = response.json()

            # Filter by query
            if query:
                query_lower = query.lower()
                assets = {
                    k: v for k, v in assets.items()
                    if query_lower in k.lower() or query_lower in v.get("name", "").lower()
                }

            # Limit results
            results = []
            for asset_id, data in list(assets.items())[:limit]:
                results.append({
                    "id": asset_id,
                    "name": data.get("name", asset_id),
                    "categories": data.get("categories", []),
                    "tags": data.get("tags", []),
                })

            return {"assets": results, "count": len(results)}

        elif action == "download":
            asset_id = params.get("id")
            resolution = params.get("resolution", "2k")
            asset_type = params.get("type", "hdri")  # hdri, texture, model

            if not asset_id:
                raise ValueError("id is required")

            # Get download URL
            response = requests.get(f"{base_url}/files/{asset_id}")
            response.raise_for_status()
            files = response.json()

            # Find appropriate file
            download_url = None
            if asset_type == "hdri":
                hdri_files = files.get("hdri", {})
                if resolution in hdri_files:
                    download_url = hdri_files[resolution].get("hdr", {}).get("url")
            elif asset_type == "texture":
                tex_files = files.get("Diffuse", {})
                if resolution in tex_files:
                    download_url = tex_files[resolution].get("jpg", {}).get("url")

            if not download_url:
                return {"downloaded": False, "error": "File not found for specified resolution"}

            # Download file
            save_dir = Path.home() / ".fgp" / "assets" / "polyhaven" / asset_type
            save_dir.mkdir(parents=True, exist_ok=True)
            save_path = save_dir / f"{asset_id}_{resolution}.{'hdr' if asset_type == 'hdri' else 'jpg'}"

            if not save_path.exists():
                response = requests.get(download_url)
                response.raise_for_status()
                with open(save_path, "wb") as f:
                    f.write(response.content)

            return {
                "downloaded": True,
                "id": asset_id,
                "path": str(save_path),
                "resolution": resolution,
            }

        else:
            raise ValueError(f"Unknown polyhaven action: {action}")

    def _sketchfab(self, action: str, params: dict[str, Any]) -> dict[str, Any]:
        """Handle Sketchfab asset operations."""
        try:
            import requests
        except ImportError:
            raise RuntimeError("requests package required: pip install requests")

        # Note: Sketchfab API requires authentication for downloads
        api_token = params.get("api_token")
        base_url = "https://api.sketchfab.com/v3"

        if action == "search":
            query = params.get("query", "")
            limit = params.get("limit", 20)
            downloadable = params.get("downloadable", True)

            response = requests.get(
                f"{base_url}/search",
                params={
                    "type": "models",
                    "q": query,
                    "downloadable": downloadable,
                    "count": limit,
                },
            )
            response.raise_for_status()
            data = response.json()

            results = []
            for model in data.get("results", []):
                results.append({
                    "uid": model["uid"],
                    "name": model["name"],
                    "description": model.get("description", "")[:100],
                    "thumbnail": model.get("thumbnails", {}).get("images", [{}])[0].get("url"),
                    "downloadable": model.get("isDownloadable", False),
                })

            return {"models": results, "count": len(results)}

        elif action == "import":
            if not api_token:
                raise ValueError("api_token required for Sketchfab downloads")

            model_uid = params.get("uid")
            if not model_uid:
                raise ValueError("uid is required")

            headers = {"Authorization": f"Token {api_token}"}

            # Request download
            response = requests.get(
                f"{base_url}/models/{model_uid}/download",
                headers=headers,
            )

            if response.status_code == 401:
                raise ValueError("Invalid API token or model not downloadable")

            response.raise_for_status()
            download_data = response.json()

            # Download the GLTF file
            gltf_url = download_data.get("gltf", {}).get("url")
            if not gltf_url:
                return {"imported": False, "error": "GLTF format not available"}

            # Download to temp location
            save_dir = Path.home() / ".fgp" / "assets" / "sketchfab"
            save_dir.mkdir(parents=True, exist_ok=True)
            save_path = save_dir / f"{model_uid}.glb"

            response = requests.get(gltf_url)
            response.raise_for_status()
            with open(save_path, "wb") as f:
                f.write(response.content)

            # Import into Blender
            if self.bpy:
                self.bpy.ops.import_scene.gltf(filepath=str(save_path))
                imported = [obj.name for obj in self.bpy.context.selected_objects]
                self.sessions.update_modified()
            else:
                imported = []

            return {
                "imported": True,
                "uid": model_uid,
                "path": str(save_path),
                "objects": imported,
            }

        else:
            raise ValueError(f"Unknown sketchfab action: {action}")

    def method_list(self) -> list[MethodInfo]:
        """Return available I/O methods."""
        return [
            MethodInfo("import", "Import file (auto-detect format)", [
                ParamInfo("filepath", "string", True),
            ]),
            MethodInfo("import.fbx", "Import FBX file", [
                ParamInfo("filepath", "string", True),
            ]),
            MethodInfo("import.obj", "Import OBJ file", [
                ParamInfo("filepath", "string", True),
            ]),
            MethodInfo("import.gltf", "Import GLTF/GLB file", [
                ParamInfo("filepath", "string", True),
            ]),
            MethodInfo("import.usd", "Import USD file", [
                ParamInfo("filepath", "string", True),
            ]),
            MethodInfo("import.stl", "Import STL file", [
                ParamInfo("filepath", "string", True),
            ]),
            MethodInfo("export", "Export scene (auto-detect format)", [
                ParamInfo("filepath", "string", True),
                ParamInfo("selected_only", "boolean", False, False),
            ]),
            MethodInfo("export.fbx", "Export to FBX", [
                ParamInfo("filepath", "string", True),
                ParamInfo("selected_only", "boolean", False, False),
            ]),
            MethodInfo("export.gltf", "Export to GLTF", [
                ParamInfo("filepath", "string", True),
                ParamInfo("selected_only", "boolean", False, False),
            ]),
            MethodInfo("export.glb", "Export to GLB", [
                ParamInfo("filepath", "string", True),
                ParamInfo("selected_only", "boolean", False, False),
            ]),
            MethodInfo("export.usd", "Export to USD", [
                ParamInfo("filepath", "string", True),
                ParamInfo("selected_only", "boolean", False, False),
            ]),
            MethodInfo("export.stl", "Export to STL", [
                ParamInfo("filepath", "string", True),
                ParamInfo("selected_only", "boolean", False, False),
            ]),
            MethodInfo("assets.polyhaven.search", "Search Poly Haven assets", [
                ParamInfo("query", "string", False, ""),
                ParamInfo("category", "string", False, "all"),
                ParamInfo("limit", "integer", False, 20),
            ]),
            MethodInfo("assets.polyhaven.download", "Download Poly Haven asset", [
                ParamInfo("id", "string", True),
                ParamInfo("resolution", "string", False, "2k"),
                ParamInfo("type", "string", False, "hdri"),
            ]),
            MethodInfo("assets.sketchfab.search", "Search Sketchfab models", [
                ParamInfo("query", "string", True),
                ParamInfo("limit", "integer", False, 20),
                ParamInfo("downloadable", "boolean", False, True),
            ]),
            MethodInfo("assets.sketchfab.import", "Import Sketchfab model", [
                ParamInfo("uid", "string", True),
                ParamInfo("api_token", "string", True),
            ]),
        ]
