#!/bin/bash
#
# Nostr Nations - macOS Build Script
# Builds the application for macOS with optional code signing
#
# Usage:
#   ./scripts/build-macos.sh [--release] [--sign] [--notarize]
#
# Environment variables:
#   APPLE_SIGNING_IDENTITY    - Code signing identity (e.g., "Developer ID Application: ...")
#   APPLE_ID                  - Apple ID for notarization
#   APPLE_TEAM_ID             - Apple Developer Team ID
#   APPLE_PASSWORD            - App-specific password for notarization
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
SIGN_APP=false
NOTARIZE_APP=false
VERBOSE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            RELEASE_MODE=true
            shift
            ;;
        --sign)
            SIGN_APP=true
            shift
            ;;
        --notarize)
            NOTARIZE_APP=true
            SIGN_APP=true  # Notarization requires signing
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [--release] [--sign] [--notarize] [--verbose]"
            echo ""
            echo "Options:"
            echo "  --release    Build in release mode (optimized)"
            echo "  --sign       Sign the application with Apple Developer ID"
            echo "  --notarize   Notarize the application (requires --sign)"
            echo "  --verbose    Show detailed output"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Nostr Nations - macOS Build${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Change to project root
cd "$PROJECT_ROOT"

# Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${NC}"

# Check for Rust
if ! command -v rustc &> /dev/null; then
    echo -e "${RED}Error: Rust is not installed. Install from https://rustup.rs${NC}"
    exit 1
fi

# Check for Node.js
if ! command -v node &> /dev/null; then
    echo -e "${RED}Error: Node.js is not installed.${NC}"
    exit 1
fi

# Check for Tauri CLI
if ! command -v cargo-tauri &> /dev/null && ! npx tauri --version &> /dev/null; then
    echo -e "${YELLOW}Installing Tauri CLI...${NC}"
    cargo install tauri-cli
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

# Set target for universal binary (optional)
# Uncomment for universal binary support:
# BUILD_ARGS="$BUILD_ARGS --target universal-apple-darwin"

npx tauri build $BUILD_ARGS

echo -e "${GREEN}Tauri build complete${NC}"
echo ""

# Get the app bundle path
if [ "$RELEASE_MODE" = true ]; then
    APP_BUNDLE="$PROJECT_ROOT/src-tauri/target/release/bundle/macos/Nostr Nations.app"
    DMG_PATH="$PROJECT_ROOT/src-tauri/target/release/bundle/dmg"
else
    APP_BUNDLE="$PROJECT_ROOT/src-tauri/target/debug/bundle/macos/Nostr Nations.app"
    DMG_PATH="$PROJECT_ROOT/src-tauri/target/debug/bundle/dmg"
fi

# Code signing
if [ "$SIGN_APP" = true ]; then
    echo -e "${YELLOW}Signing application...${NC}"
    
    if [ -z "${APPLE_SIGNING_IDENTITY:-}" ]; then
        echo -e "${RED}Error: APPLE_SIGNING_IDENTITY environment variable is not set${NC}"
        echo "Set it to your Developer ID Application certificate identity."
        echo "Example: export APPLE_SIGNING_IDENTITY=\"Developer ID Application: Your Name (TEAM_ID)\""
        exit 1
    fi
    
    # Sign the app bundle
    codesign --force --options runtime --sign "$APPLE_SIGNING_IDENTITY" \
        --deep --timestamp \
        "$APP_BUNDLE"
    
    # Verify signature
    codesign --verify --verbose "$APP_BUNDLE"
    
    echo -e "${GREEN}Application signed successfully${NC}"
    echo ""
fi

# Notarization
if [ "$NOTARIZE_APP" = true ]; then
    echo -e "${YELLOW}Notarizing application...${NC}"
    
    # Check required environment variables
    if [ -z "${APPLE_ID:-}" ] || [ -z "${APPLE_TEAM_ID:-}" ] || [ -z "${APPLE_PASSWORD:-}" ]; then
        echo -e "${RED}Error: Missing notarization credentials${NC}"
        echo "Required environment variables:"
        echo "  APPLE_ID       - Your Apple ID email"
        echo "  APPLE_TEAM_ID  - Your Apple Developer Team ID"
        echo "  APPLE_PASSWORD - App-specific password"
        exit 1
    fi
    
    # Create a zip for notarization
    NOTARIZE_ZIP="/tmp/nostr-nations-notarize.zip"
    ditto -c -k --keepParent "$APP_BUNDLE" "$NOTARIZE_ZIP"
    
    # Submit for notarization
    echo "Submitting for notarization..."
    xcrun notarytool submit "$NOTARIZE_ZIP" \
        --apple-id "$APPLE_ID" \
        --team-id "$APPLE_TEAM_ID" \
        --password "$APPLE_PASSWORD" \
        --wait
    
    # Staple the notarization ticket
    echo "Stapling notarization ticket..."
    xcrun stapler staple "$APP_BUNDLE"
    
    # Clean up
    rm -f "$NOTARIZE_ZIP"
    
    echo -e "${GREEN}Notarization complete${NC}"
    echo ""
fi

# Output summary
echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}  Build Complete!${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "Build artifacts:"
echo "  App Bundle: $APP_BUNDLE"
if [ -d "$DMG_PATH" ]; then
    echo "  DMG Installer: $DMG_PATH"
    ls -la "$DMG_PATH"/*.dmg 2>/dev/null || true
fi
echo ""

if [ "$SIGN_APP" = false ]; then
    echo -e "${YELLOW}Note: Application is not code signed.${NC}"
    echo "For distribution, run with --sign flag and set APPLE_SIGNING_IDENTITY."
fi

if [ "$NOTARIZE_APP" = false ] && [ "$SIGN_APP" = true ]; then
    echo -e "${YELLOW}Note: Application is not notarized.${NC}"
    echo "For App Store or Gatekeeper approval, run with --notarize flag."
fi
