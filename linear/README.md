# FGP Linear Daemon

Fast Linear issue tracking operations via FGP protocol.

## Quick Start

```bash
# Set API key
export LINEAR_API_KEY="lin_api_xxxxx"

# Start daemon
fgp-linear start

# Or quick commands (no daemon needed)
fgp-linear me
fgp-linear issues --team ENG
```

## Installation

```bash
cargo install --path .

# Or build from source
cargo build --release
```

## Authentication

### Environment Variable (recommended)

```bash
export LINEAR_API_KEY="lin_api_xxxxx"
```

Get your API key from: https://linear.app/settings/api

### Config File

Create `~/.fgp/auth/linear/credentials.json`:

```json
{
  "api_key": "lin_api_xxxxx"
}
```

## CLI Commands

```bash
# Daemon management
fgp-linear start           # Start daemon (background)
fgp-linear start -f        # Start in foreground
fgp-linear stop            # Stop daemon
fgp-linear status          # Check daemon status

# Quick operations (no daemon)
fgp-linear me              # Get current user
fgp-linear teams           # List teams
fgp-linear issues          # List issues
fgp-linear issues --team ENG --state "In Progress"
fgp-linear search "bug login"
```

## FGP Methods

| Method | Description | Parameters |
|--------|-------------|------------|
| `linear.me` | Get current user info | - |
| `linear.teams` | List all teams | - |
| `linear.issues` | List issues | `team?`, `state?`, `assignee?`, `limit?` |
| `linear.issue` | Get single issue | `id` (required) |
| `linear.create_issue` | Create issue | `team_id`, `title` (required), `description?`, `priority?`, `assignee_id?` |
| `linear.update_issue` | Update issue | `id` (required), `title?`, `state_id?`, `priority?`, `assignee_id?` |
| `linear.comments` | Get issue comments | `issue_id` (required) |
| `linear.add_comment` | Add comment | `issue_id`, `body` (required) |
| `linear.projects` | List projects | `team?`, `limit?` |
| `linear.cycles` | List cycles/sprints | `team?`, `limit?` |
| `linear.search` | Search issues | `query` (required), `limit?` |
| `linear.states` | Get workflow states | `team_key` (required) |

## Examples

### List Issues

```bash
# Via socket (with running daemon)
echo '{"id":"1","v":1,"method":"issues","params":{"team":"ENG","limit":5}}' \
  | nc -U ~/.fgp/services/linear/daemon.sock
```

### Create Issue

```json
{
  "method": "linear.create_issue",
  "params": {
    "team_id": "abc123",
    "title": "Fix login bug",
    "description": "Users can't login with SSO",
    "priority": 1
  }
}
```

### Search Issues

```json
{
  "method": "linear.search",
  "params": {
    "query": "bug authentication",
    "limit": 10
  }
}
```

## Priority Levels

| Value | Label |
|-------|-------|
| 0 | No priority |
| 1 | Urgent |
| 2 | High |
| 3 | Normal |
| 4 | Low |

## Socket Location

Default: `~/.fgp/services/linear/daemon.sock`

## Performance

| Operation | Without FGP | With FGP |
|-----------|-------------|----------|
| List issues | ~500-800ms | ~50-150ms |
| Create issue | ~300-500ms | ~200-400ms |
| Search | ~400-600ms | ~100-200ms |

## Troubleshooting

### Invalid API Key

```
Error: GraphQL errors: Invalid API key
```

Verify your API key at https://linear.app/settings/api

### Rate Limiting

Linear has generous rate limits (10,000 requests/hour). If you hit limits, the daemon will return an error - implement backoff in your client.
