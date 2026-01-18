"""
Job Queue for async Blender operations.

Handles long-running operations like renders without blocking the daemon.

CHANGELOG:
01/17/2026 - Initial implementation (Claude)
"""

from __future__ import annotations

import json
import logging
import os
import queue
import threading
import uuid
from dataclasses import dataclass, field, asdict
from datetime import datetime, timezone
from enum import Enum
from pathlib import Path
from typing import Any, Callable, Optional

logger = logging.getLogger(__name__)

# Job storage location
FGP_JOBS_DIR = Path.home() / ".fgp" / "services" / "blender" / "jobs"


class JobStatus(str, Enum):
    """Job status values."""

    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


class JobType(str, Enum):
    """Types of async jobs."""

    RENDER_IMAGE = "render_image"
    RENDER_ANIMATION = "render_animation"
    BAKE_PHYSICS = "bake_physics"
    BAKE_TEXTURES = "bake_textures"
    EXPORT = "export"


@dataclass
class Job:
    """A queued async job."""

    id: str
    job_type: JobType
    status: JobStatus
    created_at: str
    started_at: Optional[str] = None
    completed_at: Optional[str] = None
    progress: float = 0.0  # 0.0 to 1.0
    progress_message: str = ""
    params: dict[str, Any] = field(default_factory=dict)
    result: Optional[dict[str, Any]] = None
    error: Optional[str] = None
    session_id: Optional[str] = None

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        d = asdict(self)
        d["job_type"] = self.job_type.value
        d["status"] = self.status.value
        return d

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> Job:
        """Create from dictionary."""
        data["job_type"] = JobType(data["job_type"])
        data["status"] = JobStatus(data["status"])
        return cls(**data)


class JobQueue:
    """
    Async job queue for long-running Blender operations.

    Jobs are processed in a background thread and persist across restarts.
    Clients can poll job status without blocking.

    Example:
        queue = JobQueue()
        queue.start()

        # Submit render job
        job_id = queue.submit(
            job_type=JobType.RENDER_IMAGE,
            params={"output": "/tmp/render.png", "resolution": [1920, 1080]},
            handler=lambda params, progress_cb: blender_render(params, progress_cb)
        )

        # Poll status
        job = queue.get(job_id)
        print(f"Progress: {job.progress * 100:.1f}%")

        # Wait for completion
        job = queue.wait(job_id)
    """

    def __init__(self, max_workers: int = 2) -> None:
        """
        Initialize job queue.

        Args:
            max_workers: Maximum concurrent jobs (default 2)
        """
        self._jobs: dict[str, Job] = {}
        self._handlers: dict[str, Callable] = {}
        self._queue: queue.Queue[str] = queue.Queue()
        self._workers: list[threading.Thread] = []
        self._running = False
        self._lock = threading.RLock()
        self._max_workers = max_workers
        self._jobs_dir = FGP_JOBS_DIR
        self._jobs_dir.mkdir(parents=True, exist_ok=True)
        self._jobs_file = self._jobs_dir / "jobs.json"

        # Load persisted jobs
        self._load_jobs()

    def _load_jobs(self) -> None:
        """Load jobs from disk."""
        if not self._jobs_file.exists():
            return

        try:
            with open(self._jobs_file, "r") as f:
                data = json.load(f)
                for job_data in data.get("jobs", []):
                    job = Job.from_dict(job_data)
                    self._jobs[job.id] = job

                    # Re-queue pending/running jobs from previous session
                    if job.status in (JobStatus.PENDING, JobStatus.RUNNING):
                        job.status = JobStatus.PENDING  # Reset to pending
                        job.progress = 0.0
                        self._queue.put(job.id)
        except Exception as e:
            logger.error(f"Failed to load jobs: {e}")

    def _save_jobs(self) -> None:
        """Persist jobs to disk."""
        try:
            # Only save recent jobs (last 100)
            recent_jobs = sorted(
                self._jobs.values(),
                key=lambda j: j.created_at,
                reverse=True
            )[:100]

            data = {
                "jobs": [j.to_dict() for j in recent_jobs],
            }
            with open(self._jobs_file, "w") as f:
                json.dump(data, f, indent=2)
        except Exception as e:
            logger.error(f"Failed to save jobs: {e}")

    def start(self) -> None:
        """Start the job queue workers."""
        if self._running:
            return

        self._running = True

        for i in range(self._max_workers):
            worker = threading.Thread(
                target=self._worker_loop,
                name=f"fgp-blender-worker-{i}",
                daemon=True,
            )
            worker.start()
            self._workers.append(worker)

        logger.info(f"Started job queue with {self._max_workers} workers")

    def stop(self) -> None:
        """Stop the job queue."""
        self._running = False

        # Wait for workers to finish current jobs
        for worker in self._workers:
            worker.join(timeout=5.0)

        self._workers.clear()
        self._save_jobs()

        logger.info("Stopped job queue")

    def _worker_loop(self) -> None:
        """Worker thread main loop."""
        while self._running:
            try:
                # Get job with timeout to check running flag
                try:
                    job_id = self._queue.get(timeout=1.0)
                except queue.Empty:
                    continue

                self._process_job(job_id)
                self._queue.task_done()

            except Exception as e:
                logger.exception(f"Worker error: {e}")

    def _process_job(self, job_id: str) -> None:
        """Process a single job."""
        with self._lock:
            job = self._jobs.get(job_id)
            if not job:
                return

            handler = self._handlers.get(job_id)
            if not handler:
                job.status = JobStatus.FAILED
                job.error = "No handler registered"
                self._save_jobs()
                return

            job.status = JobStatus.RUNNING
            job.started_at = datetime.now(timezone.utc).isoformat()
            self._save_jobs()

        def progress_callback(progress: float, message: str = "") -> None:
            """Update job progress."""
            with self._lock:
                if job_id in self._jobs:
                    self._jobs[job_id].progress = min(max(progress, 0.0), 1.0)
                    self._jobs[job_id].progress_message = message

        try:
            result = handler(job.params, progress_callback)

            with self._lock:
                job.status = JobStatus.COMPLETED
                job.completed_at = datetime.now(timezone.utc).isoformat()
                job.progress = 1.0
                job.result = result
                self._save_jobs()

        except Exception as e:
            logger.exception(f"Job {job_id} failed: {e}")
            with self._lock:
                job.status = JobStatus.FAILED
                job.completed_at = datetime.now(timezone.utc).isoformat()
                job.error = str(e)
                self._save_jobs()

        finally:
            # Clean up handler reference
            with self._lock:
                self._handlers.pop(job_id, None)

    def submit(
        self,
        job_type: JobType,
        params: dict[str, Any],
        handler: Callable[[dict[str, Any], Callable[[float, str], None]], dict[str, Any]],
        session_id: Optional[str] = None,
    ) -> str:
        """
        Submit a new job.

        Args:
            job_type: Type of job
            params: Job parameters
            handler: Function to execute (receives params and progress callback)
            session_id: Optional session ID

        Returns:
            Job ID
        """
        job_id = str(uuid.uuid4())[:12]
        now = datetime.now(timezone.utc).isoformat()

        job = Job(
            id=job_id,
            job_type=job_type,
            status=JobStatus.PENDING,
            created_at=now,
            params=params,
            session_id=session_id,
        )

        with self._lock:
            self._jobs[job_id] = job
            self._handlers[job_id] = handler
            self._save_jobs()

        self._queue.put(job_id)

        logger.info(f"Submitted job: {job_id} ({job_type.value})")
        return job_id

    def get(self, job_id: str) -> Optional[Job]:
        """Get job by ID."""
        with self._lock:
            return self._jobs.get(job_id)

    def list(
        self,
        status: Optional[JobStatus] = None,
        job_type: Optional[JobType] = None,
        session_id: Optional[str] = None,
        limit: int = 50,
    ) -> list[Job]:
        """
        List jobs with optional filters.

        Args:
            status: Filter by status
            job_type: Filter by type
            session_id: Filter by session
            limit: Maximum jobs to return

        Returns:
            List of matching jobs
        """
        with self._lock:
            jobs = list(self._jobs.values())

        # Apply filters
        if status:
            jobs = [j for j in jobs if j.status == status]
        if job_type:
            jobs = [j for j in jobs if j.job_type == job_type]
        if session_id:
            jobs = [j for j in jobs if j.session_id == session_id]

        # Sort by creation time (newest first)
        jobs.sort(key=lambda j: j.created_at, reverse=True)

        return jobs[:limit]

    def cancel(self, job_id: str) -> bool:
        """
        Cancel a pending job.

        Args:
            job_id: Job to cancel

        Returns:
            True if cancelled
        """
        with self._lock:
            job = self._jobs.get(job_id)
            if not job:
                return False

            if job.status != JobStatus.PENDING:
                return False  # Can only cancel pending jobs

            job.status = JobStatus.CANCELLED
            job.completed_at = datetime.now(timezone.utc).isoformat()
            self._handlers.pop(job_id, None)
            self._save_jobs()

            logger.info(f"Cancelled job: {job_id}")
            return True

    def wait(self, job_id: str, timeout: Optional[float] = None) -> Optional[Job]:
        """
        Wait for a job to complete.

        Args:
            job_id: Job to wait for
            timeout: Maximum seconds to wait

        Returns:
            Completed job, or None if timeout
        """
        import time

        start = time.time()

        while True:
            job = self.get(job_id)
            if not job:
                return None

            if job.status in (JobStatus.COMPLETED, JobStatus.FAILED, JobStatus.CANCELLED):
                return job

            if timeout and (time.time() - start) > timeout:
                return None

            time.sleep(0.1)

    def cleanup(self, max_age_hours: int = 24) -> int:
        """
        Clean up old completed/failed jobs.

        Args:
            max_age_hours: Remove jobs older than this

        Returns:
            Number of jobs removed
        """
        from datetime import timedelta

        cutoff = datetime.now(timezone.utc) - timedelta(hours=max_age_hours)
        cutoff_iso = cutoff.isoformat()

        removed = 0
        with self._lock:
            for job_id, job in list(self._jobs.items()):
                if job.status in (JobStatus.COMPLETED, JobStatus.FAILED, JobStatus.CANCELLED):
                    if job.completed_at and job.completed_at < cutoff_iso:
                        del self._jobs[job_id]
                        removed += 1

            if removed > 0:
                self._save_jobs()

        logger.info(f"Cleaned up {removed} old jobs")
        return removed
