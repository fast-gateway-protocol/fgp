# Doctrine: screen-time (FGP Screen Time Daemon)

## Purpose
- Provide fast local screen time operations for agent workflows.
- Reduce latency for local data access.

## Scope
- Local system integration and FGP method surface.
- Daemon lifecycle and request handling.

## Non-Goals
- Replacing native apps or full UI features.
- Remote hosting or multi-tenant access.

## Tenets
- Respect OS permissions and privacy.
- Keep output minimal and deterministic.
- Warm-call latency is the primary performance metric.

## Architecture
- FGP daemon handles socket requests and dispatch.
- Local system APIs or databases provide data.

## Interfaces
- FGP methods for screen time workflows.
- CLI entrypoints via `fgp call` or module binaries.

## Operational Model
- Runs locally; requires OS permissions.
- Owners: Screen Time daemon maintainers.

## Testing
- Integration tests on supported OS versions.
- Permission and database access failure tests.

## Security
- Enforce least-privilege access to local data.
- Avoid logging sensitive content.

## Observability
- Structured logs and response timing metadata.

## Risks
- OS or schema changes breaking access.

## Roadmap
- Expand coverage for additional screen time workflows.
