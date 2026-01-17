# Doctrine Coverage Expansion Plan (Google-style Doctrines)

## Overview
Create a consistent, Google-style Doctrine for each major FGP module to clarify intent, boundaries, operating principles, and long-term direction. Start with a validated inventory and then execute a structured sprint sequence that prioritizes the most important modules.

## Requirements and Goals
- Establish a single Doctrine template and style guide aligned to Google-style doctrine conventions.
- Produce an immediate, repeatable inventory of current doctrine coverage across modules.
- Prioritize core modules (highest user impact, foundational APIs, or broad dependency surface).
- Maintain clear ownership and review expectations for each Doctrine.
- Track progress via a measurable coverage metric.

## Technical Approach and Architecture
- **Doctrine template**: Define a canonical structure with required headings, examples, and constraints (1-2 page target).
- **Inventory**: Use a lightweight script to scan modules, detect doctrine docs, and evaluate required headings.
- **Prioritization**: Rank modules by dependency breadth + runtime criticality + external API surface.
- **Sprint cadence**: 1-2 modules per sprint, with consistent review gates and acceptance checklist.

### Proposed Doctrine Template (Google-style)
Required headings (baseline):
- Purpose
- Scope
- Non-Goals
- Tenets
- Architecture
- Interfaces
- Operational Model
- Testing
- Security
- Observability
- Risks
- Roadmap

Notes:
- Keep language concise, prescriptive, and centered on decision-making.
- Add service-specific sections when needed (SLOs, rate limits, quotas).

## Implementation Steps
1. **Scaffold template + rubric**
   - Draft a standard Doctrine template under `docs/`.
   - Define acceptance criteria and review checklist.
2. **Generate inventory**
   - Run `scripts/doctrine_inventory.py` to capture current coverage and missing sections.
   - Store inventory snapshot in `docs/` or `notes/` as a baseline.
3. **Prioritize modules**
   - Tier 0 (core): `daemon/`, `protocol/`, `cli/`, `browser/`.
   - Tier 1 (critical APIs): `gmail/`, `calendar/`, `github/`, `neon/`, `vercel/`.
   - Tier 2 (supporting services): `fly/`, `mcp-bridge/`, `registry/`, `dashboard/`, `workflow/`.
4. **Execute doctrine sprints**
   - Sprint 1: Tier 0 modules.
   - Sprint 2+: Tier 1 modules, then Tier 2.
   - Enforce a short review cycle and add owners in each Doctrine.
5. **Publish and validate**
   - Update module `README.md` with a Doctrine link.
   - Capture coverage metrics and update plan status.

## Files to Create or Modify
- `scripts/doctrine_inventory.py` (new, immediate inventory tool)
- `docs/Doctrine_Template.md` (new)
- `docs/doctrine_inventory_YYYY-MM-DD.md` (new, generated snapshot)
- Module-level doctrine docs (new; per module)
- Module `README.md` updates to link to doctrine docs

## Dependencies and Prerequisites
- Python 3 for the inventory script.
- Agreement on the final Doctrine template and required headings.

## Testing Strategy
- Dry-run inventory script on the repo root and validate output formatting.
- Verify each created doctrine doc passes the required headings check.
- Ensure README links resolve correctly.

## Potential Challenges and Edge Cases
- Ambiguous module boundaries (some top-level folders are not code modules).
- Mixed-language modules with non-standard docs layout.
- Doctrine scope creep (documents becoming architecture specs).
- “Doctrine” terminology mismatch with existing documentation conventions.

## Success Criteria
- 100% coverage for Tier 0 and Tier 1 modules.
- Each doctrine includes all required headings and an owner.
- Inventory script produces a clean, reproducible baseline report.
- README links to doctrines are consistent and validated.

## Immediate Next Steps
- Run `scripts/doctrine_inventory.py` to capture the baseline.
- Decide whether to finalize the required headings list or customize it per module tier.
- Draft the first Doctrine for `daemon/` as the pattern for the rest.
