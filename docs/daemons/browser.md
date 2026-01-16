# Browser Daemon

Fast browser automation using Chrome DevTools Protocol directly. **3-17x faster** than Playwright MCP (warm to warm: 3-12x, cold start elimination: 17x).

## Installation

```bash
fgp install browser
fgp start browser
```

## Methods

### Navigation

#### browser.open

Navigate to a URL.

```json
{
  "method": "browser.open",
  "params": {
    "url": "https://example.com",
    "session": "optional-session-name"
  }
}
```

**Response:**
```json
{
  "result": {
    "title": "Example Domain",
    "url": "https://example.com/"
  }
}
```

### Page Inspection

#### browser.snapshot

Get the ARIA accessibility tree (structured page content).

```json
{
  "method": "browser.snapshot",
  "params": {
    "session": "optional"
  }
}
```

**Response:**
```json
{
  "result": {
    "aria_tree": "document \"Example Domain\"\n  heading \"Example Domain\"\n  link \"More information...\""
  }
}
```

#### browser.screenshot

Capture a PNG screenshot.

```json
{
  "method": "browser.screenshot",
  "params": {
    "path": "/tmp/screenshot.png",
    "session": "optional"
  }
}
```

### Element Interaction

#### browser.click

Click an element.

```json
{
  "method": "browser.click",
  "params": {
    "selector": "button.submit"
  }
}
```

#### browser.fill

Fill a text input.

```json
{
  "method": "browser.fill",
  "params": {
    "selector": "input#email",
    "value": "user@example.com"
  }
}
```

#### browser.press

Press a keyboard key.

```json
{
  "method": "browser.press",
  "params": {
    "key": "Enter"
  }
}
```

#### browser.select

Select a dropdown option.

```json
{
  "method": "browser.select",
  "params": {
    "selector": "select#country",
    "value": "US"
  }
}
```

#### browser.check

Check or uncheck a checkbox.

```json
{
  "method": "browser.check",
  "params": {
    "selector": "input[type=checkbox]",
    "checked": true
  }
}
```

#### browser.hover

Hover over an element.

```json
{
  "method": "browser.hover",
  "params": {
    "selector": ".menu-item"
  }
}
```

#### browser.scroll

Scroll the page or an element.

```json
{
  "method": "browser.scroll",
  "params": {
    "direction": "down",
    "amount": 500
  }
}
```

#### browser.press_combo

Press a key combination.

```json
{
  "method": "browser.press_combo",
  "params": {
    "key": "a",
    "modifiers": ["ctrl"]
  }
}
```

#### browser.upload

Upload a file.

```json
{
  "method": "browser.upload",
  "params": {
    "selector": "input[type=file]",
    "path": "/path/to/file.pdf"
  }
}
```

### Session Management

#### session.new

Create an isolated browser session.

```json
{
  "method": "session.new",
  "params": {
    "name": "my-session"
  }
}
```

#### session.list

List active sessions.

```json
{
  "method": "session.list",
  "params": {}
}
```

#### session.close

Close a session.

```json
{
  "method": "session.close",
  "params": {
    "name": "my-session"
  }
}
```

## CLI Examples

```bash
# Navigate
fgp call browser open "https://example.com"

# Get page content
fgp call browser snapshot

# Interact
fgp call browser fill "#search" "hello"
fgp call browser click "button[type=submit]"

# Screenshot
fgp call browser screenshot /tmp/page.png
```

## Performance

| Operation | FGP | MCP (warm) | MCP (cold) | Speedup |
|-----------|-----|------------|------------|---------|
| Navigate | 2-8ms | 27-29ms | ~1,900ms | 3-12x warm, 17x cold |
| Snapshot | 0.7-9ms | 2-3ms | ~1,000ms | 3x warm, 11x cold |
| Click | 2-3ms | 5-10ms | ~500ms | 2-5x warm |
| Fill | 3-4ms | 8-12ms | ~500ms | 2-3x warm |
| Screenshot | 25-30ms | 50-80ms | ~1,600ms | 2x warm |

> **Note:** "Cold" = first call when MCP spawns new process. "Warm" = MCP server already running. FGP daemon is always warm.
