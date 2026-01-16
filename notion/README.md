# FGP Notion Daemon

Fast Notion pages, databases, and blocks via FGP protocol.

## Quick Start

```bash
# Set API key
export NOTION_API_KEY="secret_xxxxx"

# Start daemon
fgp-notion start

# Or quick commands (no daemon needed)
fgp-notion search "meeting notes"
fgp-notion page abc123
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
export NOTION_API_KEY="secret_xxxxx"
# or
export NOTION_TOKEN="secret_xxxxx"
```

Create an integration at: https://www.notion.so/my-integrations

**Important:** Share pages/databases with your integration for access.

### Config File

Create `~/.fgp/auth/notion/credentials.json`:

```json
{
  "api_key": "secret_xxxxx"
}
```

## CLI Commands

```bash
# Daemon management
fgp-notion start           # Start daemon (background)
fgp-notion start -f        # Start in foreground
fgp-notion stop            # Stop daemon
fgp-notion status          # Check daemon status

# Quick operations (no daemon)
fgp-notion me              # Get bot info
fgp-notion search "query"  # Search pages/databases
fgp-notion search "query" --filter page  # Search only pages
fgp-notion page <page_id>  # Get page metadata
fgp-notion blocks <page_id>          # Get page blocks
fgp-notion blocks <page_id> --recursive  # Include nested blocks
```

## FGP Methods

| Method | Description | Parameters |
|--------|-------------|------------|
| `notion.me` | Get bot/integration info | - |
| `notion.users` | List workspace users | - |
| `notion.search` | Search pages & databases | `query?`, `filter?` (page/database), `limit?` |
| `notion.page` | Get page by ID | `page_id` (required) |
| `notion.database` | Get database schema | `database_id` (required) |
| `notion.query_database` | Query database rows | `database_id`, `filter?`, `sorts?`, `limit?` |
| `notion.blocks` | Get page/block children | `block_id`, `recursive?` |
| `notion.create_page` | Create page in database | `database_id`, `properties`, `children?` |
| `notion.update_page` | Update page properties | `page_id`, `properties` |
| `notion.append_blocks` | Append blocks to page | `block_id`, `children[]` |
| `notion.comments` | Get comments | `block_id` |
| `notion.add_comment` | Add comment to page | `page_id`, `text` |

## Examples

### Search

```bash
# Via socket (with running daemon)
echo '{"id":"1","v":1,"method":"search","params":{"query":"meeting","limit":5}}' \
  | nc -U ~/.fgp/services/notion/daemon.sock
```

### Query Database

```json
{
  "method": "notion.query_database",
  "params": {
    "database_id": "abc123",
    "filter": {
      "property": "Status",
      "select": { "equals": "In Progress" }
    },
    "sorts": [
      { "property": "Created", "direction": "descending" }
    ],
    "limit": 10
  }
}
```

### Create Page

```json
{
  "method": "notion.create_page",
  "params": {
    "database_id": "abc123",
    "properties": {
      "Name": {
        "title": [{ "text": { "content": "New Task" } }]
      },
      "Status": {
        "select": { "name": "To Do" }
      }
    },
    "children": [
      {
        "type": "paragraph",
        "paragraph": {
          "rich_text": [{ "text": { "content": "Description here" } }]
        }
      }
    ]
  }
}
```

### Append Blocks

```json
{
  "method": "notion.append_blocks",
  "params": {
    "block_id": "page_id_here",
    "children": [
      {
        "type": "heading_2",
        "heading_2": {
          "rich_text": [{ "text": { "content": "New Section" } }]
        }
      },
      {
        "type": "paragraph",
        "paragraph": {
          "rich_text": [{ "text": { "content": "Content goes here" } }]
        }
      }
    ]
  }
}
```

## Block Types

Common block types you can create:

| Type | Description |
|------|-------------|
| `paragraph` | Plain text |
| `heading_1` | Large heading |
| `heading_2` | Medium heading |
| `heading_3` | Small heading |
| `bulleted_list_item` | Bullet point |
| `numbered_list_item` | Numbered item |
| `to_do` | Checkbox item |
| `code` | Code block |
| `quote` | Blockquote |
| `divider` | Horizontal rule |
| `callout` | Callout box |

## Socket Location

Default: `~/.fgp/services/notion/daemon.sock`

## Integration Setup

1. Go to https://www.notion.so/my-integrations
2. Create a new integration
3. Copy the "Internal Integration Token" (starts with `secret_`)
4. **Important:** Share pages/databases with your integration:
   - Open the page in Notion
   - Click "..." menu → "Add connections"
   - Select your integration

## Troubleshooting

### "Could not find object with ID"

The page/database exists but isn't shared with your integration. Add the integration as a connection.

### Rate Limiting

Notion has a rate limit of 3 requests per second. The daemon handles this gracefully but may return errors under heavy load.

### "Invalid request URL"

Check that your page/database ID is correct. IDs can be with or without dashes.
