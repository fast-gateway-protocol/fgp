"""
AI model generation handler for FGP Blender.

Integrates with external AI 3D generation services:
- Hyper3D Rodin (text/image to 3D)
- Hunyuan3D (Tencent's 3D generation)
- Future: Meshy, Tripo3D, etc.

CHANGELOG:
01/17/2026 - Initial implementation (Claude)
"""

from __future__ import annotations

import json
import logging
import os
import tempfile
import time
import urllib.request
import urllib.error
from pathlib import Path
from typing import Any, Optional
from dataclasses import dataclass, field
from enum import Enum

from fgp_daemon import MethodInfo, ParamInfo

logger = logging.getLogger(__name__)


class AIProvider(Enum):
    """Supported AI generation providers."""
    HYPER3D_RODIN = "hyper3d_rodin"
    HUNYUAN3D = "hunyuan3d"
    MESHY = "meshy"
    TRIPO3D = "tripo3d"


@dataclass
class GenerationJob:
    """Tracks an AI generation job."""
    id: str
    provider: AIProvider
    status: str  # pending, processing, completed, failed
    prompt: Optional[str] = None
    image_urls: Optional[list[str]] = None
    result_url: Optional[str] = None
    error: Optional[str] = None
    created_at: float = field(default_factory=time.time)
    completed_at: Optional[float] = None


class AIGenerationHandler:
    """Handler for AI 3D model generation."""

    def __init__(self, bpy: Any, sessions: Any, jobs: Any) -> None:
        self._bpy = bpy
        self._sessions = sessions
        self._jobs = jobs
        self._generation_jobs: dict[str, GenerationJob] = {}

        # API keys from environment
        self._hyper3d_key = os.environ.get("HYPER3D_API_KEY")
        self._hunyuan_key = os.environ.get("HUNYUAN_API_KEY")
        self._meshy_key = os.environ.get("MESHY_API_KEY")

    def dispatch(self, category: str, action: str, params: dict[str, Any]) -> Any:
        """Dispatch AI generation method calls."""
        method_name = f"{category}.{action}" if action else category

        handlers = {
            # Provider status
            "ai.status": self._status,
            "ai.providers": self._list_providers,

            # Hyper3D Rodin
            "ai.hyper3d.generate_text": self._hyper3d_generate_text,
            "ai.hyper3d.generate_image": self._hyper3d_generate_image,
            "ai.hyper3d.poll": self._hyper3d_poll,
            "ai.hyper3d.import": self._hyper3d_import,

            # Hunyuan3D
            "ai.hunyuan.generate": self._hunyuan_generate,
            "ai.hunyuan.poll": self._hunyuan_poll,
            "ai.hunyuan.import": self._hunyuan_import,

            # Meshy
            "ai.meshy.generate_text": self._meshy_generate_text,
            "ai.meshy.generate_image": self._meshy_generate_image,
            "ai.meshy.poll": self._meshy_poll,
            "ai.meshy.import": self._meshy_import,

            # Generic
            "ai.job.status": self._job_status,
            "ai.job.list": self._job_list,
        }

        handler = handlers.get(method_name)
        if handler is None:
            raise ValueError(f"Unknown AI method: {method_name}")

        return handler(params)

    # -------------------------------------------------------------------------
    # Status & Discovery
    # -------------------------------------------------------------------------

    def _status(self, params: dict[str, Any]) -> dict:
        """Check AI provider status."""
        return {
            "providers": {
                "hyper3d_rodin": {
                    "available": bool(self._hyper3d_key),
                    "configured": bool(self._hyper3d_key),
                    "env_var": "HYPER3D_API_KEY",
                },
                "hunyuan3d": {
                    "available": bool(self._hunyuan_key),
                    "configured": bool(self._hunyuan_key),
                    "env_var": "HUNYUAN_API_KEY",
                },
                "meshy": {
                    "available": bool(self._meshy_key),
                    "configured": bool(self._meshy_key),
                    "env_var": "MESHY_API_KEY",
                },
            }
        }

    def _list_providers(self, params: dict[str, Any]) -> dict:
        """List available AI providers."""
        return {
            "providers": [
                {
                    "id": "hyper3d_rodin",
                    "name": "Hyper3D Rodin",
                    "description": "High-quality text/image to 3D generation",
                    "capabilities": ["text_to_3d", "image_to_3d"],
                    "url": "https://hyper3d.ai/rodin",
                },
                {
                    "id": "hunyuan3d",
                    "name": "Hunyuan3D",
                    "description": "Tencent's 3D generation model",
                    "capabilities": ["text_to_3d", "image_to_3d"],
                    "url": "https://hunyuan3d.tencent.com",
                },
                {
                    "id": "meshy",
                    "name": "Meshy",
                    "description": "Fast text/image to 3D with textures",
                    "capabilities": ["text_to_3d", "image_to_3d", "texturing"],
                    "url": "https://meshy.ai",
                },
            ]
        }

    # -------------------------------------------------------------------------
    # Hyper3D Rodin
    # -------------------------------------------------------------------------

    def _hyper3d_generate_text(self, params: dict[str, Any]) -> dict:
        """Generate 3D model from text prompt using Hyper3D Rodin."""
        if not self._hyper3d_key:
            raise ValueError("HYPER3D_API_KEY not configured")

        prompt = params.get("prompt")
        if not prompt:
            raise ValueError("prompt is required")

        # Bounding box condition (optional)
        bbox = params.get("bbox_condition")

        # API call
        url = "https://hyperhuman.deemos.com/api/v2/rodin"
        headers = {
            "Authorization": f"Bearer {self._hyper3d_key}",
            "Content-Type": "application/json",
        }
        body = {
            "prompt": prompt,
        }
        if bbox:
            body["bbox_condition"] = bbox

        try:
            req = urllib.request.Request(
                url,
                data=json.dumps(body).encode(),
                headers=headers,
                method="POST"
            )
            with urllib.request.urlopen(req, timeout=30) as resp:
                result = json.loads(resp.read().decode())

            job_id = result.get("subscription_key") or result.get("request_id") or result.get("uuid")

            # Track job
            job = GenerationJob(
                id=job_id,
                provider=AIProvider.HYPER3D_RODIN,
                status="processing",
                prompt=prompt,
            )
            self._generation_jobs[job_id] = job

            return {
                "job_id": job_id,
                "provider": "hyper3d_rodin",
                "status": "processing",
                "prompt": prompt,
            }

        except urllib.error.HTTPError as e:
            raise RuntimeError(f"Hyper3D API error: {e.code} {e.reason}")

    def _hyper3d_generate_image(self, params: dict[str, Any]) -> dict:
        """Generate 3D model from images using Hyper3D Rodin."""
        if not self._hyper3d_key:
            raise ValueError("HYPER3D_API_KEY not configured")

        image_urls = params.get("image_urls", [])
        image_paths = params.get("image_paths", [])

        if not image_urls and not image_paths:
            raise ValueError("image_urls or image_paths required")

        # For local files, we'd need to upload them first
        # For now, support URLs
        url = "https://hyperhuman.deemos.com/api/v2/rodin"
        headers = {
            "Authorization": f"Bearer {self._hyper3d_key}",
            "Content-Type": "application/json",
        }
        body = {
            "images": image_urls,
        }

        bbox = params.get("bbox_condition")
        if bbox:
            body["bbox_condition"] = bbox

        try:
            req = urllib.request.Request(
                url,
                data=json.dumps(body).encode(),
                headers=headers,
                method="POST"
            )
            with urllib.request.urlopen(req, timeout=30) as resp:
                result = json.loads(resp.read().decode())

            job_id = result.get("subscription_key") or result.get("request_id") or result.get("uuid")

            job = GenerationJob(
                id=job_id,
                provider=AIProvider.HYPER3D_RODIN,
                status="processing",
                image_urls=image_urls,
            )
            self._generation_jobs[job_id] = job

            return {
                "job_id": job_id,
                "provider": "hyper3d_rodin",
                "status": "processing",
            }

        except urllib.error.HTTPError as e:
            raise RuntimeError(f"Hyper3D API error: {e.code} {e.reason}")

    def _hyper3d_poll(self, params: dict[str, Any]) -> dict:
        """Poll Hyper3D Rodin job status."""
        if not self._hyper3d_key:
            raise ValueError("HYPER3D_API_KEY not configured")

        job_id = params.get("job_id")
        if not job_id:
            raise ValueError("job_id is required")

        url = f"https://hyperhuman.deemos.com/api/v2/status/{job_id}"
        headers = {
            "Authorization": f"Bearer {self._hyper3d_key}",
        }

        try:
            req = urllib.request.Request(url, headers=headers)
            with urllib.request.urlopen(req, timeout=30) as resp:
                result = json.loads(resp.read().decode())

            status = result.get("status", "unknown")
            download_url = result.get("download_url") or result.get("model_url")

            # Update tracked job
            if job_id in self._generation_jobs:
                job = self._generation_jobs[job_id]
                job.status = status
                if download_url:
                    job.result_url = download_url
                    job.completed_at = time.time()

            return {
                "job_id": job_id,
                "status": status,
                "progress": result.get("progress"),
                "download_url": download_url,
            }

        except urllib.error.HTTPError as e:
            raise RuntimeError(f"Hyper3D API error: {e.code} {e.reason}")

    def _hyper3d_import(self, params: dict[str, Any]) -> dict:
        """Import generated Hyper3D model into Blender."""
        job_id = params.get("job_id")
        url = params.get("url")
        name = params.get("name", "Rodin_Model")

        if not job_id and not url:
            raise ValueError("job_id or url required")

        # Get URL from job if not provided
        if not url and job_id in self._generation_jobs:
            job = self._generation_jobs[job_id]
            url = job.result_url

        if not url:
            raise ValueError("No download URL available - check job status")

        return self._import_from_url(url, name, "glb")

    # -------------------------------------------------------------------------
    # Hunyuan3D
    # -------------------------------------------------------------------------

    def _hunyuan_generate(self, params: dict[str, Any]) -> dict:
        """Generate 3D model using Hunyuan3D."""
        if not self._hunyuan_key:
            raise ValueError("HUNYUAN_API_KEY not configured")

        prompt = params.get("prompt")
        image_url = params.get("image_url")

        if not prompt and not image_url:
            raise ValueError("prompt or image_url required")

        url = "https://api.hunyuan3d.tencent.com/v1/generate"
        headers = {
            "Authorization": f"Bearer {self._hunyuan_key}",
            "Content-Type": "application/json",
        }
        body = {}
        if prompt:
            body["prompt"] = prompt
        if image_url:
            body["image_url"] = image_url

        try:
            req = urllib.request.Request(
                url,
                data=json.dumps(body).encode(),
                headers=headers,
                method="POST"
            )
            with urllib.request.urlopen(req, timeout=30) as resp:
                result = json.loads(resp.read().decode())

            job_id = result.get("job_id") or result.get("task_id")

            job = GenerationJob(
                id=job_id,
                provider=AIProvider.HUNYUAN3D,
                status="processing",
                prompt=prompt,
                image_urls=[image_url] if image_url else None,
            )
            self._generation_jobs[job_id] = job

            return {
                "job_id": job_id,
                "provider": "hunyuan3d",
                "status": "processing",
            }

        except urllib.error.HTTPError as e:
            raise RuntimeError(f"Hunyuan3D API error: {e.code} {e.reason}")

    def _hunyuan_poll(self, params: dict[str, Any]) -> dict:
        """Poll Hunyuan3D job status."""
        if not self._hunyuan_key:
            raise ValueError("HUNYUAN_API_KEY not configured")

        job_id = params.get("job_id")
        if not job_id:
            raise ValueError("job_id is required")

        url = f"https://api.hunyuan3d.tencent.com/v1/status/{job_id}"
        headers = {
            "Authorization": f"Bearer {self._hunyuan_key}",
        }

        try:
            req = urllib.request.Request(url, headers=headers)
            with urllib.request.urlopen(req, timeout=30) as resp:
                result = json.loads(resp.read().decode())

            status = result.get("status", "unknown")
            download_url = result.get("download_url") or result.get("zip_url")

            if job_id in self._generation_jobs:
                job = self._generation_jobs[job_id]
                job.status = status
                if download_url:
                    job.result_url = download_url
                    job.completed_at = time.time()

            return {
                "job_id": job_id,
                "status": status,
                "download_url": download_url,
            }

        except urllib.error.HTTPError as e:
            raise RuntimeError(f"Hunyuan3D API error: {e.code} {e.reason}")

    def _hunyuan_import(self, params: dict[str, Any]) -> dict:
        """Import generated Hunyuan3D model."""
        job_id = params.get("job_id")
        url = params.get("url")
        name = params.get("name", "Hunyuan_Model")

        if not url and job_id in self._generation_jobs:
            url = self._generation_jobs[job_id].result_url

        if not url:
            raise ValueError("No download URL available")

        # Hunyuan typically returns a ZIP with GLB inside
        return self._import_from_url(url, name, "zip")

    # -------------------------------------------------------------------------
    # Meshy
    # -------------------------------------------------------------------------

    def _meshy_generate_text(self, params: dict[str, Any]) -> dict:
        """Generate 3D model from text using Meshy."""
        if not self._meshy_key:
            raise ValueError("MESHY_API_KEY not configured")

        prompt = params.get("prompt")
        if not prompt:
            raise ValueError("prompt is required")

        art_style = params.get("art_style", "realistic")  # realistic, cartoon, low-poly, sculpture
        negative_prompt = params.get("negative_prompt")

        url = "https://api.meshy.ai/v2/text-to-3d"
        headers = {
            "Authorization": f"Bearer {self._meshy_key}",
            "Content-Type": "application/json",
        }
        body = {
            "mode": "preview",  # preview or refine
            "prompt": prompt,
            "art_style": art_style,
        }
        if negative_prompt:
            body["negative_prompt"] = negative_prompt

        try:
            req = urllib.request.Request(
                url,
                data=json.dumps(body).encode(),
                headers=headers,
                method="POST"
            )
            with urllib.request.urlopen(req, timeout=30) as resp:
                result = json.loads(resp.read().decode())

            job_id = result.get("result")

            job = GenerationJob(
                id=job_id,
                provider=AIProvider.MESHY,
                status="processing",
                prompt=prompt,
            )
            self._generation_jobs[job_id] = job

            return {
                "job_id": job_id,
                "provider": "meshy",
                "status": "processing",
            }

        except urllib.error.HTTPError as e:
            raise RuntimeError(f"Meshy API error: {e.code} {e.reason}")

    def _meshy_generate_image(self, params: dict[str, Any]) -> dict:
        """Generate 3D model from image using Meshy."""
        if not self._meshy_key:
            raise ValueError("MESHY_API_KEY not configured")

        image_url = params.get("image_url")
        if not image_url:
            raise ValueError("image_url is required")

        url = "https://api.meshy.ai/v1/image-to-3d"
        headers = {
            "Authorization": f"Bearer {self._meshy_key}",
            "Content-Type": "application/json",
        }
        body = {
            "image_url": image_url,
        }

        try:
            req = urllib.request.Request(
                url,
                data=json.dumps(body).encode(),
                headers=headers,
                method="POST"
            )
            with urllib.request.urlopen(req, timeout=30) as resp:
                result = json.loads(resp.read().decode())

            job_id = result.get("result")

            job = GenerationJob(
                id=job_id,
                provider=AIProvider.MESHY,
                status="processing",
                image_urls=[image_url],
            )
            self._generation_jobs[job_id] = job

            return {
                "job_id": job_id,
                "provider": "meshy",
                "status": "processing",
            }

        except urllib.error.HTTPError as e:
            raise RuntimeError(f"Meshy API error: {e.code} {e.reason}")

    def _meshy_poll(self, params: dict[str, Any]) -> dict:
        """Poll Meshy job status."""
        if not self._meshy_key:
            raise ValueError("MESHY_API_KEY not configured")

        job_id = params.get("job_id")
        if not job_id:
            raise ValueError("job_id is required")

        url = f"https://api.meshy.ai/v2/text-to-3d/{job_id}"
        headers = {
            "Authorization": f"Bearer {self._meshy_key}",
        }

        try:
            req = urllib.request.Request(url, headers=headers)
            with urllib.request.urlopen(req, timeout=30) as resp:
                result = json.loads(resp.read().decode())

            status = result.get("status", "unknown")
            model_urls = result.get("model_urls", {})
            glb_url = model_urls.get("glb")

            if job_id in self._generation_jobs:
                job = self._generation_jobs[job_id]
                job.status = status
                if glb_url:
                    job.result_url = glb_url
                    job.completed_at = time.time()

            return {
                "job_id": job_id,
                "status": status,
                "progress": result.get("progress", 0),
                "model_urls": model_urls,
                "thumbnail_url": result.get("thumbnail_url"),
            }

        except urllib.error.HTTPError as e:
            raise RuntimeError(f"Meshy API error: {e.code} {e.reason}")

    def _meshy_import(self, params: dict[str, Any]) -> dict:
        """Import generated Meshy model."""
        job_id = params.get("job_id")
        url = params.get("url")
        name = params.get("name", "Meshy_Model")

        if not url and job_id in self._generation_jobs:
            url = self._generation_jobs[job_id].result_url

        if not url:
            raise ValueError("No download URL available")

        return self._import_from_url(url, name, "glb")

    # -------------------------------------------------------------------------
    # Job Management
    # -------------------------------------------------------------------------

    def _job_status(self, params: dict[str, Any]) -> dict:
        """Get status of an AI generation job."""
        job_id = params.get("job_id")
        if not job_id:
            raise ValueError("job_id is required")

        job = self._generation_jobs.get(job_id)
        if not job:
            return {"job_id": job_id, "status": "unknown"}

        return {
            "job_id": job.id,
            "provider": job.provider.value,
            "status": job.status,
            "prompt": job.prompt,
            "result_url": job.result_url,
            "created_at": job.created_at,
            "completed_at": job.completed_at,
        }

    def _job_list(self, params: dict[str, Any]) -> dict:
        """List AI generation jobs."""
        provider = params.get("provider")
        status = params.get("status")
        limit = params.get("limit", 50)

        jobs = list(self._generation_jobs.values())

        if provider:
            jobs = [j for j in jobs if j.provider.value == provider]
        if status:
            jobs = [j for j in jobs if j.status == status]

        jobs = sorted(jobs, key=lambda j: j.created_at, reverse=True)[:limit]

        return {
            "jobs": [
                {
                    "job_id": j.id,
                    "provider": j.provider.value,
                    "status": j.status,
                    "prompt": j.prompt,
                    "created_at": j.created_at,
                }
                for j in jobs
            ]
        }

    # -------------------------------------------------------------------------
    # Helpers
    # -------------------------------------------------------------------------

    def _import_from_url(self, url: str, name: str, format: str) -> dict:
        """Download and import a 3D model from URL."""
        if not self._bpy:
            return {"imported": True, "mock": True, "name": name}

        bpy = self._bpy

        # Download to temp file
        with tempfile.NamedTemporaryFile(suffix=f".{format}", delete=False) as tmp:
            tmp_path = tmp.name

        try:
            urllib.request.urlretrieve(url, tmp_path)

            # Import based on format
            if format == "glb" or format == "gltf":
                bpy.ops.import_scene.gltf(filepath=tmp_path)
            elif format == "fbx":
                bpy.ops.import_scene.fbx(filepath=tmp_path)
            elif format == "obj":
                bpy.ops.wm.obj_import(filepath=tmp_path)
            elif format == "zip":
                # Extract and find GLB
                import zipfile
                with zipfile.ZipFile(tmp_path, 'r') as zip_ref:
                    extract_dir = tempfile.mkdtemp()
                    zip_ref.extractall(extract_dir)
                    # Find GLB file
                    for root, dirs, files in os.walk(extract_dir):
                        for f in files:
                            if f.endswith('.glb') or f.endswith('.gltf'):
                                glb_path = os.path.join(root, f)
                                bpy.ops.import_scene.gltf(filepath=glb_path)
                                break
            else:
                raise ValueError(f"Unsupported format: {format}")

            # Rename imported object
            if bpy.context.selected_objects:
                bpy.context.selected_objects[0].name = name

            return {
                "imported": True,
                "name": name,
                "url": url,
            }

        except Exception as e:
            raise RuntimeError(f"Import failed: {e}")

        finally:
            # Cleanup temp file
            if os.path.exists(tmp_path):
                os.unlink(tmp_path)

    def method_list(self) -> list[MethodInfo]:
        """Return list of AI generation methods."""
        return [
            # Status
            MethodInfo("ai.status", "Check AI provider configuration status", []),
            MethodInfo("ai.providers", "List available AI generation providers", []),

            # Hyper3D Rodin
            MethodInfo("ai.hyper3d.generate_text", "Generate 3D from text prompt (Hyper3D Rodin)", [
                ParamInfo("prompt", "string", True, description="Text description of the 3D model"),
                ParamInfo("bbox_condition", "array", False, description="Bounding box [x, y, z] percentages"),
            ]),
            MethodInfo("ai.hyper3d.generate_image", "Generate 3D from images (Hyper3D Rodin)", [
                ParamInfo("image_urls", "array", False, description="URLs of reference images"),
                ParamInfo("image_paths", "array", False, description="Local paths of reference images"),
                ParamInfo("bbox_condition", "array", False),
            ]),
            MethodInfo("ai.hyper3d.poll", "Check Hyper3D generation status", [
                ParamInfo("job_id", "string", True, description="Generation job ID"),
            ]),
            MethodInfo("ai.hyper3d.import", "Import completed Hyper3D model", [
                ParamInfo("job_id", "string", False, description="Job ID to import from"),
                ParamInfo("url", "string", False, description="Direct download URL"),
                ParamInfo("name", "string", False, "Rodin_Model", description="Name for imported object"),
            ]),

            # Hunyuan3D
            MethodInfo("ai.hunyuan.generate", "Generate 3D using Hunyuan3D", [
                ParamInfo("prompt", "string", False, description="Text prompt"),
                ParamInfo("image_url", "string", False, description="Reference image URL"),
            ]),
            MethodInfo("ai.hunyuan.poll", "Check Hunyuan3D generation status", [
                ParamInfo("job_id", "string", True),
            ]),
            MethodInfo("ai.hunyuan.import", "Import Hunyuan3D model", [
                ParamInfo("job_id", "string", False),
                ParamInfo("url", "string", False),
                ParamInfo("name", "string", False, "Hunyuan_Model"),
            ]),

            # Meshy
            MethodInfo("ai.meshy.generate_text", "Generate 3D from text (Meshy)", [
                ParamInfo("prompt", "string", True),
                ParamInfo("art_style", "string", False, "realistic", description="realistic, cartoon, low-poly, sculpture"),
                ParamInfo("negative_prompt", "string", False),
            ]),
            MethodInfo("ai.meshy.generate_image", "Generate 3D from image (Meshy)", [
                ParamInfo("image_url", "string", True),
            ]),
            MethodInfo("ai.meshy.poll", "Check Meshy generation status", [
                ParamInfo("job_id", "string", True),
            ]),
            MethodInfo("ai.meshy.import", "Import Meshy model", [
                ParamInfo("job_id", "string", False),
                ParamInfo("url", "string", False),
                ParamInfo("name", "string", False, "Meshy_Model"),
            ]),

            # Job management
            MethodInfo("ai.job.status", "Get AI generation job status", [
                ParamInfo("job_id", "string", True),
            ]),
            MethodInfo("ai.job.list", "List AI generation jobs", [
                ParamInfo("provider", "string", False, description="Filter by provider"),
                ParamInfo("status", "string", False, description="Filter by status"),
                ParamInfo("limit", "integer", False, 50),
            ]),
        ]
