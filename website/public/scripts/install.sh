#!/bin/bash
# FGP CLI Installer
# https://getfgp.com

set -e

CYAN='\033[0;36m'
GOLD='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${CYAN}FGP${NC} - Fast Gateway Protocol"
echo ""

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin)
    PLATFORM="darwin"
    ;;
  Linux)
    PLATFORM="linux"
    ;;
  *)
    echo "Unsupported OS: $OS"
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64|amd64)
    ARCH="x86_64"
    ;;
  arm64|aarch64)
    ARCH="aarch64"
    ;;
  *)
    echo "Unsupported architecture: $ARCH"
    exit 1
    ;;
esac

echo "Detected platform: ${PLATFORM}-${ARCH}"
echo ""

# Installation directory
FGP_HOME="${FGP_HOME:-$HOME/.fgp}"
BIN_DIR="${FGP_HOME}/bin"

echo "Installing to: ${FGP_HOME}"
mkdir -p "${BIN_DIR}"
mkdir -p "${FGP_HOME}/services"
mkdir -p "${FGP_HOME}/auth"

# Download CLI
# TODO: Replace with actual release URL
DOWNLOAD_URL="https://github.com/fast-gateway-protocol/cli/releases/latest/download/fgp-${PLATFORM}-${ARCH}"

echo "Downloading FGP CLI..."
# curl -fsSL "${DOWNLOAD_URL}" -o "${BIN_DIR}/fgp"
# chmod +x "${BIN_DIR}/fgp"

# For now, create a placeholder
cat > "${BIN_DIR}/fgp" << 'EOF'
#!/bin/bash
echo "FGP CLI (placeholder)"
echo "Visit https://getfgp.com for installation instructions"
EOF
chmod +x "${BIN_DIR}/fgp"

# Add to PATH
SHELL_NAME="$(basename "$SHELL")"
case "$SHELL_NAME" in
  zsh)
    PROFILE="${HOME}/.zshrc"
    ;;
  bash)
    PROFILE="${HOME}/.bashrc"
    ;;
  *)
    PROFILE="${HOME}/.profile"
    ;;
esac

if ! grep -q "FGP_HOME" "${PROFILE}" 2>/dev/null; then
  echo "" >> "${PROFILE}"
  echo "# FGP - Fast Gateway Protocol" >> "${PROFILE}"
  echo "export FGP_HOME=\"${FGP_HOME}\"" >> "${PROFILE}"
  echo "export PATH=\"\${FGP_HOME}/bin:\${PATH}\"" >> "${PROFILE}"
  echo ""
  echo -e "Added FGP to ${GOLD}${PROFILE}${NC}"
fi

echo ""
echo -e "${CYAN}Installation complete!${NC}"
echo ""
echo "To get started:"
echo "  1. Restart your terminal (or run: source ${PROFILE})"
echo "  2. Install a package:  fgp install browser"
echo "  3. Start the daemon:   fgp start browser"
echo ""
echo -e "Visit ${CYAN}https://getfgp.com${NC} for documentation"
