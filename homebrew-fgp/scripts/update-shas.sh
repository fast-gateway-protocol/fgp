#!/bin/bash
# Update SHA256 values for Homebrew formulas
# Usage: ./update-shas.sh [version]
#
# This script fetches the release assets and updates the SHA256 values
# in the Homebrew formula files.

set -e

VERSION="${1:-0.1.0}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FORMULA_DIR="$SCRIPT_DIR/../Formula"

echo "Updating SHA256 values for version $VERSION..."

# Function to get SHA256 for a URL
get_sha256() {
    local url="$1"
    curl -sL "$url" | shasum -a 256 | awk '{print $1}'
}

# Update fgp.rb
echo "Updating fgp.rb..."

FGP_MACOS_ARM64=$(get_sha256 "https://github.com/fast-gateway-protocol/cli/releases/download/v${VERSION}/fgp-macos-arm64.tar.gz")
FGP_MACOS_X64=$(get_sha256 "https://github.com/fast-gateway-protocol/cli/releases/download/v${VERSION}/fgp-macos-x64.tar.gz")
FGP_LINUX_ARM64=$(get_sha256 "https://github.com/fast-gateway-protocol/cli/releases/download/v${VERSION}/fgp-linux-arm64.tar.gz")
FGP_LINUX_X64=$(get_sha256 "https://github.com/fast-gateway-protocol/cli/releases/download/v${VERSION}/fgp-linux-x64.tar.gz")

echo "  macOS ARM64: $FGP_MACOS_ARM64"
echo "  macOS x64:   $FGP_MACOS_X64"
echo "  Linux ARM64: $FGP_LINUX_ARM64"
echo "  Linux x64:   $FGP_LINUX_X64"

# Update fgp-browser.rb
echo "Updating fgp-browser.rb..."

BROWSER_MACOS_ARM64=$(get_sha256 "https://github.com/fast-gateway-protocol/browser/releases/download/v${VERSION}/browser-gateway-macos-arm64.tar.gz")
BROWSER_MACOS_X64=$(get_sha256 "https://github.com/fast-gateway-protocol/browser/releases/download/v${VERSION}/browser-gateway-macos-x64.tar.gz")
BROWSER_LINUX_ARM64=$(get_sha256 "https://github.com/fast-gateway-protocol/browser/releases/download/v${VERSION}/browser-gateway-linux-arm64.tar.gz")
BROWSER_LINUX_X64=$(get_sha256 "https://github.com/fast-gateway-protocol/browser/releases/download/v${VERSION}/browser-gateway-linux-x64.tar.gz")

echo "  macOS ARM64: $BROWSER_MACOS_ARM64"
echo "  macOS x64:   $BROWSER_MACOS_X64"
echo "  Linux ARM64: $BROWSER_LINUX_ARM64"
echo "  Linux x64:   $BROWSER_LINUX_X64"

echo ""
echo "SHA256 values retrieved. Update the formula files with these values."
echo ""
echo "fgp.rb:"
echo "  sha256 \"$FGP_MACOS_ARM64\"  # macos-arm64"
echo "  sha256 \"$FGP_MACOS_X64\"  # macos-x64"
echo "  sha256 \"$FGP_LINUX_ARM64\"  # linux-arm64"
echo "  sha256 \"$FGP_LINUX_X64\"  # linux-x64"
echo ""
echo "fgp-browser.rb:"
echo "  sha256 \"$BROWSER_MACOS_ARM64\"  # macos-arm64"
echo "  sha256 \"$BROWSER_MACOS_X64\"  # macos-x64"
echo "  sha256 \"$BROWSER_LINUX_ARM64\"  # linux-arm64"
echo "  sha256 \"$BROWSER_LINUX_X64\"  # linux-x64"
