# Unit Test Coverage Expansion Plan

## Overview
Systematically expand unit test coverage across the FGP workspace with a consistent inventory workflow, clear prioritization tiers, and sprint-based execution. Start with the most important modules and establish a durable baseline for tracking progress over time.

## Requirements and Goals
- Establish a baseline inventory of existing unit tests by module.
- Prioritize core modules (protocol, SDKs, CLI, and critical daemons).
- Add focused unit tests for request/response shape, error handling, and boundary conditions.
- Improve confidence without slowing development velocity.
- Track coverage with repeatable snapshots.

## Technical Approach and Architecture
- **Inventory script**: Scan top-level modules for test files and inline tests.
- **Tiered prioritization**: Focus on highest impact modules first.
- **Test taxonomy**: Separate unit, integration, and e2e concerns.
- **Sprint cadence**: 1-2 modules per sprint with measurable targets.

## Implementation Steps
1. **Baseline inventory**
   - Run `scripts/test_inventory.py` to capture current test coverage.
   - Save the output as a dated snapshot in `docs/`.
2. **Define coverage expectations**
   - For Rust: unit tests for core logic + error paths; keep integration tests in `tests/`.
   - For Python: `pytest` unit tests in `tests/`.
   - For Node: `*.test.*` or `__tests__/` organization.
3. **Prioritize modules**
   - **Tier 0 (core)**: `daemon/`, `protocol/`, `cli/`, `browser/`.
   - **Tier 1 (critical services)**: `gmail/`, `calendar/`, `github/`, `neon/`, `vercel/`, `mcp-bridge/`.
   - **Tier 2 (infra & supporting)**: `registry/`, `workflow/`, `dashboard/`, `fly/`, `daemon-py/`.
   - **Tier 3 (ecosystem)**: remaining daemons and integrations.
4. **Sprint execution**
   - Add unit tests for parsing, validation, error handling, and boundary cases.
   - Use fixtures for deterministic input/output.
   - Keep test runtime under control (fast suite).
5. **Review and gate**
   - Require new/updated tests for request/response shape changes.
   - Track coverage growth by module tier.

## Files to Create or Modify
- `scripts/test_inventory.py` (new)
- `docs/test_inventory_YYYY-MM-DD.md` (generated snapshot)
- `tests/` or `src/` tests per module

## Dependencies and Prerequisites
- Python 3 for inventory script.
- Agreement on test placement and naming conventions.

## Testing Strategy
- Run per-module test commands in relevant sub-repos (e.g., `cargo test`).
- Add unit tests alongside any protocol or daemon logic changes.

## Potential Challenges and Edge Cases
- Mixed language modules with inconsistent test layouts.
- Dependency-heavy modules may require mocks/fixtures.
- Long-running integration tests slowing CI.

## Success Criteria
- Tier 0 + Tier 1 modules have meaningful unit tests and regression coverage.
- Inventory snapshots show sustained growth.
- Clear, repeatable test structure across modules.

## Immediate Next Steps
- Generate and review the baseline inventory snapshot.
- Start with Tier 0 unit tests (protocol parsing, daemon dispatch, CLI command parsing, browser request validation).
