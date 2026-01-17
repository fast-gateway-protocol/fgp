#!/usr/bin/env python3
"""
FGP Daemon Generator

Scaffolds a new FGP daemon from templates.

Usage:
    python generate.py <service_name> [options]

Examples:
    python generate.py slack --display-name "Slack" --api-url "https://slack.com/api"
    python generate.py linear --display-name "Linear" --api-url "https://api.linear.app"
    python generate.py notion --display-name "Notion" --api-url "https://api.notion.com/v1"
"""

import argparse
import os
import re
import sys
from datetime import datetime
from pathlib import Path


def to_pascal_case(name: str) -> str:
    """Convert snake_case or kebab-case to PascalCase."""
    # Replace hyphens with underscores, then capitalize each word
    parts = re.split(r'[-_]', name)
    return ''.join(word.capitalize() for word in parts)


def to_snake_case(name: str) -> str:
    """Convert to snake_case."""
    # Replace hyphens with underscores
    return name.replace('-', '_').lower()


def get_date() -> str:
    """Get current date in MM/DD/YYYY format."""
    return datetime.now().strftime("%m/%d/%Y")


def render_template(template: str, context: dict) -> str:
    """Simple template rendering with {{variable}} syntax."""
    result = template
    for key, value in context.items():
        result = result.replace(f"{{{{{key}}}}}", str(value))
    return result


def print_presets(presets: dict) -> None:
    """Print all available presets grouped by category."""
    # Group by category
    by_category = {}
    for name, info in presets.items():
        category = info.get("category", "Other")
        if category not in by_category:
            by_category[category] = []
        by_category[category].append((name, info))

    # Sort categories and services within each
    category_order = [
        "Communication",
        "Project Management",
        "Knowledge",
        "Tasks",
        "Productivity",
        "Design",
        "Dev Tools",
        "AI & Search",
        "AI Tooling",
        "Data",
        "Data Infrastructure",
        "Cloud",
        "Monitoring",
        "Sales",
        "CRM",
        "Payments",
        "Fintech",
        "E-commerce",
        "Media",
        "Automation",
        "Email",
        "Support",
        "HR",
        "Storage",
        "Security",
    ]

    print()
    print("=" * 70)
    print(f"FGP Daemon Generator - {len(presets)} Service Presets Available")
    print("=" * 70)
    print()

    for category in category_order:
        if category not in by_category:
            continue

        services = sorted(by_category[category], key=lambda x: x[0])
        print(f"{category}")
        print("-" * len(category))

        for name, info in services:
            desc = info.get("description", "")
            env = info.get("env_token", "")
            print(f"  {name:20} {desc:35} [{env}]")

        print()

    # Print usage hint
    print("Usage:")
    print("  python generate.py <service> --preset")
    print()
    print("Example:")
    print("  python generate.py slack --preset")
    print("  python generate.py linear --preset")
    print()


def create_daemon(
    service_name: str,
    display_name: str,
    api_base_url: str,
    env_token: str,
    author: str,
    output_dir: Path,
):
    """Create a new daemon from templates."""

    # Normalize names
    service_name = to_snake_case(service_name)
    service_struct = to_pascal_case(service_name)

    # Build context for template rendering
    context = {
        "service_name": service_name,
        "service_struct": service_struct,
        "display_name": display_name,
        "api_base_url": api_base_url,
        "env_token": env_token,
        "author": author,
        "date": get_date(),
    }

    # Get template directory
    script_dir = Path(__file__).parent
    template_dir = script_dir / "templates"

    # Create output directory
    daemon_dir = output_dir / service_name
    daemon_dir.mkdir(parents=True, exist_ok=True)

    # Template files to process
    templates = [
        ("Cargo.toml.template", "Cargo.toml"),
        ("main.rs.template", "src/main.rs"),
        ("service.rs.template", "src/service.rs"),
        ("models.rs.template", "src/models.rs"),
        ("api/mod.rs.template", "src/api/mod.rs"),
        ("api/client.rs.template", "src/api/client.rs"),
    ]

    for template_name, output_name in templates:
        template_path = template_dir / template_name
        output_path = daemon_dir / output_name

        if not template_path.exists():
            print(f"Warning: Template not found: {template_path}")
            continue

        # Read template
        with open(template_path, 'r') as f:
            template_content = f.read()

        # Render template
        rendered = render_template(template_content, context)

        # Create output directory if needed
        output_path.parent.mkdir(parents=True, exist_ok=True)

        # Write output
        with open(output_path, 'w') as f:
            f.write(rendered)

        print(f"  Created: {output_path.relative_to(output_dir)}")

    # Create .gitignore
    gitignore_path = daemon_dir / ".gitignore"
    with open(gitignore_path, 'w') as f:
        f.write("/target\n")
    print(f"  Created: {gitignore_path.relative_to(output_dir)}")

    # Print next steps
    print()
    print("=" * 60)
    print(f"Daemon '{service_name}' created successfully!")
    print("=" * 60)
    print()
    print("Next steps:")
    print()
    print(f"  1. cd {daemon_dir}")
    print()
    print("  2. Update src/api/client.rs with actual API endpoints")
    print("     - Implement ping() for health check")
    print("     - Add domain-specific methods (list, get, create, etc.)")
    print()
    print("  3. Update src/models.rs with actual data types")
    print("     - Define request/response structures")
    print()
    print("  4. Update src/service.rs with method implementations")
    print("     - Add methods to dispatch()")
    print("     - Update method_list()")
    print()
    print("  5. Build and test:")
    print(f"     cargo build --release")
    print(f"     ./target/release/fgp-{service_name} start -f")
    print()
    print(f"  6. Set {env_token} environment variable")
    print()


def main():
    parser = argparse.ArgumentParser(
        description="Generate a new FGP daemon from templates",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s slack --display-name "Slack" --api-url "https://slack.com/api"
  %(prog)s linear --display-name "Linear" --api-url "https://api.linear.app"
  %(prog)s notion --display-name "Notion" --api-url "https://api.notion.com/v1"
  %(prog)s stripe --display-name "Stripe" --api-url "https://api.stripe.com/v1"
  %(prog)s jira --display-name "Jira" --api-url "https://your-domain.atlassian.net/rest/api/3"

Service presets (auto-configures API URL and token env var):
  %(prog)s slack --preset          # Team communication
  %(prog)s linear --preset         # Issue tracking
  %(prog)s notion --preset         # Knowledge base
  %(prog)s stripe --preset         # Payments
  %(prog)s todoist --preset        # Task management
  %(prog)s exa --preset            # AI search
  %(prog)s confluence --preset     # Documentation
  %(prog)s figma --preset          # Design
  %(prog)s airtable --preset       # Database
  %(prog)s asana --preset          # Project management

List all presets:
  %(prog)s --list-presets
        """,
    )

    parser.add_argument(
        "service_name",
        nargs="?",  # Make optional for --list-presets
        help="Name of the service (e.g., slack, linear, notion)",
    )

    parser.add_argument(
        "--list-presets",
        action="store_true",
        help="List all available service presets",
    )

    parser.add_argument(
        "--display-name",
        dest="display_name",
        help="Human-readable display name (e.g., 'Slack', 'Linear')",
    )

    parser.add_argument(
        "--api-url",
        dest="api_url",
        help="Base URL for the API (e.g., 'https://api.linear.app')",
    )

    parser.add_argument(
        "--env-token",
        dest="env_token",
        help="Environment variable name for API token (e.g., 'SLACK_TOKEN')",
    )

    parser.add_argument(
        "--author",
        default="Claude",
        help="Author name for changelog entries (default: Claude)",
    )

    parser.add_argument(
        "--output-dir", "-o",
        dest="output_dir",
        default=".",
        help="Output directory (default: current directory)",
    )

    parser.add_argument(
        "--preset",
        action="store_true",
        help="Use preset configuration for known services",
    )

    args = parser.parse_args()

    # Known service presets organized by category
    presets = {
        # =====================================================================
        # Communication & Collaboration
        # =====================================================================
        "slack": {
            "display_name": "Slack",
            "api_url": "https://slack.com/api",
            "env_token": "SLACK_TOKEN",
            "category": "Communication",
            "description": "Team messaging and collaboration",
        },
        "discord": {
            "display_name": "Discord",
            "api_url": "https://discord.com/api/v10",
            "env_token": "DISCORD_BOT_TOKEN",
            "category": "Communication",
            "description": "Community chat platform",
        },
        "telegram": {
            "display_name": "Telegram",
            "api_url": "https://api.telegram.org/bot",
            "env_token": "TELEGRAM_BOT_TOKEN",
            "category": "Communication",
            "description": "Messaging platform",
        },
        "teams": {
            "display_name": "Microsoft Teams",
            "api_url": "https://graph.microsoft.com/v1.0",
            "env_token": "MICROSOFT_GRAPH_TOKEN",
            "category": "Communication",
            "description": "Microsoft Teams integration",
        },

        # =====================================================================
        # Project Management & Issue Tracking
        # =====================================================================
        "linear": {
            "display_name": "Linear",
            "api_url": "https://api.linear.app",
            "env_token": "LINEAR_API_KEY",
            "category": "Project Management",
            "description": "Modern issue tracking",
        },
        "jira": {
            "display_name": "Jira",
            "api_url": "https://your-domain.atlassian.net/rest/api/3",
            "env_token": "JIRA_API_TOKEN",
            "category": "Project Management",
            "description": "Atlassian issue tracking",
        },
        "asana": {
            "display_name": "Asana",
            "api_url": "https://app.asana.com/api/1.0",
            "env_token": "ASANA_ACCESS_TOKEN",
            "category": "Project Management",
            "description": "Work management platform",
        },
        "trello": {
            "display_name": "Trello",
            "api_url": "https://api.trello.com/1",
            "env_token": "TRELLO_API_KEY",
            "category": "Project Management",
            "description": "Kanban-style boards",
        },
        "monday": {
            "display_name": "Monday.com",
            "api_url": "https://api.monday.com/v2",
            "env_token": "MONDAY_API_TOKEN",
            "category": "Project Management",
            "description": "Work OS platform",
        },
        "clickup": {
            "display_name": "ClickUp",
            "api_url": "https://api.clickup.com/api/v2",
            "env_token": "CLICKUP_API_TOKEN",
            "category": "Project Management",
            "description": "Productivity platform",
        },
        "height": {
            "display_name": "Height",
            "api_url": "https://api.height.app",
            "env_token": "HEIGHT_API_KEY",
            "category": "Project Management",
            "description": "Autonomous project management",
        },

        # =====================================================================
        # Knowledge & Documentation
        # =====================================================================
        "notion": {
            "display_name": "Notion",
            "api_url": "https://api.notion.com/v1",
            "env_token": "NOTION_TOKEN",
            "category": "Knowledge",
            "description": "All-in-one workspace",
        },
        "confluence": {
            "display_name": "Confluence",
            "api_url": "https://your-domain.atlassian.net/wiki/rest/api",
            "env_token": "CONFLUENCE_API_TOKEN",
            "category": "Knowledge",
            "description": "Team documentation",
        },
        "coda": {
            "display_name": "Coda",
            "api_url": "https://coda.io/apis/v1",
            "env_token": "CODA_API_TOKEN",
            "category": "Knowledge",
            "description": "All-in-one doc",
        },
        "gitbook": {
            "display_name": "GitBook",
            "api_url": "https://api.gitbook.com/v1",
            "env_token": "GITBOOK_API_TOKEN",
            "category": "Knowledge",
            "description": "Documentation platform",
        },

        # =====================================================================
        # Task Management
        # =====================================================================
        "todoist": {
            "display_name": "Todoist",
            "api_url": "https://api.todoist.com/rest/v2",
            "env_token": "TODOIST_API_TOKEN",
            "category": "Tasks",
            "description": "Personal task management",
        },
        "ticktick": {
            "display_name": "TickTick",
            "api_url": "https://api.ticktick.com/open/v1",
            "env_token": "TICKTICK_ACCESS_TOKEN",
            "category": "Tasks",
            "description": "Todo list and habit tracker",
        },
        "things": {
            "display_name": "Things",
            "api_url": "things:///",
            "env_token": "THINGS_AUTH_TOKEN",
            "category": "Tasks",
            "description": "macOS/iOS task manager (URL scheme)",
        },

        # =====================================================================
        # Design & Creative
        # =====================================================================
        "figma": {
            "display_name": "Figma",
            "api_url": "https://api.figma.com/v1",
            "env_token": "FIGMA_ACCESS_TOKEN",
            "category": "Design",
            "description": "Collaborative design tool",
        },
        "canva": {
            "display_name": "Canva",
            "api_url": "https://api.canva.com/rest/v1",
            "env_token": "CANVA_ACCESS_TOKEN",
            "category": "Design",
            "description": "Visual design platform",
        },
        "miro": {
            "display_name": "Miro",
            "api_url": "https://api.miro.com/v2",
            "env_token": "MIRO_ACCESS_TOKEN",
            "category": "Design",
            "description": "Online whiteboard",
        },

        # =====================================================================
        # AI & Search
        # =====================================================================
        "exa": {
            "display_name": "Exa Search",
            "api_url": "https://api.exa.ai",
            "env_token": "EXA_API_KEY",
            "category": "AI & Search",
            "description": "AI-native search engine",
        },
        "perplexity": {
            "display_name": "Perplexity",
            "api_url": "https://api.perplexity.ai",
            "env_token": "PERPLEXITY_API_KEY",
            "category": "AI & Search",
            "description": "AI search and reasoning",
        },
        "tavily": {
            "display_name": "Tavily",
            "api_url": "https://api.tavily.com",
            "env_token": "TAVILY_API_KEY",
            "category": "AI & Search",
            "description": "AI search for agents",
        },
        "brave_search": {
            "display_name": "Brave Search",
            "api_url": "https://api.search.brave.com/res/v1",
            "env_token": "BRAVE_SEARCH_API_KEY",
            "category": "AI & Search",
            "description": "Privacy-focused search",
        },
        "serper": {
            "display_name": "Serper",
            "api_url": "https://google.serper.dev",
            "env_token": "SERPER_API_KEY",
            "category": "AI & Search",
            "description": "Google search API",
        },

        # =====================================================================
        # Payments & Finance
        # =====================================================================
        "stripe": {
            "display_name": "Stripe",
            "api_url": "https://api.stripe.com/v1",
            "env_token": "STRIPE_API_KEY",
            "category": "Payments",
            "description": "Payment processing",
        },
        "plaid": {
            "display_name": "Plaid",
            "api_url": "https://production.plaid.com",
            "env_token": "PLAID_SECRET",
            "category": "Payments",
            "description": "Banking data aggregation",
        },
        "mercury": {
            "display_name": "Mercury",
            "api_url": "https://api.mercury.com/api/v1",
            "env_token": "MERCURY_API_TOKEN",
            "category": "Payments",
            "description": "Business banking",
        },

        # =====================================================================
        # CRM & Sales
        # =====================================================================
        "hubspot": {
            "display_name": "HubSpot",
            "api_url": "https://api.hubapi.com",
            "env_token": "HUBSPOT_ACCESS_TOKEN",
            "category": "CRM",
            "description": "CRM and marketing platform",
        },
        "salesforce": {
            "display_name": "Salesforce",
            "api_url": "https://your-instance.salesforce.com/services/data/v59.0",
            "env_token": "SALESFORCE_ACCESS_TOKEN",
            "category": "CRM",
            "description": "Enterprise CRM",
        },
        "pipedrive": {
            "display_name": "Pipedrive",
            "api_url": "https://api.pipedrive.com/v1",
            "env_token": "PIPEDRIVE_API_TOKEN",
            "category": "CRM",
            "description": "Sales CRM",
        },
        "close": {
            "display_name": "Close",
            "api_url": "https://api.close.com/api/v1",
            "env_token": "CLOSE_API_KEY",
            "category": "CRM",
            "description": "Sales engagement CRM",
        },

        # =====================================================================
        # Data & Databases
        # =====================================================================
        "airtable": {
            "display_name": "Airtable",
            "api_url": "https://api.airtable.com/v0",
            "env_token": "AIRTABLE_API_KEY",
            "category": "Data",
            "description": "Spreadsheet-database hybrid",
        },
        "supabase": {
            "display_name": "Supabase",
            "api_url": "https://your-project.supabase.co/rest/v1",
            "env_token": "SUPABASE_SERVICE_KEY",
            "category": "Data",
            "description": "Open source Firebase alternative",
        },
        "mongodb_atlas": {
            "display_name": "MongoDB Atlas",
            "api_url": "https://cloud.mongodb.com/api/atlas/v2",
            "env_token": "MONGODB_ATLAS_API_KEY",
            "category": "Data",
            "description": "Cloud MongoDB",
        },
        "planetscale": {
            "display_name": "PlanetScale",
            "api_url": "https://api.planetscale.com/v1",
            "env_token": "PLANETSCALE_SERVICE_TOKEN",
            "category": "Data",
            "description": "Serverless MySQL",
        },

        # =====================================================================
        # Cloud & DevOps
        # =====================================================================
        "cloudflare": {
            "display_name": "Cloudflare",
            "api_url": "https://api.cloudflare.com/client/v4",
            "env_token": "CLOUDFLARE_API_TOKEN",
            "category": "Cloud",
            "description": "CDN and edge platform",
        },
        "digitalocean": {
            "display_name": "DigitalOcean",
            "api_url": "https://api.digitalocean.com/v2",
            "env_token": "DIGITALOCEAN_TOKEN",
            "category": "Cloud",
            "description": "Cloud infrastructure",
        },
        "railway": {
            "display_name": "Railway",
            "api_url": "https://backboard.railway.app/graphql/v2",
            "env_token": "RAILWAY_API_TOKEN",
            "category": "Cloud",
            "description": "App deployment platform",
        },
        "render": {
            "display_name": "Render",
            "api_url": "https://api.render.com/v1",
            "env_token": "RENDER_API_KEY",
            "category": "Cloud",
            "description": "Cloud hosting platform",
        },
        "netlify": {
            "display_name": "Netlify",
            "api_url": "https://api.netlify.com/api/v1",
            "env_token": "NETLIFY_AUTH_TOKEN",
            "category": "Cloud",
            "description": "Web hosting and serverless",
        },

        # =====================================================================
        # Monitoring & Analytics
        # =====================================================================
        "sentry": {
            "display_name": "Sentry",
            "api_url": "https://sentry.io/api/0",
            "env_token": "SENTRY_AUTH_TOKEN",
            "category": "Monitoring",
            "description": "Error tracking",
        },
        "datadog": {
            "display_name": "Datadog",
            "api_url": "https://api.datadoghq.com/api/v2",
            "env_token": "DATADOG_API_KEY",
            "category": "Monitoring",
            "description": "Cloud monitoring",
        },
        "posthog": {
            "display_name": "PostHog",
            "api_url": "https://app.posthog.com/api",
            "env_token": "POSTHOG_API_KEY",
            "category": "Monitoring",
            "description": "Product analytics",
        },
        "mixpanel": {
            "display_name": "Mixpanel",
            "api_url": "https://api.mixpanel.com",
            "env_token": "MIXPANEL_SERVICE_ACCOUNT",
            "category": "Monitoring",
            "description": "Product analytics",
        },
        "amplitude": {
            "display_name": "Amplitude",
            "api_url": "https://amplitude.com/api/2",
            "env_token": "AMPLITUDE_API_KEY",
            "category": "Monitoring",
            "description": "Product analytics",
        },

        # =====================================================================
        # Content & Media
        # =====================================================================
        "youtube": {
            "display_name": "YouTube",
            "api_url": "https://www.googleapis.com/youtube/v3",
            "env_token": "YOUTUBE_API_KEY",
            "category": "Media",
            "description": "Video platform",
        },
        "spotify": {
            "display_name": "Spotify",
            "api_url": "https://api.spotify.com/v1",
            "env_token": "SPOTIFY_ACCESS_TOKEN",
            "category": "Media",
            "description": "Music streaming",
        },
        "twitter": {
            "display_name": "Twitter/X",
            "api_url": "https://api.twitter.com/2",
            "env_token": "TWITTER_BEARER_TOKEN",
            "category": "Media",
            "description": "Social media platform",
        },
        "reddit": {
            "display_name": "Reddit",
            "api_url": "https://oauth.reddit.com",
            "env_token": "REDDIT_ACCESS_TOKEN",
            "category": "Media",
            "description": "Social news platform",
        },

        # =====================================================================
        # Automation & Integration
        # =====================================================================
        "zapier": {
            "display_name": "Zapier",
            "api_url": "https://api.zapier.com/v1",
            "env_token": "ZAPIER_API_KEY",
            "category": "Automation",
            "description": "Workflow automation",
        },
        "make": {
            "display_name": "Make (Integromat)",
            "api_url": "https://hook.us1.make.com",
            "env_token": "MAKE_API_KEY",
            "category": "Automation",
            "description": "Visual automation",
        },
        "n8n": {
            "display_name": "n8n",
            "api_url": "https://your-instance.n8n.cloud/api/v1",
            "env_token": "N8N_API_KEY",
            "category": "Automation",
            "description": "Self-hosted automation",
        },

        # =====================================================================
        # Email
        # =====================================================================
        "sendgrid": {
            "display_name": "SendGrid",
            "api_url": "https://api.sendgrid.com/v3",
            "env_token": "SENDGRID_API_KEY",
            "category": "Email",
            "description": "Email delivery",
        },
        "resend": {
            "display_name": "Resend",
            "api_url": "https://api.resend.com",
            "env_token": "RESEND_API_KEY",
            "category": "Email",
            "description": "Modern email API",
        },
        "mailgun": {
            "display_name": "Mailgun",
            "api_url": "https://api.mailgun.net/v3",
            "env_token": "MAILGUN_API_KEY",
            "category": "Email",
            "description": "Email delivery",
        },
        "postmark": {
            "display_name": "Postmark",
            "api_url": "https://api.postmarkapp.com",
            "env_token": "POSTMARK_SERVER_TOKEN",
            "category": "Email",
            "description": "Transactional email",
        },

        # =====================================================================
        # Customer Support
        # =====================================================================
        "intercom": {
            "display_name": "Intercom",
            "api_url": "https://api.intercom.io",
            "env_token": "INTERCOM_ACCESS_TOKEN",
            "category": "Support",
            "description": "Customer messaging",
        },
        "zendesk": {
            "display_name": "Zendesk",
            "api_url": "https://your-subdomain.zendesk.com/api/v2",
            "env_token": "ZENDESK_API_TOKEN",
            "category": "Support",
            "description": "Customer service platform",
        },
        "freshdesk": {
            "display_name": "Freshdesk",
            "api_url": "https://your-domain.freshdesk.com/api/v2",
            "env_token": "FRESHDESK_API_KEY",
            "category": "Support",
            "description": "Helpdesk software",
        },

        # =====================================================================
        # HR & Recruiting
        # =====================================================================
        "greenhouse": {
            "display_name": "Greenhouse",
            "api_url": "https://harvest.greenhouse.io/v1",
            "env_token": "GREENHOUSE_API_KEY",
            "category": "HR",
            "description": "Recruiting platform",
        },
        "lever": {
            "display_name": "Lever",
            "api_url": "https://api.lever.co/v1",
            "env_token": "LEVER_API_KEY",
            "category": "HR",
            "description": "Recruiting software",
        },
        "rippling": {
            "display_name": "Rippling",
            "api_url": "https://api.rippling.com",
            "env_token": "RIPPLING_API_KEY",
            "category": "HR",
            "description": "HR management platform",
        },

        # =====================================================================
        # Storage & Files
        # =====================================================================
        "dropbox": {
            "display_name": "Dropbox",
            "api_url": "https://api.dropboxapi.com/2",
            "env_token": "DROPBOX_ACCESS_TOKEN",
            "category": "Storage",
            "description": "Cloud storage",
        },
        "box": {
            "display_name": "Box",
            "api_url": "https://api.box.com/2.0",
            "env_token": "BOX_ACCESS_TOKEN",
            "category": "Storage",
            "description": "Enterprise content management",
        },
        "google_drive": {
            "display_name": "Google Drive",
            "api_url": "https://www.googleapis.com/drive/v3",
            "env_token": "GOOGLE_DRIVE_API_KEY",
            "category": "Storage",
            "description": "Cloud storage and docs",
        },

        # =====================================================================
        # Productivity
        # =====================================================================
        "calendly": {
            "display_name": "Calendly",
            "api_url": "https://api.calendly.com",
            "env_token": "CALENDLY_API_KEY",
            "category": "Productivity",
            "description": "Scheduling and appointments",
        },
        "loom": {
            "display_name": "Loom",
            "api_url": "https://www.loom.com/v1",
            "env_token": "LOOM_API_KEY",
            "category": "Productivity",
            "description": "Async video messaging",
        },
        "roam": {
            "display_name": "Roam Research",
            "api_url": "https://api.roamresearch.com",
            "env_token": "ROAM_API_TOKEN",
            "category": "Productivity",
            "description": "Networked note-taking",
        },
        "raindrop": {
            "display_name": "Raindrop.io",
            "api_url": "https://api.raindrop.io/rest/v1",
            "env_token": "RAINDROP_ACCESS_TOKEN",
            "category": "Productivity",
            "description": "Bookmark management",
        },
        "readwise": {
            "display_name": "Readwise",
            "api_url": "https://readwise.io/api/v2",
            "env_token": "READWISE_ACCESS_TOKEN",
            "category": "Productivity",
            "description": "Reading highlights aggregation",
        },

        # =====================================================================
        # Dev Tools
        # =====================================================================
        "github": {
            "display_name": "GitHub",
            "api_url": "https://api.github.com",
            "env_token": "GITHUB_TOKEN",
            "category": "Dev Tools",
            "description": "Code hosting and collaboration",
        },
        "gitlab": {
            "display_name": "GitLab",
            "api_url": "https://gitlab.com/api/v4",
            "env_token": "GITLAB_ACCESS_TOKEN",
            "category": "Dev Tools",
            "description": "DevOps platform",
        },
        "bitbucket": {
            "display_name": "Bitbucket",
            "api_url": "https://api.bitbucket.org/2.0",
            "env_token": "BITBUCKET_ACCESS_TOKEN",
            "category": "Dev Tools",
            "description": "Atlassian git hosting",
        },
        "vercel": {
            "display_name": "Vercel",
            "api_url": "https://api.vercel.com",
            "env_token": "VERCEL_TOKEN",
            "category": "Dev Tools",
            "description": "Frontend deployment platform",
        },
        "fly": {
            "display_name": "Fly.io",
            "api_url": "https://api.machines.dev/v1",
            "env_token": "FLY_API_TOKEN",
            "category": "Dev Tools",
            "description": "Global app deployment",
        },
        "snyk": {
            "display_name": "Snyk",
            "api_url": "https://api.snyk.io/v1",
            "env_token": "SNYK_TOKEN",
            "category": "Dev Tools",
            "description": "Security vulnerability scanning",
        },
        "buildkite": {
            "display_name": "Buildkite",
            "api_url": "https://api.buildkite.com/v2",
            "env_token": "BUILDKITE_ACCESS_TOKEN",
            "category": "Dev Tools",
            "description": "CI/CD pipelines",
        },
        "circleci": {
            "display_name": "CircleCI",
            "api_url": "https://circleci.com/api/v2",
            "env_token": "CIRCLECI_TOKEN",
            "category": "Dev Tools",
            "description": "Continuous integration",
        },
        "doppler": {
            "display_name": "Doppler",
            "api_url": "https://api.doppler.com/v3",
            "env_token": "DOPPLER_TOKEN",
            "category": "Dev Tools",
            "description": "Secrets management",
        },
        "launchdarkly": {
            "display_name": "LaunchDarkly",
            "api_url": "https://app.launchdarkly.com/api/v2",
            "env_token": "LAUNCHDARKLY_ACCESS_TOKEN",
            "category": "Dev Tools",
            "description": "Feature flag management",
        },

        # =====================================================================
        # Sales & Prospecting
        # =====================================================================
        "apollo": {
            "display_name": "Apollo.io",
            "api_url": "https://api.apollo.io/v1",
            "env_token": "APOLLO_API_KEY",
            "category": "Sales",
            "description": "Sales intelligence platform",
        },
        "outreach": {
            "display_name": "Outreach",
            "api_url": "https://api.outreach.io/api/v2",
            "env_token": "OUTREACH_ACCESS_TOKEN",
            "category": "Sales",
            "description": "Sales engagement platform",
        },
        "gong": {
            "display_name": "Gong",
            "api_url": "https://api.gong.io/v2",
            "env_token": "GONG_ACCESS_KEY",
            "category": "Sales",
            "description": "Revenue intelligence",
        },
        "clearbit": {
            "display_name": "Clearbit",
            "api_url": "https://company.clearbit.com/v2",
            "env_token": "CLEARBIT_API_KEY",
            "category": "Sales",
            "description": "Data enrichment",
        },
        "zoominfo": {
            "display_name": "ZoomInfo",
            "api_url": "https://api.zoominfo.com",
            "env_token": "ZOOMINFO_API_KEY",
            "category": "Sales",
            "description": "B2B contact database",
        },

        # =====================================================================
        # Data Infrastructure
        # =====================================================================
        "snowflake": {
            "display_name": "Snowflake",
            "api_url": "https://your-account.snowflakecomputing.com/api/v2",
            "env_token": "SNOWFLAKE_API_KEY",
            "category": "Data Infrastructure",
            "description": "Cloud data warehouse",
        },
        "databricks": {
            "display_name": "Databricks",
            "api_url": "https://your-workspace.cloud.databricks.com/api/2.0",
            "env_token": "DATABRICKS_TOKEN",
            "category": "Data Infrastructure",
            "description": "Lakehouse platform",
        },
        "fivetran": {
            "display_name": "Fivetran",
            "api_url": "https://api.fivetran.com/v1",
            "env_token": "FIVETRAN_API_KEY",
            "category": "Data Infrastructure",
            "description": "Data integration (ELT)",
        },
        "airbyte": {
            "display_name": "Airbyte",
            "api_url": "https://api.airbyte.com/v1",
            "env_token": "AIRBYTE_API_KEY",
            "category": "Data Infrastructure",
            "description": "Open-source data integration",
        },
        "dbt_cloud": {
            "display_name": "dbt Cloud",
            "api_url": "https://cloud.getdbt.com/api/v2",
            "env_token": "DBT_CLOUD_API_TOKEN",
            "category": "Data Infrastructure",
            "description": "Data transformation",
        },
        "clickhouse": {
            "display_name": "ClickHouse",
            "api_url": "https://api.clickhouse.cloud/v1",
            "env_token": "CLICKHOUSE_API_KEY",
            "category": "Data Infrastructure",
            "description": "Analytics database",
        },
        "pinecone": {
            "display_name": "Pinecone",
            "api_url": "https://api.pinecone.io",
            "env_token": "PINECONE_API_KEY",
            "category": "Data Infrastructure",
            "description": "Vector database",
        },
        "weaviate": {
            "display_name": "Weaviate",
            "api_url": "https://your-cluster.weaviate.network/v1",
            "env_token": "WEAVIATE_API_KEY",
            "category": "Data Infrastructure",
            "description": "Vector search engine",
        },

        # =====================================================================
        # AI Tooling
        # =====================================================================
        "openai": {
            "display_name": "OpenAI",
            "api_url": "https://api.openai.com/v1",
            "env_token": "OPENAI_API_KEY",
            "category": "AI Tooling",
            "description": "GPT models and embeddings",
        },
        "anthropic": {
            "display_name": "Anthropic",
            "api_url": "https://api.anthropic.com/v1",
            "env_token": "ANTHROPIC_API_KEY",
            "category": "AI Tooling",
            "description": "Claude models",
        },
        "cohere": {
            "display_name": "Cohere",
            "api_url": "https://api.cohere.ai/v1",
            "env_token": "COHERE_API_KEY",
            "category": "AI Tooling",
            "description": "Embeddings and reranking",
        },
        "replicate": {
            "display_name": "Replicate",
            "api_url": "https://api.replicate.com/v1",
            "env_token": "REPLICATE_API_TOKEN",
            "category": "AI Tooling",
            "description": "Model hosting platform",
        },
        "huggingface": {
            "display_name": "Hugging Face",
            "api_url": "https://huggingface.co/api",
            "env_token": "HUGGINGFACE_TOKEN",
            "category": "AI Tooling",
            "description": "Model hub and inference",
        },
        "langsmith": {
            "display_name": "LangSmith",
            "api_url": "https://api.smith.langchain.com",
            "env_token": "LANGSMITH_API_KEY",
            "category": "AI Tooling",
            "description": "LLM observability",
        },
        "helicone": {
            "display_name": "Helicone",
            "api_url": "https://api.helicone.ai/v1",
            "env_token": "HELICONE_API_KEY",
            "category": "AI Tooling",
            "description": "LLM gateway and logging",
        },
        "modal": {
            "display_name": "Modal",
            "api_url": "https://api.modal.com",
            "env_token": "MODAL_TOKEN_ID",
            "category": "AI Tooling",
            "description": "Serverless GPU compute",
        },

        # =====================================================================
        # Fintech & Billing
        # =====================================================================
        "chargebee": {
            "display_name": "Chargebee",
            "api_url": "https://your-site.chargebee.com/api/v2",
            "env_token": "CHARGEBEE_API_KEY",
            "category": "Fintech",
            "description": "Subscription billing",
        },
        "recurly": {
            "display_name": "Recurly",
            "api_url": "https://v3.recurly.com",
            "env_token": "RECURLY_API_KEY",
            "category": "Fintech",
            "description": "Recurring payments",
        },
        "brex": {
            "display_name": "Brex",
            "api_url": "https://platform.brexapis.com/v1",
            "env_token": "BREX_TOKEN",
            "category": "Fintech",
            "description": "Corporate cards and spend",
        },
        "ramp": {
            "display_name": "Ramp",
            "api_url": "https://api.ramp.com/developer/v1",
            "env_token": "RAMP_API_KEY",
            "category": "Fintech",
            "description": "Spend management",
        },

        # =====================================================================
        # E-commerce
        # =====================================================================
        "shopify": {
            "display_name": "Shopify",
            "api_url": "https://your-store.myshopify.com/admin/api/2024-01",
            "env_token": "SHOPIFY_ACCESS_TOKEN",
            "category": "E-commerce",
            "description": "E-commerce platform",
        },
        "woocommerce": {
            "display_name": "WooCommerce",
            "api_url": "https://your-site.com/wp-json/wc/v3",
            "env_token": "WOOCOMMERCE_API_KEY",
            "category": "E-commerce",
            "description": "WordPress commerce",
        },
        "gumroad": {
            "display_name": "Gumroad",
            "api_url": "https://api.gumroad.com/v2",
            "env_token": "GUMROAD_ACCESS_TOKEN",
            "category": "E-commerce",
            "description": "Creator sales platform",
        },
        "lemonsqueezy": {
            "display_name": "Lemon Squeezy",
            "api_url": "https://api.lemonsqueezy.com/v1",
            "env_token": "LEMONSQUEEZY_API_KEY",
            "category": "E-commerce",
            "description": "Digital product payments",
        },

        # =====================================================================
        # Security
        # =====================================================================
        "onepassword": {
            "display_name": "1Password",
            "api_url": "https://your-domain.1password.com/api/v1",
            "env_token": "OP_SERVICE_ACCOUNT_TOKEN",
            "category": "Security",
            "description": "Password and secrets manager",
        },
    }

    # Handle --list-presets
    if args.list_presets:
        print_presets(presets)
        return

    # Require service_name if not listing presets
    if not args.service_name:
        parser.error("service_name is required (or use --list-presets)")

    service_key = args.service_name.lower().replace("-", "_")

    # Apply preset if requested or if values not provided
    if args.preset and service_key in presets:
        preset = presets[service_key]
        display_name = args.display_name or preset["display_name"]
        api_url = args.api_url or preset["api_url"]
        env_token = args.env_token or preset["env_token"]
    else:
        # Use provided values or derive defaults
        display_name = args.display_name or to_pascal_case(args.service_name)
        api_url = args.api_url or f"https://api.{args.service_name}.com"
        env_token = args.env_token or f"{args.service_name.upper().replace('-', '_')}_TOKEN"

    output_dir = Path(args.output_dir).resolve()

    print()
    print(f"Generating FGP daemon: {args.service_name}")
    print(f"  Display name: {display_name}")
    print(f"  API URL: {api_url}")
    print(f"  Token env var: {env_token}")
    print(f"  Output: {output_dir / args.service_name}")
    print()

    create_daemon(
        service_name=args.service_name,
        display_name=display_name,
        api_base_url=api_url,
        env_token=env_token,
        author=args.author,
        output_dir=output_dir,
    )


if __name__ == "__main__":
    main()
