# FGP Skill Import System Design

**Version:** 1.0.0
**Status:** Draft
**Author:** Wolfgang Schoenberger + Claude
**Created:** 01/15/2026

---

## Abstract

This document specifies a **bidirectional skill portability system** that extends FGP's existing export infrastructure with **import capabilities**. Given an existing skill in any supported agent format (Claude Code, Cursor, Codex, Gemini, etc.), the system reconstructs a canonical `skill.yaml` manifest, enabling round-trip skill portability across AI agent ecosystems.

## Motivation

### Current State
FGP can export from canonical format to 8 agent ecosystems:
```
skill.yaml → [Claude Code, Cursor, Codex, MCP, Zed, Windsurf, Gemini, Aider]
```

### Problem
Users have existing skills in various formats:
- Claude Code skills in `~/.claude/skills/*/`
- Cursor rules in `.cursorrules` files
- Codex configs in `codex.json`
- Custom agent instructions scattered across projects

**There is no way to:**
1. Import these into FGP's canonical format
2. Port skills between agent ecosystems
3. Consolidate duplicate skills into a single source of truth

### Solution
Add reverse-engineering capability:
```
[Any Agent Format] → Parser → Normalizer → skill.yaml + instruction files
```

---

## Design Overview

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     SKILL IMPORT PIPELINE                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │    INPUT     │    │   PARSER     │    │  NORMALIZER  │       │
│  │              │    │              │    │              │       │
│  │ SKILL.md    │───▶│ Format-      │───▶│ Unified      │       │
│  │ .cursorrules│    │ specific     │    │ Intermediate │       │
│  │ .codex.json │    │ extractors   │    │ Repr (UIR)   │       │
│  │ .mcp.json   │    │              │    │              │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
│                                                 │                │
│                                                 ▼                │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │   OUTPUT     │◀───│  GENERATOR   │◀───│   ENRICHER   │       │
│  │              │    │              │    │              │       │
│  │ skill.yaml  │    │ Canonical    │    │ Daemon       │       │
│  │ instructions/│    │ manifest     │    │ registry     │       │
│  │ workflows/  │    │ builder      │    │ lookup       │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
Input File (e.g., SKILL.md)
    │
    ▼
┌─────────────────────────────────────────────┐
│ 1. DETECT FORMAT                            │
│    - File extension / naming patterns       │
│    - Content structure analysis             │
│    - Confidence: High (deterministic)       │
└─────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────┐
│ 2. PARSE TO UIR                             │
│    - Extract metadata (name, version, etc.) │
│    - Extract instructions (markdown blocks) │
│    - Extract method references              │
│    - Confidence: Varies by format           │
└─────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────┐
│ 3. ENRICH FROM REGISTRY                     │
│    - Look up daemon method schemas          │
│    - Resolve version constraints            │
│    - Fill in missing parameter info         │
│    - Confidence: High (if in registry)      │
└─────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────┐
│ 4. GENERATE CANONICAL OUTPUT                │
│    - skill.yaml with confidence markers     │
│    - instructions/{agent}.md files          │
│    - Placeholder workflows (if detected)    │
│    - Validation report                      │
└─────────────────────────────────────────────┘
```

---

## Supported Import Formats

### Tier 1: High Fidelity (70-85% recovery)

| Format | File Pattern | Recovery Rate | Key Advantage |
|--------|--------------|---------------|---------------|
| **Claude Code** | `SKILL.md` | ~80% | YAML frontmatter preserves structured metadata |
| **Gemini** | `gemini-extension.json` + `GEMINI.md` | ~75% | JSON manifest is structured |

### Tier 2: Medium Fidelity (40-60% recovery)

| Format | File Pattern | Recovery Rate | Key Challenge |
|--------|--------------|---------------|---------------|
| **Cursor** | `.cursorrules` | ~50% | Pure markdown, no structured metadata |
| **Zed** | `.rules` | ~45% | Plaintext with semantic markers |
| **Windsurf** | `.windsurf.md` | ~50% | Markdown cascade format |
| **Aider** | `.CONVENTIONS.md` | ~45% | Conventions format, minimal structure |

### Tier 3: Low Fidelity (20-35% recovery)

| Format | File Pattern | Recovery Rate | Key Challenge |
|--------|--------------|---------------|---------------|
| **MCP** | `.mcp.json` | ~30% | Minimal schema, no instructions |
| **Codex** | `.codex.json` | ~25% | Severely simplified tool spec |

---

## Unified Intermediate Representation (UIR)

All parsers output to this common structure before canonical generation:

```rust
/// Unified Intermediate Representation for imported skills
pub struct ImportedSkill {
    // === METADATA (High Confidence) ===
    pub name: ImportedField<String>,
    pub version: ImportedField<Option<String>>,
    pub description: ImportedField<String>,
    pub author: ImportedField<Option<Author>>,

    // === DAEMONS (Medium Confidence) ===
    pub daemons: Vec<ImportedDaemon>,

    // === INSTRUCTIONS (High Confidence) ===
    pub instructions: ImportedInstructions,

    // === TRIGGERS (Medium Confidence) ===
    pub triggers: ImportedTriggers,

    // === WORKFLOWS (Low Confidence) ===
    pub workflows: Vec<ImportedWorkflow>,

    // === CONFIG (Low Confidence) ===
    pub config: Vec<ImportedConfigOption>,

    // === AUTH (Low Confidence) ===
    pub auth: ImportedAuth,

    // === SOURCE METADATA ===
    pub source_format: ImportFormat,
    pub source_path: PathBuf,
    pub import_timestamp: DateTime<Utc>,
}

/// Every imported field carries confidence metadata
pub struct ImportedField<T> {
    pub value: T,
    pub confidence: Confidence,
    pub source: FieldSource,
    pub notes: Option<String>,
}

pub enum Confidence {
    /// Directly extracted from structured data
    High,
    /// Inferred from patterns/context
    Medium,
    /// Guessed or placeholder
    Low,
    /// Could not determine, needs user input
    Unknown,
}

pub enum FieldSource {
    /// From YAML/JSON frontmatter
    Frontmatter,
    /// From markdown headers/content
    Content,
    /// From filename/path
    Filename,
    /// Inferred from method calls in text
    MethodExtraction,
    /// Looked up from daemon registry
    Registry,
    /// User-provided during import
    UserInput,
    /// Default/placeholder value
    Default,
}
```

---

## Format-Specific Parsers

### Claude Code SKILL.md Parser

**Input Structure:**
```markdown
---
name: gmail
description: Fast Gmail access via FGP daemon
tools:
  - gmail.inbox
  - gmail.unread
  - gmail.search
---

# Gmail Skill

Instructions here...

## Available Methods

| Method | Description |
|--------|-------------|
| `gmail.inbox` | List recent emails |
...
```

**Extraction Strategy:**
```rust
impl ClaudeCodeParser {
    fn parse(&self, content: &str) -> Result<ImportedSkill> {
        // 1. Extract YAML frontmatter (high confidence)
        let frontmatter = extract_yaml_frontmatter(content)?;

        // 2. Parse tool list → daemon methods
        let tools = frontmatter.get("tools")
            .map(|t| parse_tool_list(t))
            .unwrap_or_default();

        // 3. Extract daemon names from method prefixes
        let daemons = infer_daemons_from_methods(&tools);

        // 4. Parse markdown body for additional context
        let body = extract_body_after_frontmatter(content);

        // 5. Extract method tables if present
        let method_tables = extract_markdown_tables(&body, "Method");

        // 6. Look for trigger sections
        let triggers = extract_triggers_from_content(&body);

        Ok(ImportedSkill {
            name: ImportedField::high(frontmatter.name),
            description: ImportedField::high(frontmatter.description),
            daemons: daemons.into_iter()
                .map(|d| ImportedDaemon::medium(d))
                .collect(),
            instructions: ImportedInstructions {
                core: ImportedField::high(body.clone()),
                source_agent: ImportFormat::ClaudeCode,
            },
            // ...
        })
    }
}
```

**Recovery Capabilities:**

| Field | Recovery | Method |
|-------|----------|--------|
| `name` | ✅ High | YAML frontmatter |
| `description` | ✅ High | YAML frontmatter |
| `daemons[].name` | ✅ High | Tool list prefixes |
| `daemons[].methods` | ✅ High | Tool list |
| `daemons[].version` | ⚠️ Low | Must infer/default |
| `instructions.core` | ✅ High | Markdown body |
| `triggers.keywords` | ⚠️ Medium | Content analysis |
| `workflows` | ❌ None | Not in export |
| `config` | ❌ None | Not in export |
| `auth` | ⚠️ Low | Infer from Setup section |

### Cursor .cursorrules Parser

**Input Structure:**
```markdown
# FGP iMessage Gateway

Fast iMessage operations for macOS via FGP daemon.

## Quick Start

```bash
fgp-imessage recent --json
```

## Daemon Methods

| Method | Description | Params |
|--------|-------------|--------|
| recent | Recent messages | days, limit |
...
```

**Extraction Strategy:**
```rust
impl CursorParser {
    fn parse(&self, content: &str) -> Result<ImportedSkill> {
        // 1. Extract skill name from first H1 header
        let name = extract_first_h1(content)
            .map(|h| slugify(h))
            .ok_or("No H1 header found")?;

        // 2. Extract description from first paragraph
        let description = extract_first_paragraph(content);

        // 3. Find method tables and extract daemon info
        let method_tables = find_tables_with_header(content, "Method");
        let daemons = infer_daemons_from_tables(&method_tables);

        // 4. Extract code blocks for CLI patterns
        let code_blocks = extract_code_blocks(content, "bash");
        let cli_patterns = extract_fgp_calls(&code_blocks);

        Ok(ImportedSkill {
            name: ImportedField::medium(name),
            description: ImportedField::medium(description),
            daemons: daemons,
            instructions: ImportedInstructions {
                core: ImportedField::high(content.to_string()),
                source_agent: ImportFormat::Cursor,
            },
            // Lower confidence across the board
        })
    }
}
```

### JSON Format Parsers (MCP, Codex, Gemini)

**Common Pattern:**
```rust
impl JsonFormatParser {
    fn parse(&self, content: &str) -> Result<ImportedSkill> {
        let json: Value = serde_json::from_str(content)?;

        // Direct field mapping where available
        let name = json.get("name")
            .and_then(|v| v.as_str())
            .map(|s| ImportedField::high(s.to_string()))
            .ok_or("Missing name field")?;

        // Schema extraction for method info
        let tools = json.get("tools")
            .and_then(|v| v.as_array())
            .map(|arr| parse_tool_schemas(arr))
            .unwrap_or_default();

        // ...
    }
}
```

---

## Daemon Registry Integration

### Purpose
Enrich imported skills with full method schemas by looking up daemon definitions.

### Registry Structure
```rust
/// Daemon method registry for import enrichment
pub struct DaemonRegistry {
    /// Map of daemon_name -> DaemonDefinition
    daemons: HashMap<String, DaemonDefinition>,
}

pub struct DaemonDefinition {
    pub name: String,
    pub version: String,
    pub methods: Vec<MethodDefinition>,
}

pub struct MethodDefinition {
    pub name: String,
    pub description: String,
    pub schema: JsonSchema,
    pub returns: Option<JsonSchema>,
    pub examples: Vec<Example>,
}
```

### Enrichment Flow
```rust
impl Enricher {
    fn enrich(&self, skill: ImportedSkill, registry: &DaemonRegistry) -> EnrichedSkill {
        let mut enriched = skill.clone();

        for daemon in &mut enriched.daemons {
            if let Some(def) = registry.get(&daemon.name) {
                // Upgrade confidence for known daemons
                daemon.confidence = Confidence::High;

                // Fill in method schemas
                for method in &mut daemon.methods {
                    if let Some(method_def) = def.get_method(&method.name) {
                        method.schema = Some(method_def.schema.clone());
                        method.description = Some(method_def.description.clone());
                    }
                }

                // Add version constraint
                daemon.version = Some(format!(">={}", def.version));
            }
        }

        enriched
    }
}
```

### Registry Population

**Option A: Static Registry (MVP)**
```yaml
# registry/daemons.yaml
gmail:
  version: "1.0.0"
  methods:
    - name: inbox
      description: List recent emails from inbox
      schema:
        type: object
        properties:
          limit:
            type: integer
            default: 10
    - name: send
      # ...

browser:
  version: "1.0.0"
  methods:
    - name: open
    - name: snapshot
    # ...
```

**Option B: Dynamic Registry (Future)**
```rust
// Query running daemons for their schemas
async fn populate_registry() -> DaemonRegistry {
    let socket_dir = "~/.fgp/services/";
    let mut registry = DaemonRegistry::new();

    for entry in fs::read_dir(socket_dir)? {
        let daemon_name = entry.file_name();
        let socket_path = entry.path().join("daemon.sock");

        if socket_path.exists() {
            // Call the `schema` method on each daemon
            let schema_response = fgp_call(&socket_path, "schema", json!({})).await?;
            registry.add_daemon(daemon_name, schema_response);
        }
    }

    registry
}
```

---

## Canonical Output Generation

### Directory Structure
```
{skill_name}/
├── skill.yaml              # Canonical manifest
├── instructions/
│   ├── core.md             # Extracted from source
│   └── {source_agent}.md   # Copy of original
├── workflows/              # (empty or placeholders)
│   └── .gitkeep
└── IMPORT_REPORT.md        # Confidence report
```

### skill.yaml Generation

```rust
impl CanonicalGenerator {
    fn generate(&self, skill: &EnrichedSkill) -> String {
        let mut yaml = String::new();

        // Header comment with import metadata
        yaml.push_str(&format!(
            "# Imported from {} on {}\n",
            skill.source_format,
            skill.import_timestamp
        ));
        yaml.push_str("# Fields marked [*LOW-CONFIDENCE*] need review\n\n");

        // Core metadata
        yaml.push_str(&format!("name: {}\n", skill.name.value));
        yaml.push_str(&format!("version: {}\n",
            skill.version.value.as_deref().unwrap_or("1.0.0")));
        yaml.push_str(&format!("description: {}\n", skill.description.value));

        // Daemons section
        if !skill.daemons.is_empty() {
            yaml.push_str("\ndaemons:\n");
            for daemon in &skill.daemons {
                yaml.push_str(&format!("  - name: {}\n", daemon.name));
                if daemon.confidence < Confidence::High {
                    yaml.push_str("    # [*LOW-CONFIDENCE*] Verify daemon name\n");
                }
                if let Some(v) = &daemon.version {
                    yaml.push_str(&format!("    version: \"{}\"\n", v));
                }
                if !daemon.methods.is_empty() {
                    yaml.push_str("    methods:\n");
                    for method in &daemon.methods {
                        yaml.push_str(&format!("      - {}\n", method));
                    }
                }
            }
        }

        // Instructions (reference files)
        yaml.push_str("\ninstructions:\n");
        yaml.push_str("  core: ./instructions/core.md\n");
        yaml.push_str(&format!("  {}: ./instructions/{}.md\n",
            skill.source_format.to_key(),
            skill.source_format.to_key()));

        // Triggers (if extracted)
        if !skill.triggers.keywords.is_empty() {
            yaml.push_str("\ntriggers:\n");
            yaml.push_str("  keywords:\n");
            for kw in &skill.triggers.keywords {
                yaml.push_str(&format!("    - {}\n", kw.value));
            }
        }

        // Placeholders for unrecoverable sections
        yaml.push_str("\n# [*INCOMPLETE*] Workflows not recoverable from export\n");
        yaml.push_str("# workflows:\n");
        yaml.push_str("#   default:\n");
        yaml.push_str("#     file: ./workflows/main.yaml\n");

        yaml.push_str("\n# [*INCOMPLETE*] Config options not recoverable from export\n");
        yaml.push_str("# config: {}\n");

        yaml
    }
}
```

### Import Report Generation

```markdown
# Import Report: gmail

**Source:** SKILL.md (Claude Code format)
**Imported:** 01/15/2026 12:00 PM PST
**Overall Confidence:** 75%

## Field Recovery Summary

| Field | Confidence | Source | Notes |
|-------|------------|--------|-------|
| name | ✅ High | Frontmatter | Direct extraction |
| description | ✅ High | Frontmatter | Direct extraction |
| daemons | ✅ High | Tool list | 5 methods found |
| instructions | ✅ High | Body | 200 lines preserved |
| triggers | ⚠️ Medium | Inferred | From keywords in description |
| workflows | ❌ None | N/A | Not in export format |
| config | ❌ None | N/A | Not in export format |
| auth | ⚠️ Low | Setup section | Inferred OAuth requirement |

## Required User Actions

1. **Verify daemon version constraints** - Currently set to `>=1.0.0`
2. **Add workflow definitions** if this skill has multi-step operations
3. **Define config options** for user-customizable behavior
4. **Review auth requirements** - Detected OAuth, needs confirmation

## Unrecoverable Data

The following data cannot be recovered and must be manually added:

- Workflow YAML files (original workflow logic)
- JSON Schema definitions for method parameters
- Marketplace/distribution configuration
- Entitlements and licensing information
```

---

## CLI Interface

### Commands

```bash
# Import a single skill file
fgp skill import ./SKILL.md --output ./imported-gmail/

# Import with format hint (auto-detect by default)
fgp skill import ./rules.txt --format cursor --output ./imported-skill/

# Import with daemon registry lookup
fgp skill import ./SKILL.md --enrich --registry ~/.fgp/registry/

# Dry run (show what would be extracted)
fgp skill import ./SKILL.md --dry-run

# Batch import from directory
fgp skill import ./skills/ --batch --output ./canonical-skills/
```

### Output Example

```
$ fgp skill import ~/.claude/skills/gmail/SKILL.md

Detected format: Claude Code (SKILL.md)
Parsing... done

Extracted:
  ✅ name: gmail
  ✅ description: Fast Gmail access via FGP daemon (10x faster than MCP)
  ✅ daemons: gmail (5 methods)
  ⚠️ triggers: [email, gmail, inbox, send] (inferred)
  ❌ workflows: none (not in export)
  ❌ config: none (not in export)

Overall confidence: 75%

Writing to ./gmail/
  → skill.yaml
  → instructions/core.md
  → instructions/claude-code.md
  → IMPORT_REPORT.md

Done! Review IMPORT_REPORT.md for required actions.
```

---

## Implementation Phases

### Phase 1: MVP (Claude Code Only)
**Effort:** ~2-3 days

- [ ] YAML frontmatter parser for SKILL.md
- [ ] Basic markdown body extraction
- [ ] Method table parser
- [ ] skill.yaml generator with confidence markers
- [ ] Import report generator
- [ ] CLI `fgp skill import` command

**Deliverable:** Import Claude Code skills with ~80% fidelity

### Phase 2: Extended Format Support
**Effort:** ~3-4 days

- [ ] Cursor .cursorrules parser
- [ ] Zed .rules parser
- [ ] Windsurf .windsurf.md parser
- [ ] Aider .CONVENTIONS.md parser
- [ ] Gemini JSON+MD parser
- [ ] Unified parser interface

**Deliverable:** Import from 6 additional formats

### Phase 3: Registry Integration
**Effort:** ~2-3 days

- [ ] Static daemon registry YAML format
- [ ] Registry loader
- [ ] Enrichment pass with schema lookup
- [ ] Dynamic registry from running daemons

**Deliverable:** Full method schemas in imported skills

### Phase 4: Round-Trip Validation
**Effort:** ~1-2 days

- [ ] Import → Export → Diff tool
- [ ] Fidelity scoring
- [ ] Regression test suite
- [ ] CI integration

**Deliverable:** Confidence that import/export are consistent

### Phase 5: Advanced Features (Future)
**Effort:** TBD

- [ ] LLM-assisted field inference
- [ ] Interactive import wizard (TUI)
- [ ] Workflow reconstruction hints
- [ ] Bulk migration tooling

---

## Data Loss Matrix

### What CAN Be Recovered

| Data | Claude Code | Cursor | MCP | Codex | Gemini |
|------|-------------|--------|-----|-------|--------|
| name | ✅ | ⚠️ | ⚠️ | ⚠️ | ✅ |
| description | ✅ | ⚠️ | ⚠️ | ⚠️ | ✅ |
| daemon names | ✅ | ⚠️ | ⚠️ | ⚠️ | ⚠️ |
| method list | ✅ | ⚠️ | ✅ | ✅ | ⚠️ |
| instructions | ✅ | ✅ | ❌ | ❌ | ✅ |
| triggers | ⚠️ | ⚠️ | ❌ | ❌ | ⚠️ |

### What CANNOT Be Recovered (Any Format)

| Data | Why | Mitigation |
|------|-----|------------|
| Workflow YAML | Never exported | Placeholder + manual creation |
| Full JSON Schemas | Simplified/dropped | Registry lookup |
| Config options | Not exported | Placeholder |
| Auth secrets | Security | Placeholder |
| Marketplace config | Not relevant to agents | Omit |
| Exact version constraints | Dropped | Default to `>=1.0.0` |

---

## Security Considerations

1. **Input Validation**: All imported content must be sanitized
2. **Path Traversal**: Validate output paths don't escape target directory
3. **YAML Parsing**: Use safe YAML parser (no arbitrary code execution)
4. **Secrets Detection**: Warn if imported content contains potential secrets

---

## Success Metrics

1. **Fidelity Score**: % of fields correctly recovered (target: 80% for Claude Code)
2. **Round-Trip Loss**: Export → Import → Export diff should be minimal
3. **User Effort**: Time to complete import + manual fixups
4. **Format Coverage**: 8/8 export formats should be importable

---

## Open Questions

1. **Workflow Inference**: Should we attempt to infer workflow structure from instruction text?
2. **Schema Registry Location**: Central registry vs per-project vs dynamic lookup?
3. **Conflict Resolution**: How to handle importing a skill that already exists?
4. **Versioning**: Should imported skills get special version markers (e.g., `1.0.0-imported`)?

---

## References

- `/cli/src/commands/skill_export.rs` - Export implementation
- `/protocol/schemas/skill.schema.json` - Canonical schema
- `/protocol/FGP-SCHEMA-SPEC.md` - Schema specification
- `/protocol/skills/research-assistant/skill.yaml` - Example manifest

---

## Changelog

| Date | Author | Change |
|------|--------|--------|
| 01/15/2026 | Wolfgang + Claude | Initial design document |
