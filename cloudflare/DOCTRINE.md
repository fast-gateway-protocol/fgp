# Doctrine: cloudflare (FGP Cloudflare Daemon)

## Purpose
- Provide fast Cloudflare operations for agent workflows.
- Reduce latency for common API queries and actions.

## Scope
- Cloudflare API integration and FGP method surface.
- Local daemon execution and request routing.

## Non-Goals
- Full replacement for official Cloudflare clients.
- Managing billing or unrelated admin workflows.
- Hosting or multi-tenant service operation.

## Tenets
- Keep API calls explicit and minimal.
- Favor predictable responses over breadth.
- Warm-call performance is the primary metric.
- Avoid leaking sensitive data in logs.

## Architecture
- FGP daemon handles socket requests and dispatch.
- Cloudflare API calls authenticated via local credentials.

## Interfaces
- FGP methods for Cloudflare workflows.
- CLI entrypoints via `fgp call`.

## Operational Model
- Runs locally with provider credentials.
- Owners: Cloudflare daemon maintainers.

## Testing
- Integration tests for core endpoints.
- Error handling for auth and rate limits.

## Security
- Credentials provided via environment or local config.
- Avoid logging sensitive payloads.

## Observability
- Include timing metadata in responses.
- Surface API errors with context.

## Risks
- API changes or rate limits affecting reliability.

## Roadmap
- Expand coverage for additional Cloudflare workflows.
