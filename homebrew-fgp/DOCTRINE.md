# Doctrine: homebrew-fgp (FGP Homebrew)

## Purpose
- Distribute FGP binaries via Homebrew formulas.
- Keep installation smooth for macOS users.

## Scope
- Homebrew formulas and packaging scripts.

## Non-Goals
- Serving binaries directly or managing releases.
- Supporting non-Homebrew package managers.

## Tenets
- Keep formulas minimal and reliable.
- Track upstream versions accurately.

## Architecture
- Formula definitions that fetch and install FGP binaries.

## Interfaces
- Homebrew formula files under `Formula/`.

## Operational Model
- Updated alongside releases.
- Owners: Packaging maintainers.

## Testing
- Install tests for formula validation.

## Security
- Verify checksums for packaged artifacts.

## Observability
- Clear changelog for formula updates.

## Risks
- Formula lag behind release cadence.

## Roadmap
- Automate formula updates from release metadata.
