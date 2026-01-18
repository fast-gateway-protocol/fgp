# Draft: r/ClaudeCode Post

---

**Title options (pick one):**

1. I replaced my MCP servers with persistent daemons - here's what I learned about latency
2. Built a daemon-based alternative to MCP for Claude Code - cold starts were killing my workflow
3. [Open Source] FGP: Persistent daemons instead of MCP stdio servers - benchmarks inside

---

**Post:**

After months of MCP frustration (OAuth nightmares, slow cold starts, 90+ tools bloating my context), I built something different: **FGP (Fast Gateway Protocol)** - persistent UNIX socket daemons that stay warm across Claude Code sessions.

## The Problem

Every time Claude Code spawns an MCP server, you pay a cold start tax:
- Playwright MCP: ~1.1s cold start
- Gmail MCP: ~1.6s cold start
- Node-based servers: 200-500ms minimum

For a single tool call, whatever. But Claude Code makes dozens of sequential calls per session. That adds up.

## The Approach

Instead of spawning a new process per session, FGP daemons:
- Start once, stay running in the background
- Communicate over UNIX sockets (no TCP overhead)
- Use NDJSON protocol (human-readable, debuggable)
- Expose as CLI tools that Claude Code can call directly

```bash
# Instead of MCP tool calls through JSON-RPC...
browser-gateway open "https://example.com"
browser-gateway snapshot  # Returns ARIA tree
browser-gateway click "button#submit"

# Or Gmail
gmail-gateway inbox --limit 10
gmail-gateway send --to "..." --subject "..." --body "..."
```

## Honest Benchmarks

I'm not going to claim "1000x faster" - here's what's actually true:

**Cold start elimination (real savings):**
| Service | MCP Cold Start | FGP | Saved |
|---------|----------------|-----|-------|
| Browser (Playwright) | 1,080ms | 0ms | 1.1s |
| Gmail | 1,597ms | 0ms | 1.6s |

**Warm-to-warm (network APIs):**
| Operation | MCP Warm | FGP Warm | Speedup |
|-----------|----------|----------|---------|
| Gmail fetch | 145ms | 142ms | ~1x |

Network latency dominates for API calls. FGP doesn't magically make the internet faster.

**Local operations (where it shines):**
| Operation | Subprocess | FGP Daemon | Speedup |
|-----------|------------|------------|---------|
| iMessage recent | 80ms | 5ms | 16x |
| Screen Time query | 5ms | 0.32ms | 15.7x |
| Browser ARIA snapshot | 2.6ms | 0.7-3ms | ~3x |

## What's Available

Built so far:
- **browser** - Chrome DevTools Protocol (not Playwright)
- **gmail** / **calendar** - Google APIs
- **github** - GraphQL + REST
- **imessage** / **contacts** / **photos** - macOS native (SQLite + AppleScript)
- **screen-time** - macOS knowledgeC.db
- **cloudflare** / **discord** / **youtube** / **supabase** / **composio**

All Rust, ~10MB memory per daemon.

## Claude Code Integration

I use these as CLI tools via skills. Example SKILL.md:

```yaml
---
name: browser-fgp
description: Fast browser automation. Use for navigation, screenshots, form filling.
---

# Browser Gateway

\`\`\`bash
browser-gateway open "https://..."
browser-gateway snapshot
browser-gateway fill "input#email" "test@example.com"
browser-gateway click "button[type=submit]"
\`\`\`
```

Claude Code picks these up and uses them like any other CLI tool.

## Source

Everything's open source:
- **GitHub org:** https://github.com/fast-gateway-protocol
- **Main repo:** https://github.com/fast-gateway-protocol/daemon (Rust SDK)
- **Skills:** https://github.com/fast-gateway-protocol/fgp-skills

Would love feedback. Especially interested in:
1. What services would you want daemons for?
2. Anyone else frustrated with MCP cold starts?
3. Ideas for better Claude Code integration patterns?

---

**Suggested flair:** [Open Source] or [Project]

---

## Notes for posting:

- Post during US morning (9-11am PT) for best visibility
- Cross-post to r/ClaudeAI after 24h if it gets traction
- Reply to comments quickly in first 2 hours
- If someone asks about MCP compatibility, mention you're considering an MCP-over-FGP bridge
