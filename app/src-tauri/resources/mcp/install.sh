#!/bin/bash
# FGP MCP Server Installation Script
# This script registers the FGP MCP server with Claude Code and installs the skill.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SKILL_DIR="$HOME/.claude/skills/fgp-gateway"
MCP_SERVER="$SCRIPT_DIR/fgp-mcp-server.py"

echo "FGP MCP Server Installer"
echo "========================"
echo ""

# Check if Claude Code is installed
if ! command -v claude &> /dev/null; then
    echo "Error: Claude Code CLI not found."
    echo "Please install Claude Code first: https://claude.ai/code"
    exit 1
fi

# Check if Python 3 is available
if ! command -v python3 &> /dev/null; then
    echo "Error: Python 3 not found."
    echo "Please install Python 3 first."
    exit 1
fi

# Check if MCP SDK is installed
if ! python3 -c "import mcp" &> /dev/null; then
    echo "Installing MCP SDK..."
    pip3 install mcp
fi

# Register MCP server with Claude Code
echo "Registering FGP MCP server with Claude Code..."
claude mcp add fgp -- python3 "$MCP_SERVER" 2>/dev/null || {
    echo "Note: MCP server may already be registered. Updating..."
    claude mcp remove fgp 2>/dev/null || true
    claude mcp add fgp -- python3 "$MCP_SERVER"
}

# Install Claude Code skill
echo "Installing FGP skill..."
mkdir -p "$SKILL_DIR"
cp "$SCRIPT_DIR/../skill/skill.md" "$SKILL_DIR/skill.md"

echo ""
echo "Installation complete!"
echo ""
echo "Available tools (after restarting Claude Code):"
echo "  - fgp_list_daemons    List installed FGP daemons"
echo "  - fgp_start_daemon    Start a daemon by name"
echo "  - fgp_stop_daemon     Stop a daemon by name"
echo "  - fgp_browser_*       Browser automation tools"
echo "  - fgp_gmail_*         Gmail tools"
echo "  - fgp_github_*        GitHub tools"
echo "  - ... and more based on installed daemons"
echo ""
echo "To verify installation:"
echo "  claude mcp list"
echo ""
echo "To use FGP in Claude Code:"
echo "  1. Start a daemon: fgp_start_daemon(name=\"browser\")"
echo "  2. Use its tools: fgp_browser_open(url=\"https://example.com\")"
