# FGP Daemon Generator

## Doctrine

See [DOCTRINE.md](./DOCTRINE.md).


Scaffolds new FGP daemons from templates with minimal effort.

## Quick Start

```bash
# List all 67 available presets
python generate.py --list-presets

# Generate a Slack daemon with preset config
python generate.py slack --preset

# Generate a custom daemon
python generate.py myservice --display-name "My Service" --api-url "https://api.example.com"
```

## Usage

```
python generate.py <service_name> [options]

Arguments:
  service_name          Name of the service (e.g., slack, linear, notion)

Options:
  --list-presets        List all available service presets
  --display-name        Human-readable display name (e.g., 'Slack')
  --api-url             Base URL for the API
  --env-token           Environment variable name for API token
  --author              Author name for changelog entries (default: Claude)
  --output-dir, -o      Output directory (default: current directory)
  --preset              Use preset configuration for known services
```

## Preset Categories (67 services)

### Communication (4)
`slack`, `discord`, `telegram`, `teams`

### Project Management (7)
`linear`, `jira`, `asana`, `trello`, `monday`, `clickup`, `height`

### Knowledge & Documentation (4)
`notion`, `confluence`, `coda`, `gitbook`

### Task Management (3)
`todoist`, `ticktick`, `things`

### Design & Creative (3)
`figma`, `canva`, `miro`

### AI & Search (5)
`exa`, `perplexity`, `tavily`, `brave_search`, `serper`

### Payments & Finance (3)
`stripe`, `plaid`, `mercury`

### CRM & Sales (4)
`hubspot`, `salesforce`, `pipedrive`, `close`

### Data & Databases (4)
`airtable`, `supabase`, `mongodb_atlas`, `planetscale`

### Cloud & DevOps (5)
`cloudflare`, `digitalocean`, `railway`, `render`, `netlify`

### Monitoring & Analytics (5)
`sentry`, `datadog`, `posthog`, `mixpanel`, `amplitude`

### Content & Media (4)
`youtube`, `spotify`, `twitter`, `reddit`

### Automation & Integration (3)
`zapier`, `make`, `n8n`

### Email (4)
`sendgrid`, `resend`, `mailgun`, `postmark`

### Customer Support (3)
`intercom`, `zendesk`, `freshdesk`

### HR & Recruiting (3)
`greenhouse`, `lever`, `rippling`

### Storage & Files (3)
`dropbox`, `box`, `google_drive`

## Generated Structure

```
<service_name>/
├── Cargo.toml           # Package config with FGP SDK dependency
├── .gitignore
└── src/
    ├── main.rs          # CLI with start/stop/status commands
    ├── service.rs       # FgpService implementation
    ├── models.rs        # Request/response data types
    └── api/
        ├── mod.rs       # Module exports
        └── client.rs    # HTTP client for the API
```

## After Generation

1. **Update `src/api/client.rs`** with actual API endpoints:
   - Implement `ping()` for health checks
   - Add domain-specific methods

2. **Update `src/models.rs`** with actual data types:
   - Define request/response structures matching the API

3. **Update `src/service.rs`**:
   - Add methods to `dispatch()` match statement
   - Update `method_list()` for documentation

4. **Build and test**:
   ```bash
   cargo build --release
   ./target/release/fgp-<service> start -f
   ```

## Template Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `{{service_name}}` | Snake_case service name | `slack` |
| `{{service_struct}}` | PascalCase for struct names | `Slack` |
| `{{display_name}}` | Human-readable name | `Slack` |
| `{{api_base_url}}` | API base URL | `https://slack.com/api` |
| `{{env_token}}` | Token env var name | `SLACK_TOKEN` |
| `{{author}}` | Author for changelogs | `Claude` |
| `{{date}}` | Current date | `01/14/2026` |

## Examples

### Generate from preset

```bash
python generate.py slack --preset
python generate.py linear --preset
python generate.py stripe --preset
```

### Generate custom daemon

```bash
python generate.py myapi \
  --display-name "My Custom API" \
  --api-url "https://api.myservice.com/v2" \
  --env-token "MYAPI_SECRET_KEY" \
  --author "Your Name"
```

### Workflow example

```bash
# 1. Generate daemon scaffold
python generate.py slack --preset

# 2. Navigate to daemon directory
cd slack

# 3. Implement API methods in src/api/client.rs
# 4. Define models in src/models.rs
# 5. Wire up service in src/service.rs

# 6. Build
cargo build --release

# 7. Run in foreground for testing
./target/release/fgp-slack start -f

# 8. Test via FGP CLI
fgp call slack.list '{"limit": 10}'
```
