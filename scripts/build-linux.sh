#!/bin/bash
#
# Nostr Nations - Linux Build Script
# Builds the application for Linux in multiple formats
#
# Usage:
#   ./scripts/build-linux.sh [--release] [--deb] [--appimage] [--rpm] [--all]
#
# Dependencies:
#   - Rust toolchain
#   - Node.js and npm
#   - Build essentials (gcc, make, etc.)
#   - WebKit2GTK development files
#   - For .deb: dpkg-deb
#   - For .rpm: rpmbuild
#   - For AppImage: appimagetool (optional, bundled with tauri)
#

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Default options
RELEASE_MODE=false
BUILD_DEB=false
BUILD_APPIMAGE=false
BUILD_RPM=false
BUILD_ALL=false
VERBOSE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            RELEASE_MODE=true
            shift
            ;;
        --deb)
            BUILD_DEB=true
            shift
            ;;
        --appimage)
            BUILD_APPIMAGE=true
            shift
            ;;
        --rpm)
            BUILD_RPM=true
            shift
            ;;
        --all)
            BUILD_ALL=true
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [--release] [--deb] [--appimage] [--rpm] [--all] [--verbose]"
            echo ""
            echo "Options:"
            echo "  --release    Build in release mode (optimized)"
            echo "  --deb        Build .deb package (Debian/Ubuntu)"
            echo "  --appimage   Build AppImage (universal)"
            echo "  --rpm        Build .rpm package (Fedora/RHEL)"
            echo "  --all        Build all formats (default if none specified)"
            echo "  --verbose    Show detailed output"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# If no format specified, build all
if [ "$BUILD_DEB" = false ] && [ "$BUILD_APPIMAGE" = false ] && [ "$BUILD_RPM" = false ]; then
    BUILD_ALL=true
fi

if [ "$BUILD_ALL" = true ]; then
    BUILD_DEB=true
    BUILD_APPIMAGE=true
    BUILD_RPM=true
fi

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Nostr Nations - Linux Build${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Change to project root
cd "$PROJECT_ROOT"

# Detect distribution
detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        echo "$ID"
    elif [ -f /etc/lsb-release ]; then
        . /etc/lsb-release
        echo "$DISTRIB_ID" | tr '[:upper:]' '[:lower:]'
    else
        echo "unknown"
    fi
}

DISTRO=$(detect_distro)
echo -e "${BLUE}Detected distribution: $DISTRO${NC}"
echo ""

# Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${NC}"

# Check for Rust
if ! command -v rustc &> /dev/null; then
    echo -e "${RED}Error: Rust is not installed. Install from https://rustup.rs${NC}"
    exit 1
fi
echo "  Rust: $(rustc --version)"

# Check for Node.js
if ! command -v node &> /dev/null; then
    echo -e "${RED}Error: Node.js is not installed.${NC}"
    exit 1
fi
echo "  Node.js: $(node --version)"

# Check for required system libraries
echo ""
echo -e "${YELLOW}Checking system dependencies...${NC}"

check_package() {
    local pkg_name="$1"
    local check_cmd="$2"
    
    if eval "$check_cmd" &> /dev/null; then
        echo -e "  ${GREEN}[OK]${NC} $pkg_name"
        return 0
    else
        echo -e "  ${RED}[MISSING]${NC} $pkg_name"
        return 1
    fi
}

MISSING_DEPS=()

# Check for WebKit2GTK (required)
if ! pkg-config --exists webkit2gtk-4.1 2>/dev/null && ! pkg-config --exists webkit2gtk-4.0 2>/dev/null; then
    MISSING_DEPS+=("webkit2gtk-4.1-dev or webkit2gtk-4.0-dev")
fi

# Check for GTK3
if ! pkg-config --exists gtk+-3.0 2>/dev/null; then
    MISSING_DEPS+=("libgtk-3-dev")
fi

# Check for other required libraries
pkg-config --exists libappindicator3-0.1 2>/dev/null || pkg-config --exists ayatana-appindicator3-0.1 2>/dev/null || MISSING_DEPS+=("libayatana-appindicator3-dev or libappindicator3-dev")

if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
    echo -e "${RED}Missing dependencies:${NC}"
    for dep in "${MISSING_DEPS[@]}"; do
        echo "  - $dep"
    done
    echo ""
    echo "Install missing dependencies:"
    case "$DISTRO" in
        ubuntu|debian|pop)
            echo "  sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev"
            ;;
        fedora)
            echo "  sudo dnf install webkit2gtk4.1-devel gtk3-devel libappindicator-gtk3-devel librsvg2-devel"
            ;;
        arch|manjaro)
            echo "  sudo pacman -S webkit2gtk-4.1 gtk3 libappindicator-gtk3 librsvg"
            ;;
        opensuse*)
            echo "  sudo zypper install webkit2gtk3-devel gtk3-devel libappindicator3-devel librsvg-devel"
            ;;
        *)
            echo "  Please install WebKit2GTK, GTK3, and AppIndicator development packages for your distribution"
            ;;
    esac
    exit 1
fi

echo -e "${GREEN}System dependencies OK${NC}"

# Check for package building tools
echo ""
echo -e "${YELLOW}Checking package building tools...${NC}"

if [ "$BUILD_DEB" = true ]; then
    if ! command -v dpkg-deb &> /dev/null; then
        echo -e "${YELLOW}Warning: dpkg-deb not found, .deb builds may fail${NC}"
    else
        echo "  dpkg-deb: available"
    fi
fi

if [ "$BUILD_RPM" = true ]; then
    if ! command -v rpmbuild &> /dev/null; then
        echo -e "${YELLOW}Warning: rpmbuild not found, .rpm builds may fail${NC}"
        echo "  Install with: sudo apt install rpm (Debian/Ubuntu) or sudo dnf install rpm-build (Fedora)"
    else
        echo "  rpmbuild: available"
    fi
fi

echo -e "${GREEN}Prerequisites OK${NC}"
echo ""

# Install frontend dependencies
echo -e "${YELLOW}Installing frontend dependencies...${NC}"
npm ci
echo -e "${GREEN}Dependencies installed${NC}"
echo ""

# Build frontend
echo -e "${YELLOW}Building frontend...${NC}"
npm run build
echo -e "${GREEN}Frontend built${NC}"
echo ""

# Build Tauri app
echo -e "${YELLOW}Building Tauri application...${NC}"

BUILD_ARGS=""
if [ "$RELEASE_MODE" = true ]; then
    BUILD_ARGS="--release"
fi

if [ "$VERBOSE" = true ]; then
    BUILD_ARGS="$BUILD_ARGS --verbose"
fi

# Build specific targets
BUNDLE_TARGETS=""
if [ "$BUILD_DEB" = true ]; then
    BUNDLE_TARGETS="${BUNDLE_TARGETS},deb"
fi
if [ "$BUILD_APPIMAGE" = true ]; then
    BUNDLE_TARGETS="${BUNDLE_TARGETS},appimage"
fi
if [ "$BUILD_RPM" = true ]; then
    BUNDLE_TARGETS="${BUNDLE_TARGETS},rpm"
fi

# Remove leading comma
BUNDLE_TARGETS="${BUNDLE_TARGETS#,}"

if [ -n "$BUNDLE_TARGETS" ]; then
    BUILD_ARGS="$BUILD_ARGS --bundles $BUNDLE_TARGETS"
fi

npx tauri build $BUILD_ARGS

echo -e "${GREEN}Tauri build complete${NC}"
echo ""

# Get build artifact paths
if [ "$RELEASE_MODE" = true ]; then
    BUNDLE_DIR="$PROJECT_ROOT/src-tauri/target/release/bundle"
else
    BUNDLE_DIR="$PROJECT_ROOT/src-tauri/target/debug/bundle"
fi

# Output summary
echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}  Build Complete!${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "Build artifacts:"

if [ "$BUILD_DEB" = true ] && [ -d "$BUNDLE_DIR/deb" ]; then
    echo ""
    echo "  .deb packages:"
    find "$BUNDLE_DIR/deb" -name "*.deb" -exec echo "    {}" \;
fi

if [ "$BUILD_APPIMAGE" = true ] && [ -d "$BUNDLE_DIR/appimage" ]; then
    echo ""
    echo "  AppImage:"
    find "$BUNDLE_DIR/appimage" -name "*.AppImage" -exec echo "    {}" \;
fi

if [ "$BUILD_RPM" = true ] && [ -d "$BUNDLE_DIR/rpm" ]; then
    echo ""
    echo "  .rpm packages:"
    find "$BUNDLE_DIR/rpm" -name "*.rpm" -exec echo "    {}" \;
fi

echo ""
echo "Installation instructions:"

if [ "$BUILD_DEB" = true ]; then
    echo ""
    echo "  Debian/Ubuntu (.deb):"
    echo "    sudo dpkg -i path/to/nostr-nations_*.deb"
    echo "    sudo apt-get install -f  # Install dependencies if needed"
fi

if [ "$BUILD_APPIMAGE" = true ]; then
    echo ""
    echo "  AppImage (Universal):"
    echo "    chmod +x path/to/nostr-nations_*.AppImage"
    echo "    ./nostr-nations_*.AppImage"
fi

if [ "$BUILD_RPM" = true ]; then
    echo ""
    echo "  Fedora/RHEL (.rpm):"
    echo "    sudo rpm -i path/to/nostr-nations-*.rpm"
    echo "    # Or: sudo dnf install path/to/nostr-nations-*.rpm"
fi
