# Doctrine: registry (FGP Skill Registry)

## Purpose
- Provide a central registry for discovering and distributing FGP skills.
- Ensure skills can be indexed, validated, and synced reliably.

## Scope
- Registry API and database schema for skills metadata.
- Skill import, validation, and sync workflows.

## Non-Goals
- Serving binary daemon artifacts.
- Acting as a general package registry beyond skills.
- Hosting user authentication beyond registry needs.

## Tenets
- Trust but verify: validate skill content rigorously.
- Keep APIs stable and documented.
- Prefer deterministic indexing over heuristic enrichment.

## Architecture
- Rust API server with database-backed storage.
- Import pipeline for multiple skill formats.
- Sync process for external sources.

## Interfaces
- HTTP API server (registry endpoints).
- Database schema migrations under `migrations/`.

## Operational Model
- Runs as a local or hosted service with database access.
- Owners: Registry maintainers.
- Changes should be backward compatible with clients.

## Testing
- Integration tests for API endpoints and sync jobs.
- Validation tests for skill import and security checks.

## Security
- Validate inbound skill data to prevent unsafe content.
- Protect database credentials and webhooks.

## Observability
- Structured logging for import and sync operations.
- Metrics for ingestion success/failure rates.

## Risks
- Schema drift between registry and clients.
- Untrusted skill content or supply chain issues.

## Roadmap
- Add API auth and rate limiting.
- Improve sync performance for large registries.
