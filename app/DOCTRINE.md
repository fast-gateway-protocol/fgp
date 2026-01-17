# Doctrine: app (FGP App)

## Purpose
- Provide the primary user-facing application experience for FGP.
- Make core workflows discoverable and easy to operate.

## Scope
- UI surfaces, client-side logic, and user interactions.
- Integration with backend APIs and daemon status.

## Non-Goals
- Replacing CLI workflows for power users.
- Hosting remote services or infrastructure management.

## Tenets
- Favor clarity and speed over visual complexity.
- Keep critical paths short and predictable.
- Preserve compatibility with core APIs.

## Architecture
- Client application with API integrations.
- Shared UI components and routing.

## Interfaces
- UI routes and client-side APIs.

## Operational Model
- Runs locally or in browser contexts.
- Owners: App maintainers.

## Testing
- UI and integration tests for core flows.

## Security
- Avoid exposing tokens or sensitive data in logs.

## Observability
- Client-side logging and error reporting.

## Risks
- UI drift from backend capabilities.

## Roadmap
- Improve onboarding and workflow guidance.
