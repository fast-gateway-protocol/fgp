# FGP Skill Platform

> The canonical skill registry and marketplace for AI coding agents, with FGP acceleration at its core.

---

## Plan Status

| Field | Value |
|-------|-------|
| **Status** | Planning |
| **Created** | 01/15/2026 07:47 AM PST |
| **Last Updated** | 01/15/2026 07:47 AM PST |
| **Owner** | Wolfgang |
| **Priority** | P0 - Strategic |
| **Completion** | 0% |

---

## Executive Summary

Build the **canonical skill registry and marketplace** for AI coding agents. While SkillsMP.com has indexed 65k+ skills from GitHub, they only provide discovery. FGP Platform provides the full lifecycle: **discovery → installation → acceleration → publishing**.

**Key insight:** SKILL.md has become the universal format across Claude Code, Codex CLI, Gemini CLI, and Cursor. FGP is positioned to be the **npm of agent skills** - agent-agnostic infrastructure with performance acceleration.

### Value Proposition

| Actor | Value |
|-------|-------|
| **Skill Users** | One-command install, auto-updates, cross-agent compatibility |
| **Skill Authors** | Canonical distribution, usage metrics, FGP acceleration |
| **Agent Vendors** | Ecosystem growth, standardized skill format |
| **FGP** | Platform lock-in, distribution moat, revenue potential |

---

## Market Analysis

### Current Landscape

| Platform | Role | Limitations |
|----------|------|-------------|
| **SkillsMP** | Discovery (indexes GitHub) | No installation, no runtime, single-agent UI |
| **GitHub** | Source hosting | No skill-specific features, scattered repos |
| **anthropics/skills** | Official examples | Limited selection, no marketplace |
| **Agent CLIs** | Local installation | Manual process, no registry |

### Opportunity

- **65,635 skills** already exist in SKILL.md format
- **No canonical registry** - skills scattered across GitHub
- **No installation tooling** - users manually clone repos
- **No cross-agent story** - each agent has its own skill folder
- **No acceleration layer** - all skills run at native speed

### Quality Reality Check

**Not all 65k skills are worth indexing.** Many are:
- Low-quality repos with 0-2 stars
- Abandoned/unmaintained
- Potentially malicious (random GitHub repos = untrusted code)
- Duplicates or forks

**Strategy: Quality tiers, not bulk import.**

---

## Quality Tiers & Security

### Tier System

```
┌─────────────────────────────────────────────────────────────────────┐
│                       QUALITY TIERS                                 │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  TIER 0: VERIFIED (~50-100 skills)                          │   │
│  │  • Official vendor skills (anthropics/skills, pytorch, etc) │   │
│  │  • FGP-native skills (our own)                              │   │
│  │  • Manually reviewed and tested                             │   │
│  │  • Install without prompts                                  │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              ▼                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  TIER 1: TRUSTED (~1,000-2,000 skills)                      │   │
│  │  • 100+ GitHub stars                                        │   │
│  │  • From known orgs (vercel, electron, google, etc)          │   │
│  │  • Active maintenance (updated in last 6 months)            │   │
│  │  • Has marketplace.json                                     │   │
│  │  • Install with brief confirmation                          │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              ▼                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  TIER 2: COMMUNITY (~5,000-10,000 skills)                   │   │
│  │  • 10+ GitHub stars                                         │   │
│  │  • Updated in last 12 months                                │   │
│  │  • Valid SKILL.md format                                    │   │
│  │  • Install with warning + confirmation                      │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              ▼                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  TIER 3: UNVERIFIED (~50,000+ skills)                       │   │
│  │  • Everything else indexed by SkillsMP                      │   │
│  │  • Searchable but NOT installable by default                │   │
│  │  • Must use --allow-unverified flag                         │   │
│  │  • Strong warning about security risks                      │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

### Initial Sync Strategy

**Phase 1: Start with Tier 0 + Tier 1 only (~2,000 skills)**

```bash
# SkillsMP API filters
GET /api/v1/skills/search?sortBy=stars&limit=100&page=1-20

# Filter criteria for initial sync:
- stars >= 100 (Tier 1) OR
- from trusted orgs (anthropics, pytorch, vercel, electron, etc) OR
- has marketplace.json
```

**Phase 2: Expand to Tier 2 (~10,000 skills)**
- Add skills with 10+ stars
- After we have security scanning in place

**Phase 3: Index Tier 3 (searchable only)**
- Full SkillsMP index for discovery
- Users see results but can't install without explicit flag

### Security Measures

| Layer | Protection |
|-------|------------|
| **Tier filtering** | Only install from Tier 0-2 by default |
| **Content scanning** | Check SKILL.md for suspicious patterns (shell commands, URLs) |
| **Sandbox preview** | `fgp skill preview <name>` shows what will be installed |
| **Checksum verification** | SHA256 of SKILL.md stored, verified on install |
| **Rollback** | `fgp skill uninstall <name>` removes cleanly |
| **Audit log** | Track all installations locally |

### Trusted Organizations (Tier 1 auto-qualify)

```yaml
trusted_orgs:
  - anthropics      # Claude/Anthropic official
  - openai          # Codex official
  - google-gemini   # Gemini official
  - pytorch         # PyTorch
  - vercel          # Next.js, Vercel
  - electron        # Electron
  - oven-sh         # Bun
  - n8n-io          # n8n automation
  - microsoft       # Microsoft
  - github          # GitHub
  - cloudflare      # Cloudflare
  - supabase        # Supabase
  - prisma          # Prisma
```

### CLI Behavior by Tier

```bash
# Tier 0: Verified - installs silently
$ fgp skill install anthropics/browser
✓ Installed anthropics/browser (verified)

# Tier 1: Trusted - brief confirmation
$ fgp skill install vercel/cache-components
⚡ vercel/cache-components (137k stars, trusted org)
   Install? [Y/n] y
✓ Installed

# Tier 2: Community - warning + confirmation
$ fgp skill install someuser/cool-skill
⚠️  someuser/cool-skill (42 stars, community)
   This skill is from an unverified author.
   Review source: https://github.com/someuser/cool-skill
   Install anyway? [y/N] y
✓ Installed

# Tier 3: Unverified - blocked by default
$ fgp skill install random/sketchy-skill
✗ random/sketchy-skill is unverified (2 stars)
  To install anyway: fgp skill install random/sketchy-skill --allow-unverified

$ fgp skill install random/sketchy-skill --allow-unverified
🚨 WARNING: This skill has not been reviewed for security.
   You are installing code from an untrusted source.
   Source: https://github.com/random/sketchy-skill

   Type 'I UNDERSTAND THE RISK' to proceed: I UNDERSTAND THE RISK
✓ Installed (unverified)
```

---

### Competitive Moat

1. **First mover** - No one has built the skill registry yet
2. **FGP acceleration** - 10-100x speedup is defensible differentiation
3. **Cross-agent** - Agent-agnostic from day one
4. **Network effects** - More skills → more users → more authors

---

## Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        FGP SKILL PLATFORM                               │
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │                      REGISTRY SERVICE                              │ │
│  │                                                                    │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐               │ │
│  │  │   Sync      │  │   Search    │  │   Publish   │               │ │
│  │  │   Engine    │  │   API       │  │   API       │               │ │
│  │  │             │  │             │  │             │               │ │
│  │  │ • GitHub    │  │ • Keyword   │  │ • Validate  │               │ │
│  │  │ • SkillsMP  │  │ • Semantic  │  │ • Store     │               │ │
│  │  │ • Direct    │  │ • Category  │  │ • Version   │               │ │
│  │  └─────────────┘  └─────────────┘  └─────────────┘               │ │
│  │         │                │                │                       │ │
│  │         └────────────────┼────────────────┘                       │ │
│  │                          ▼                                        │ │
│  │  ┌─────────────────────────────────────────────────────────────┐ │ │
│  │  │                    SKILL DATABASE                           │ │ │
│  │  │                                                             │ │ │
│  │  │  • Skill metadata (name, description, author, version)     │ │ │
│  │  │  • SKILL.md content (validated, parsed)                    │ │ │
│  │  │  • Cross-agent compatibility flags                         │ │ │
│  │  │  • FGP daemon bindings                                     │ │ │
│  │  │  • Usage statistics, ratings                               │ │ │
│  │  └─────────────────────────────────────────────────────────────┘ │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                   │                                     │
│                                   ▼                                     │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │                         FGP CLI                                    │ │
│  │                                                                    │ │
│  │  fgp skill search <query>      # Search registry                  │ │
│  │  fgp skill install <name>      # Install to agent(s)              │ │
│  │  fgp skill update [name]       # Update installed skills          │ │
│  │  fgp skill list                # List installed skills            │ │
│  │  fgp skill info <name>         # Show skill details               │ │
│  │  fgp skill publish <path>      # Publish to registry              │ │
│  │  fgp skill convert <path>      # Cross-agent conversion           │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                   │                                     │
│                                   ▼                                     │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │                    AGENT TARGETS                                   │ │
│  │                                                                    │ │
│  │  ~/.claude/skills/     Claude Code (Anthropic)                    │ │
│  │  ~/.codex/skills/      Codex CLI (OpenAI)                         │ │
│  │  ~/.gemini/skills/     Gemini CLI (Google)                        │ │
│  │  ~/.cursor/skills/     Cursor (Anysphere)                         │ │
│  │  [custom paths]        Other SKILL.md-compatible agents           │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                   │                                     │
│                                   ▼                                     │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │                    FGP ACCELERATION LAYER                          │ │
│  │                                                                    │ │
│  │  Skills with FGP daemon bindings get auto-accelerated:            │ │
│  │                                                                    │ │
│  │  gmail-gateway     → gmail daemon      (69x faster)               │ │
│  │  browser-fgp       → browser daemon    (292x faster)              │ │
│  │  calendar-gateway  → calendar daemon   (45x faster)               │ │
│  │  github-fgp        → github daemon     (75x faster)               │ │
│  └───────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘
```

### Data Model

```sql
-- Core skill table
CREATE TABLE skills (
    id              UUID PRIMARY KEY,
    name            TEXT NOT NULL UNIQUE,
    slug            TEXT NOT NULL UNIQUE,
    description     TEXT,
    author          TEXT,
    author_url      TEXT,
    version         TEXT DEFAULT '1.0.0',
    license         TEXT,

    -- Source tracking
    source_type     TEXT NOT NULL,  -- 'github', 'skillsmp', 'direct'
    source_url      TEXT,
    source_repo     TEXT,           -- 'org/repo' for GitHub
    source_path     TEXT,           -- Path within repo
    source_sha      TEXT,           -- Git SHA for versioning

    -- Content
    skill_md        TEXT NOT NULL,  -- Raw SKILL.md content
    parsed_meta     JSONB,          -- Parsed frontmatter

    -- Compatibility
    agents          TEXT[],         -- ['claude-code', 'codex', 'gemini', 'cursor']
    min_agent_ver   JSONB,          -- {'claude-code': '1.0.0', ...}

    -- FGP integration
    fgp_daemon      TEXT,           -- Linked FGP daemon name
    fgp_speedup     FLOAT,          -- Measured speedup factor

    -- Metadata
    stars           INTEGER DEFAULT 0,
    downloads       INTEGER DEFAULT 0,
    rating          FLOAT,

    -- Timestamps
    created_at      TIMESTAMP DEFAULT NOW(),
    updated_at      TIMESTAMP DEFAULT NOW(),
    synced_at       TIMESTAMP,

    -- Flags
    verified        BOOLEAN DEFAULT FALSE,
    featured        BOOLEAN DEFAULT FALSE,
    deprecated      BOOLEAN DEFAULT FALSE
);

-- Skill versions (for rollback/pinning)
CREATE TABLE skill_versions (
    id              UUID PRIMARY KEY,
    skill_id        UUID REFERENCES skills(id),
    version         TEXT NOT NULL,
    skill_md        TEXT NOT NULL,
    source_sha      TEXT,
    created_at      TIMESTAMP DEFAULT NOW(),

    UNIQUE(skill_id, version)
);

-- Categories
CREATE TABLE categories (
    id              UUID PRIMARY KEY,
    name            TEXT NOT NULL UNIQUE,
    slug            TEXT NOT NULL UNIQUE,
    description     TEXT,
    skill_count     INTEGER DEFAULT 0
);

-- Skill-category mapping
CREATE TABLE skill_categories (
    skill_id        UUID REFERENCES skills(id),
    category_id     UUID REFERENCES categories(id),
    PRIMARY KEY (skill_id, category_id)
);

-- User installations (for analytics)
CREATE TABLE installations (
    id              UUID PRIMARY KEY,
    skill_id        UUID REFERENCES skills(id),
    agent           TEXT NOT NULL,
    installed_at    TIMESTAMP DEFAULT NOW(),
    machine_hash    TEXT  -- Anonymized machine identifier
);

-- FGP daemon mappings
CREATE TABLE fgp_daemons (
    id              UUID PRIMARY KEY,
    name            TEXT NOT NULL UNIQUE,
    socket_path     TEXT NOT NULL,
    methods         TEXT[],
    version         TEXT,
    speedup_avg     FLOAT
);
```

### API Design

```yaml
# Registry API (REST + optional GraphQL)
Base URL: https://registry.fgp.dev/api/v1

# Search
GET /skills/search
  ?q=<query>
  &category=<slug>
  &agent=<agent-name>
  &fgp_only=<bool>
  &sort=stars|downloads|recent
  &page=<n>
  &limit=<n>

GET /skills/ai-search
  ?q=<natural language query>

# Skill details
GET /skills/:slug
GET /skills/:slug/versions
GET /skills/:slug/versions/:version
GET /skills/:slug/skill.md

# Categories
GET /categories
GET /categories/:slug/skills

# Publishing
POST /skills
  Body: { skill_md: "...", source_url: "..." }
  Auth: Bearer token

PUT /skills/:slug
  Body: { skill_md: "...", version: "..." }
  Auth: Bearer token (must be author)

# Stats
GET /stats
GET /skills/:slug/stats

# FGP integration
GET /daemons
GET /skills/:slug/fgp-binding
```

---

## Implementation Phases

### Phase 1: Registry Foundation (2 weeks)

**Goal:** Functional registry with curated skills and basic CLI

#### 1.1 Database Setup
- [ ] Provision Neon Postgres database
- [ ] Create schema (tables above)
- [ ] Add quality_tier column (0-3)
- [ ] Seed categories from SkillsMP (12 categories)

#### 1.2 Curated Sync Engine
- [ ] SkillsMP API client with star filtering
- [ ] GitHub API client (for metadata enrichment)
- [ ] SKILL.md parser and validator
- [ ] **Initial sync: Tier 0 + Tier 1 only (~2,000 skills)**
  - Filter: stars >= 100 OR trusted_org OR has marketplace.json
  - Skip: repos with < 10 stars
  - Manual review: anthropics/skills (Tier 0)

#### 1.3 Search API
- [ ] Keyword search endpoint
- [ ] Category filtering
- [ ] **Tier filtering (default: Tier 0-2 only)**
- [ ] Pagination and sorting
- [ ] Basic rate limiting

#### 1.4 CLI Foundation
- [ ] `fgp skill search <query>`
- [ ] `fgp skill info <name>` (shows tier, stars, source)
- [ ] Pretty terminal output (tables, colors)
- [ ] Tier badges in output (✓ verified, ⚡ trusted, ⚠️ community)

**Deliverable:** Can search ~2,000 curated skills from CLI

---

### Phase 2: Installation Flow (2 weeks)

**Goal:** Secure, tier-aware skill installation to any agent

#### 2.1 Agent Detection
- [ ] Detect installed agents (Claude Code, Codex, Gemini, Cursor)
- [ ] Locate skill directories for each agent
- [ ] Handle custom paths via config

#### 2.2 Tier-Aware Installer
- [ ] Download skill from registry
- [ ] Validate SKILL.md integrity (SHA256 checksum)
- [ ] **Tier-based confirmation prompts:**
  - Tier 0: Silent install
  - Tier 1: Brief confirmation
  - Tier 2: Warning + confirmation
  - Tier 3: Blocked unless `--allow-unverified`
- [ ] `fgp skill preview <name>` to inspect before install
- [ ] Install to target agent(s)

#### 2.3 Security Scanning
- [ ] Check for suspicious patterns in SKILL.md:
  - Raw shell commands (`rm`, `curl | sh`, etc.)
  - Hardcoded URLs/IPs
  - Base64 encoded content
  - References to sensitive paths
- [ ] Flag and warn on detection (don't block Tier 1-2)

#### 2.4 Installation Tracking
- [ ] Local manifest of installed skills (`~/.fgp/installed.json`)
- [ ] Track: skill, version, tier, install date, agent
- [ ] `fgp skill list` shows installed skills with tiers
- [ ] Audit log for all install/uninstall actions

#### 2.5 Update Mechanism
- [ ] Check registry for newer versions
- [ ] `fgp skill update` updates all
- [ ] `fgp skill update <name>` updates specific skill
- [ ] Respect tier: don't auto-update to lower-tier version

**Deliverable:** `fgp skill install vercel/cache-components` with tier prompts

---

### Phase 3: FGP Acceleration (2 weeks)

**Goal:** Skills with FGP daemons get automatic speedup

#### 3.1 Daemon Mapping
- [ ] Define daemon-to-skill mappings in registry
- [ ] Document which methods each daemon accelerates
- [ ] Track speedup metrics

#### 3.2 Acceleration Detection
- [ ] On install, check if FGP daemon available
- [ ] Auto-configure skill to use daemon
- [ ] Fallback to native if daemon not installed

#### 3.3 Skill Enhancement
- [ ] Generate FGP-enhanced SKILL.md variants
- [ ] Include daemon invocation in skill instructions
- [ ] Badge "FGP Accelerated" in listings

#### 3.4 Performance Dashboard
- [ ] Track acceleration metrics
- [ ] Show speedup in `fgp skill info`
- [ ] Aggregate stats for marketing

**Deliverable:** Installing gmail-gateway auto-uses gmail daemon

---

### Phase 4: Publishing Flow (2 weeks)

**Goal:** Authors can publish and update skills

#### 4.1 Publisher Authentication
- [ ] GitHub OAuth for identity
- [ ] API key generation
- [ ] Author verification

#### 4.2 Publish API
- [ ] `fgp skill publish ./my-skill`
- [ ] Validate SKILL.md format
- [ ] Check for naming conflicts
- [ ] Version management

#### 4.3 Update Flow
- [ ] Authors can push new versions
- [ ] Changelog tracking
- [ ] Deprecation support

#### 4.4 Quality Controls
- [ ] Automated SKILL.md validation
- [ ] Malware scanning (basic)
- [ ] Community flagging

**Deliverable:** New skills can be published to registry

---

### Phase 5: Cross-Agent Features (2 weeks)

**Goal:** Seamless cross-agent skill portability

#### 5.1 Compatibility Matrix
- [ ] Track which skills work on which agents
- [ ] Document agent-specific quirks
- [ ] Flag incompatibilities

#### 5.2 Conversion Engine
- [ ] `fgp skill convert ./skill --target=codex`
- [ ] Handle agent-specific syntax differences
- [ ] Generate compatibility shims

#### 5.3 Multi-Agent Install
- [ ] `fgp skill install X --agents=all`
- [ ] Install to multiple agents simultaneously
- [ ] Sync installations across agents

#### 5.4 Agent Parity Testing
- [ ] Test skills across all supported agents
- [ ] Report compatibility issues
- [ ] Auto-generate compatibility badges

**Deliverable:** Install once, works on Claude Code + Codex + Gemini

---

### Phase 6: Web Marketplace (3 weeks)

**Goal:** Web UI for discovery and management

#### 6.1 Marketing Site
- [ ] Landing page (value prop, getting started)
- [ ] Documentation
- [ ] Blog / changelog

#### 6.2 Skill Browser
- [ ] Search interface
- [ ] Category browsing
- [ ] Skill detail pages
- [ ] Installation instructions

#### 6.3 Author Dashboard
- [ ] Manage published skills
- [ ] View download stats
- [ ] Respond to issues

#### 6.4 User Accounts
- [ ] Favorites / bookmarks
- [ ] Installation history
- [ ] Ratings and reviews

**Deliverable:** registry.fgp.dev live with full UI

---

## Technical Decisions

### Database: Neon Postgres
- **Why:** Already integrated with FGP, serverless scaling, branching for migrations
- **Alternative considered:** SQLite (simpler but no concurrent access for API)

### API: Rust (Axum)
- **Why:** Consistent with FGP codebase, fast, type-safe
- **Alternative considered:** Python FastAPI (faster to prototype but inconsistent)

### CLI: Extend existing `fgp` CLI
- **Why:** Single tool for all FGP operations, consistent UX
- **Alternative considered:** Separate `fgp-skills` binary (fragmentation)

### Sync Strategy: SkillsMP first, then direct GitHub
- **Why:** SkillsMP already indexed 65k skills, bootstrap faster
- **Alternative considered:** Build our own GitHub scraper (reinventing wheel)

### Search: Postgres full-text + optional Cloudflare AI
- **Why:** Start simple, add semantic search later
- **Alternative considered:** Elasticsearch (overkill for initial scale)

---

## Success Metrics

### Phase 1-2 (MVP)
- [ ] ~2,000 curated skills indexed (Tier 0-1)
- [ ] 100+ skills installed via CLI
- [ ] <500ms search latency
- [ ] Zero security incidents from installed skills

### Phase 3-4 (Growth)
- [ ] ~10,000 skills indexed (Tier 0-2)
- [ ] 50+ FGP-accelerated skills
- [ ] 10+ community-published skills
- [ ] 1000+ weekly active users

### Phase 5-6 (Scale)
- [ ] 4+ agents supported
- [ ] 10k+ weekly skill installations
- [ ] 100+ skill authors
- [ ] Full 65k index searchable (Tier 3 with warnings)

---

## Risks and Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| **Malware/malicious skills** | Critical | Medium | Tier system, security scanning, curated-first approach |
| SkillsMP API goes down/changes | High | Medium | Build GitHub fallback, cache data |
| Low adoption | High | Medium | Focus on killer FGP-accelerated skills |
| Agent vendors compete | Medium | Low | Stay agent-agnostic, focus on acceleration |
| Quality control issues | Medium | Medium | Automated validation, community flagging |
| Scale issues | Low | Low | Neon auto-scales, CDN for skill content |
| User installs bad skill despite warnings | Medium | Low | Audit logs, easy uninstall, sandboxed preview |

---

## Open Questions

1. **Monetization:** Free forever? Freemium? Paid tiers for publishers?
2. **Curation:** How much human review vs automated validation?
3. **Namespace:** Global names vs org-scoped (e.g., `@anthropic/browser`)?
4. **Versioning:** SemVer required? How to handle breaking changes?
5. **Dependencies:** Support skill dependencies? Or keep skills atomic?

---

## References

- [SkillsMP](https://skillsmp.com/) - Current skill discovery platform
- [Anthropic Skills Spec](https://code.claude.com/docs/en/skills) - Official SKILL.md format
- [OpenAI Codex Skills](https://github.com/openai/codex/blob/main/docs/skills.md) - Codex adoption
- [Agent Skills Spec](https://agentskills.io) - Cross-agent standard

---

## Changelog

| Date | Change | Author |
|------|--------|--------|
| 01/15/2026 07:47 AM PST | Initial plan created | Claude |

