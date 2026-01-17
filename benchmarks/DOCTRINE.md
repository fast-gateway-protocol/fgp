# Doctrine: benchmarks (FGP Benchmarks)

## Purpose
- Measure and report FGP performance across key workflows.
- Provide repeatable benchmarks for regression detection.

## Scope
- Benchmark scripts, datasets, and reporting outputs.

## Non-Goals
- Production monitoring or always-on telemetry.
- Benchmarking third-party systems beyond FGP needs.

## Tenets
- Repeatability over ad-hoc testing.
- Prefer simple, explainable metrics.

## Architecture
- Script-based benchmarks with configurable iterations.

## Interfaces
- CLI scripts and benchmark outputs.

## Operational Model
- Run on demand by developers.
- Owners: Performance maintainers.

## Testing
- Sanity checks for benchmark scripts.

## Security
- Avoid using real credentials in benchmark fixtures.

## Observability
- Store benchmark outputs for comparison.

## Risks
- Benchmarks diverging from real workloads.

## Roadmap
- Add coverage for new daemons and workflows.
