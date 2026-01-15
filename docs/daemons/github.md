# GitHub Daemon

GitHub operations via GraphQL and REST APIs.

## Installation

```bash
fgp install github
fgp start github
```

## Authentication

Set your GitHub token:

```bash
export GITHUB_TOKEN="ghp_..."
```

Or store in `~/.fgp/services/github/config.toml`.

## Methods

### github.issues

List repository issues.

```json
{
  "method": "github.issues",
  "params": {
    "repo": "owner/repo",
    "state": "open"
  }
}
```

### github.prs

List pull requests.

```json
{
  "method": "github.prs",
  "params": {
    "repo": "owner/repo",
    "state": "open"
  }
}
```

### github.repos

List user repositories.

```json
{
  "method": "github.repos",
  "params": {
    "user": "username"
  }
}
```

### github.create_issue

Create a new issue.

```json
{
  "method": "github.create_issue",
  "params": {
    "repo": "owner/repo",
    "title": "Bug report",
    "body": "Description..."
  }
}
```

## CLI Examples

```bash
# List issues
fgp call github issues --repo "fast-gateway-protocol/fgp"

# List PRs
fgp call github prs --repo "fast-gateway-protocol/browser"
```

## Status

**Beta** - Basic operations work.
