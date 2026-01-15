# Gmail Daemon

Fast Gmail operations via Google API.

## Installation

```bash
fgp install gmail
fgp start gmail
```

## Authentication

The Gmail daemon uses OAuth2. On first run, you'll be prompted to authorize access.

Credentials are stored in `~/.fgp/services/gmail/credentials.json`.

## Methods

### gmail.list

List emails from inbox.

```json
{
  "method": "gmail.list",
  "params": {
    "max_results": 10,
    "label": "INBOX"
  }
}
```

### gmail.read

Read a specific email.

```json
{
  "method": "gmail.read",
  "params": {
    "message_id": "abc123"
  }
}
```

### gmail.send

Send an email.

```json
{
  "method": "gmail.send",
  "params": {
    "to": "recipient@example.com",
    "subject": "Hello",
    "body": "Message content"
  }
}
```

### gmail.search

Search emails.

```json
{
  "method": "gmail.search",
  "params": {
    "query": "from:boss@company.com is:unread"
  }
}
```

## CLI Examples

```bash
# List recent emails
fgp call gmail list

# Search
fgp call gmail search "is:unread"

# Send
fgp call gmail send --to "user@example.com" --subject "Hi" --body "Hello!"
```

## Status

**Beta** - Core operations work, advanced features in progress.
