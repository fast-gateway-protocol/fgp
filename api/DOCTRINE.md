# Doctrine: api (FGP API)

## Purpose
- Provide backend APIs that power FGP applications and services.
- Offer stable, documented interfaces for clients.

## Scope
- API endpoints, request/response schemas, and auth boundaries.
- Core data access for FGP app workflows.

## Non-Goals
- Acting as a public service without authorization.
- Replacing daemon-to-daemon communication.

## Tenets
- API stability over rapid change.
- Clear error messages and predictable schemas.
- Keep latency low and payloads small.

## Architecture
- Node-based API service with modular routing.
- Shared utilities for auth and validation.

## Interfaces
- HTTP endpoints and documented schemas.

## Operational Model
- Runs as a service for FGP applications.
- Owners: API maintainers.

## Testing
- Contract tests for endpoints.
- Validation and auth failure scenarios.

## Security
- Enforce auth and input validation.
- Avoid leaking sensitive data.

## Observability
- Structured request logging and metrics.

## Risks
- Schema drift with client apps.

## Roadmap
- Expand endpoint coverage with versioned APIs.
