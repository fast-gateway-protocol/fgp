#!/bin/bash
# FGP Manager Visual Test Runner
# Captures screenshots of UI components for visual regression testing

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
ARTIFACTS_DIR="$HOME/.fgp/test-artifacts"

echo "🧪 FGP Manager Visual Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Parse arguments
OPEN_REPORT=false
while [[ $# -gt 0 ]]; do
    case $1 in
        --open)
            OPEN_REPORT=true
            shift
            ;;
        --clean)
            echo "🧹 Cleaning artifacts..."
            rm -rf "$ARTIFACTS_DIR"
            shift
            ;;
        *)
            shift
            ;;
    esac
done

cd "$PROJECT_DIR"

# Build first
echo "🔨 Building..."
swift build 2>&1 | grep -E "(error:|Build complete)" || true
echo ""

# Run visual tests
echo "📸 Running visual tests..."
.build/arm64-apple-macosx/debug/FGPManager --visual-tests

echo ""

# Open report if requested
if [ "$OPEN_REPORT" = true ]; then
    echo "🌐 Opening report in browser..."
    open "$ARTIFACTS_DIR/report.html"
fi

echo ""
echo "Usage:"
echo "  ./scripts/run-visual-tests.sh        # Run tests"
echo "  ./scripts/run-visual-tests.sh --open # Run and open report"
echo "  ./scripts/run-visual-tests.sh --clean # Clean artifacts first"
