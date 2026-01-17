# Doctrine: generator (FGP Generator)

## Purpose
- Generate scaffolding for FGP packages and daemons.
- Reduce boilerplate and enforce conventions.

## Scope
- Templates, generators, and config presets.

## Non-Goals
- Replacing manual design decisions.
- Generating fully production-ready services without review.

## Tenets
- Prefer explicit templates over opaque codegen.
- Keep generated output easy to modify.

## Architecture
- Template-driven generator with module presets.

## Interfaces
- CLI or scripts that emit project scaffolds.

## Operational Model
- Run locally by developers when creating new modules.
- Owners: Generator maintainers.

## Testing
- Snapshot tests for generated output.

## Security
- Avoid embedding secrets in generated files.

## Observability
- Clear logs for generator actions.

## Risks
- Template drift from current best practices.

## Roadmap
- Expand templates to cover more service types.
