# Doctrine: google-sheets (FGP Google Sheets Daemon)

## Purpose
- Provide fast Google Sheets operations for agent workflows.
- Reduce latency for common API queries and actions.

## Scope
- Google Sheets API integration and FGP method surface.
- Local daemon execution and request routing.

## Non-Goals
- Full replacement for official Google Sheets clients.
- Managing billing or unrelated admin workflows.
- Hosting or multi-tenant service operation.

## Tenets
- Keep API calls explicit and minimal.
- Favor predictable responses over breadth.
- Warm-call performance is the primary metric.
- Avoid leaking sensitive data in logs.

## Architecture
- FGP daemon handles socket requests and dispatch.
- Google Sheets API calls authenticated via local credentials.

## Interfaces
- FGP methods for Google Sheets workflows.
- CLI entrypoints via `fgp call`.

## Operational Model
- Runs locally with provider credentials.
- Owners: Google Sheets daemon maintainers.

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
- Expand coverage for additional Google Sheets workflows.
