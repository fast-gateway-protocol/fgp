#!/bin/bash
# FGP Installer - Fast Gateway Protocol
# Usage: curl -fsSL https://raw.githubusercontent.com/fast-gateway-protocol/fgp/master/install.sh | bash
#    or: curl -fsSL https://fgp.dev/install.sh | bash -s -- browser gmail calendar

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
FGP_HOME="${FGP_HOME:-$HOME/.fgp}"
BIN_DIR="$FGP_HOME/bin"
SERVICES_DIR="$FGP_HOME/services"
GITHUB_ORG="fast-gateway-protocol"

# Available daemons
AVAILABLE_DAEMONS="browser cli gmail calendar github fly neon vercel"

print_banner() {
    echo -e "${BLUE}"
    echo "  _____ ____ ____  "
    echo " |  ___|  _ \  _ \ "
    echo " | |_  | | _|| |_) |"
    echo " |  _| | |_| |  __/ "
    echo " |_|    \____|_|    "
    echo -e "${NC}"
    echo "Fast Gateway Protocol Installer"
    echo "================================"
    echo ""
}

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

detect_os() {
    local os
    os="$(uname -s)"
    case "$os" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "macos" ;;
        *)       error "Unsupported OS: $os" ;;
    esac
}

detect_arch() {
    local arch
    arch="$(uname -m)"
    case "$arch" in
        x86_64)  echo "x64" ;;
        aarch64) echo "arm64" ;;
        arm64)   echo "arm64" ;;
        *)       error "Unsupported architecture: $arch" ;;
    esac
}

get_latest_version() {
    local repo="$1"
    local version
    version=$(curl -fsSL "https://api.github.com/repos/$GITHUB_ORG/$repo/releases/latest" 2>/dev/null | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
    if [ -z "$version" ]; then
        echo "v0.1.0"  # Fallback
    else
        echo "$version"
    fi
}

get_binary_name() {
    local daemon="$1"
    case "$daemon" in
        browser)  echo "browser-gateway" ;;
        cli)      echo "fgp" ;;
        *)        echo "fgp-$daemon" ;;
    esac
}

get_artifact_name() {
    local daemon="$1"
    local os="$2"
    local arch="$3"
    local binary_name
    binary_name=$(get_binary_name "$daemon")
    echo "${binary_name}-${os}-${arch}"
}

download_and_install() {
    local daemon="$1"
    local os="$2"
    local arch="$3"

    info "Installing $daemon daemon..."

    local version
    version=$(get_latest_version "$daemon")

    local artifact_name
    artifact_name=$(get_artifact_name "$daemon" "$os" "$arch")

    local binary_name
    binary_name=$(get_binary_name "$daemon")

    local download_url="https://github.com/$GITHUB_ORG/$daemon/releases/download/$version/${artifact_name}.tar.gz"

    local tmp_dir
    tmp_dir=$(mktemp -d)
    trap "rm -rf $tmp_dir" EXIT

    info "Downloading $daemon $version for $os-$arch..."

    if ! curl -fsSL "$download_url" -o "$tmp_dir/archive.tar.gz" 2>/dev/null; then
        warn "No pre-built binary found for $daemon. You may need to build from source."
        warn "  git clone https://github.com/$GITHUB_ORG/$daemon && cd $daemon && cargo build --release"
        return 1
    fi

    # Extract
    tar -xzf "$tmp_dir/archive.tar.gz" -C "$tmp_dir"

    # Install binary
    mkdir -p "$BIN_DIR"
    mv "$tmp_dir/$binary_name" "$BIN_DIR/"
    chmod +x "$BIN_DIR/$binary_name"

    # Create service directory
    mkdir -p "$SERVICES_DIR/$daemon"

    success "$daemon installed to $BIN_DIR/$binary_name"
}

install_all() {
    local os="$1"
    local arch="$2"
    shift 2
    local daemons=("$@")

    local failed=()

    for daemon in "${daemons[@]}"; do
        if ! download_and_install "$daemon" "$os" "$arch"; then
            failed+=("$daemon")
        fi
    done

    if [ ${#failed[@]} -gt 0 ]; then
        warn "Failed to install: ${failed[*]}"
    fi
}

setup_path() {
    local shell_rc=""
    local shell_name=""

    if [ -n "$BASH_VERSION" ]; then
        if [ -f "$HOME/.bashrc" ]; then
            shell_rc="$HOME/.bashrc"
            shell_name="bash"
        elif [ -f "$HOME/.bash_profile" ]; then
            shell_rc="$HOME/.bash_profile"
            shell_name="bash"
        fi
    fi

    if [ -n "$ZSH_VERSION" ] || [ -f "$HOME/.zshrc" ]; then
        shell_rc="$HOME/.zshrc"
        shell_name="zsh"
    fi

    if [ -z "$shell_rc" ]; then
        warn "Could not detect shell configuration file"
        echo ""
        echo "Add the following to your shell profile:"
        echo "  export PATH=\"$BIN_DIR:\$PATH\""
        return
    fi

    # Check if already in PATH
    if grep -q "FGP_HOME" "$shell_rc" 2>/dev/null; then
        info "PATH already configured in $shell_rc"
        return
    fi

    echo "" >> "$shell_rc"
    echo "# FGP - Fast Gateway Protocol" >> "$shell_rc"
    echo "export FGP_HOME=\"$FGP_HOME\"" >> "$shell_rc"
    echo "export PATH=\"\$FGP_HOME/bin:\$PATH\"" >> "$shell_rc"

    success "Added FGP to PATH in $shell_rc"
    echo ""
    echo "Run this to update your current shell:"
    echo "  source $shell_rc"
}

print_usage() {
    echo "Usage: $0 [daemon...]"
    echo ""
    echo "Install FGP daemons. If no daemons specified, installs cli + browser."
    echo ""
    echo "Available daemons:"
    echo "  cli       - FGP command-line interface (recommended)"
    echo "  browser   - Browser automation daemon"
    echo "  gmail     - Gmail API daemon"
    echo "  calendar  - Google Calendar daemon"
    echo "  github    - GitHub API daemon"
    echo "  fly       - Fly.io daemon"
    echo "  neon      - Neon Postgres daemon"
    echo "  vercel    - Vercel daemon"
    echo ""
    echo "Examples:"
    echo "  $0                    # Install cli + browser"
    echo "  $0 gmail calendar     # Install gmail and calendar"
    echo "  $0 all                # Install all daemons"
    echo ""
    echo "Environment variables:"
    echo "  FGP_HOME    Installation directory (default: ~/.fgp)"
}

main() {
    print_banner

    # Parse arguments
    local daemons=()

    for arg in "$@"; do
        case "$arg" in
            -h|--help)
                print_usage
                exit 0
                ;;
            all)
                daemons=($AVAILABLE_DAEMONS)
                ;;
            *)
                if echo "$AVAILABLE_DAEMONS" | grep -qw "$arg"; then
                    daemons+=("$arg")
                else
                    error "Unknown daemon: $arg. Use --help for available options."
                fi
                ;;
        esac
    done

    # Default to cli + browser if no daemons specified
    if [ ${#daemons[@]} -eq 0 ]; then
        daemons=("cli" "browser")
    fi

    # Detect platform
    local os arch
    os=$(detect_os)
    arch=$(detect_arch)

    info "Detected platform: $os-$arch"
    info "Installation directory: $FGP_HOME"
    info "Installing: ${daemons[*]}"
    echo ""

    # Create directories
    mkdir -p "$FGP_HOME"
    mkdir -p "$BIN_DIR"
    mkdir -p "$SERVICES_DIR"

    # Install daemons
    install_all "$os" "$arch" "${daemons[@]}"

    echo ""

    # Setup PATH
    setup_path

    echo ""
    success "Installation complete!"
    echo ""
    echo "Quick start:"
    echo "  fgp start browser        # Start browser daemon"
    echo "  fgp call browser.health  # Check daemon health"
    echo "  fgp --help               # See all commands"
    echo ""
    echo "Documentation: https://fast-gateway-protocol.github.io/fgp/"
}

main "$@"
