# Nostr Nations Deployment Guide

This guide covers building and deploying the Nostr Nations Tauri application for macOS, Windows, and Linux.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Development Build](#development-build)
3. [Production Build](#production-build)
4. [macOS Deployment](#macos-deployment)
5. [Windows Deployment](#windows-deployment)
6. [Linux Deployment](#linux-deployment)
7. [Cross-Compilation](#cross-compilation)
8. [Environment Variables](#environment-variables)
9. [Troubleshooting](#troubleshooting)
10. [Release Checklist](#release-checklist)

---

## Prerequisites

### Rust Toolchain

- **Minimum Version**: Rust 1.70.0 or later (2021 edition required)
- **Recommended**: Latest stable release

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version

# Update to latest stable
rustup update stable
```

### Node.js

- **Minimum Version**: Node.js 18.x LTS
- **Recommended**: Node.js 20.x LTS or later

```bash
# Verify installation
node --version  # Should be >= 18.0.0
npm --version   # Should be >= 9.0.0
```

### Tauri CLI

The Tauri CLI is included as a dev dependency. You can also install it globally:

```bash
# Install via cargo (recommended for production builds)
cargo install tauri-cli

# Or use the npm package (included in devDependencies)
npm run tauri -- --version
```

### Platform-Specific Dependencies

#### macOS

```bash
# Xcode Command Line Tools (required)
xcode-select --install

# Verify installation
clang --version
```

**Requirements:**

- macOS 10.15 (Catalina) or later
- Xcode Command Line Tools
- For code signing: Apple Developer account

#### Windows

**Requirements:**

- Windows 10/11 (64-bit)
- Visual Studio 2019 or later with "Desktop development with C++" workload
- WebView2 (included in Windows 11, auto-installed on Windows 10)

```powershell
# Install Visual Studio Build Tools (if not using full Visual Studio)
winget install Microsoft.VisualStudio.2022.BuildTools

# During installation, select:
# - "Desktop development with C++"
# - Windows 10/11 SDK
```

#### Linux

**Debian/Ubuntu:**

```bash
sudo apt update
sudo apt install -y \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libwebkit2gtk-4.1-dev \
    libjavascriptcoregtk-4.1-dev \
    libsoup-3.0-dev
```

**Fedora:**

```bash
sudo dnf install -y \
    gcc-c++ \
    webkit2gtk4.1-devel \
    openssl-devel \
    curl \
    wget \
    file \
    libappindicator-gtk3-devel \
    librsvg2-devel
```

**Arch Linux:**

```bash
sudo pacman -S --needed \
    base-devel \
    curl \
    wget \
    file \
    openssl \
    webkit2gtk-4.1 \
    libappindicator-gtk3 \
    librsvg
```

---

## Development Build

### Running in Dev Mode

```bash
# Install dependencies first
npm install

# Start development server with hot reload
npm run tauri dev

# Or using cargo directly
cargo tauri dev
```

### Hot Reload Behavior

- **Frontend changes**: Vite provides instant hot module replacement (HMR) for React/TypeScript changes
- **Rust changes**: The Tauri backend will automatically recompile when Rust source files change
- **Tauri config changes**: Require a full restart of the dev server

### Debugging Tips

**Frontend Debugging:**

```bash
# Open DevTools in the app window
# macOS: Cmd + Option + I
# Windows/Linux: Ctrl + Shift + I

# Or enable DevTools by default in tauri.conf.json:
# "app": { "windows": [{ "devtools": true }] }
```

**Rust Backend Debugging:**

```bash
# Enable debug logging
RUST_LOG=debug npm run tauri dev

# For verbose Tauri logging
RUST_LOG=tauri=debug npm run tauri dev

# Debug with LLDB (macOS)
cargo tauri dev --debug
lldb target/debug/nostr-nations-tauri

# Debug with GDB (Linux)
gdb target/debug/nostr-nations-tauri
```

**VS Code Debug Configuration:**

Add to `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Tauri",
      "cargo": {
        "args": ["build", "--manifest-path=./src-tauri/Cargo.toml"]
      },
      "preLaunchTask": "npm: dev"
    }
  ]
}
```

---

## Production Build

### Building for Current Platform

```bash
# Build optimized production binary
npm run tauri build

# Or using cargo directly
cargo tauri build

# Build with verbose output
cargo tauri build --verbose

# Build specific bundle formats only
cargo tauri build --bundles deb,appimage  # Linux
cargo tauri build --bundles dmg,app       # macOS
cargo tauri build --bundles msi,nsis      # Windows
```

### Build Output Locations

After a successful build, artifacts are located in:

```
src-tauri/target/release/
├── nostr-nations-tauri          # Binary (Linux/macOS)
├── nostr-nations-tauri.exe      # Binary (Windows)
└── bundle/
    ├── macos/
    │   ├── Nostr Nations.app    # Application bundle
    │   └── Nostr Nations.dmg    # Disk image
    ├── dmg/
    │   └── Nostr Nations_0.1.0_x64.dmg
    ├── deb/
    │   └── nostr-nations_0.1.0_amd64.deb
    ├── appimage/
    │   └── nostr-nations_0.1.0_amd64.AppImage
    ├── rpm/
    │   └── nostr-nations-0.1.0-1.x86_64.rpm
    ├── msi/
    │   └── Nostr Nations_0.1.0_x64_en-US.msi
    └── nsis/
        └── Nostr Nations_0.1.0_x64-setup.exe
```

### Code Signing Requirements

Code signing is required for:

- macOS: Distribution outside the App Store (Gatekeeper)
- Windows: Avoiding SmartScreen warnings
- Linux: Optional, but recommended for package repositories

---

## macOS Deployment

### Building .app and .dmg

```bash
# Build all macOS bundles
cargo tauri build --bundles app,dmg

# Build only .app
cargo tauri build --bundles app

# Build only .dmg
cargo tauri build --bundles dmg
```

### Code Signing with Developer ID

**Prerequisites:**

- Apple Developer Program membership ($99/year)
- Developer ID Application certificate
- Developer ID Installer certificate (for pkg)

**Setup:**

```bash
# List available signing identities
security find-identity -v -p codesigning

# Set environment variables
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)"
export APPLE_CERTIFICATE="Developer ID Application: Your Name (TEAM_ID)"
```

**Configure in `tauri.conf.json`:**

```json
{
  "bundle": {
    "macOS": {
      "signingIdentity": "Developer ID Application: Your Name (TEAM_ID)",
      "entitlements": "./entitlements.plist"
    }
  }
}
```

**Create `entitlements.plist`:**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>
    <key>com.apple.security.network.client</key>
    <true/>
    <key>com.apple.security.network.server</key>
    <true/>
</dict>
</plist>
```

### Notarization Process

**Prerequisites:**

- App-specific password from Apple ID
- Apple Developer Team ID

```bash
# Store credentials in keychain
xcrun notarytool store-credentials "notarytool-profile" \
    --apple-id "your@email.com" \
    --team-id "YOUR_TEAM_ID" \
    --password "app-specific-password"

# Build and notarize
cargo tauri build

# Manual notarization (if needed)
xcrun notarytool submit "target/release/bundle/dmg/Nostr Nations_0.1.0_x64.dmg" \
    --keychain-profile "notarytool-profile" \
    --wait

# Staple the notarization ticket
xcrun stapler staple "target/release/bundle/dmg/Nostr Nations_0.1.0_x64.dmg"
```

**Configure automatic notarization in `tauri.conf.json`:**

```json
{
  "bundle": {
    "macOS": {
      "signingIdentity": "Developer ID Application: Your Name (TEAM_ID)",
      "notarization": {
        "teamId": "YOUR_TEAM_ID"
      }
    }
  }
}
```

Set environment variables for CI:

```bash
export APPLE_ID="your@email.com"
export APPLE_PASSWORD="app-specific-password"
export APPLE_TEAM_ID="YOUR_TEAM_ID"
```

### Gatekeeper Considerations

- **Unsigned apps**: Users must right-click and select "Open" to bypass Gatekeeper
- **Signed but not notarized**: macOS may still block; users need to allow in Security settings
- **Signed and notarized**: App opens without warnings

**Testing Gatekeeper:**

```bash
# Check code signature
codesign -dv --verbose=4 "Nostr Nations.app"

# Verify notarization
spctl -a -vv "Nostr Nations.app"

# Check Gatekeeper assessment
spctl --assess --verbose "Nostr Nations.app"
```

---

## Windows Deployment

### Building .exe and .msi

```bash
# Build all Windows bundles
cargo tauri build --bundles msi,nsis

# Build only MSI installer
cargo tauri build --bundles msi

# Build only NSIS installer (exe)
cargo tauri build --bundles nsis
```

### Code Signing with Authenticode

**Prerequisites:**

- Code signing certificate (.pfx file) from a trusted CA
- Windows SDK signtool.exe

**Environment Variables:**

```powershell
# Certificate file path
$env:TAURI_SIGNING_PRIVATE_KEY_PATH = "C:\path\to\certificate.pfx"
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "certificate-password"

# Or use thumbprint for certificates in Windows certificate store
$env:TAURI_SIGNING_CERTIFICATE_THUMBPRINT = "YOUR_CERT_THUMBPRINT"
```

**Configure in `tauri.conf.json`:**

```json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": "YOUR_CERT_THUMBPRINT",
      "timestampUrl": "http://timestamp.digicert.com"
    }
  }
}
```

**Manual Signing:**

```powershell
# Sign using signtool
signtool sign /f certificate.pfx /p password /t http://timestamp.digicert.com /v "Nostr Nations_0.1.0_x64-setup.exe"

# Verify signature
signtool verify /pa "Nostr Nations_0.1.0_x64-setup.exe"
```

### Windows Defender SmartScreen

- **Unsigned apps**: SmartScreen will show "Windows protected your PC" warning
- **Signed with standard certificate**: May still show warnings until reputation is built
- **EV (Extended Validation) certificate**: Immediate SmartScreen trust

**Building Reputation:**

- Use an EV code signing certificate for immediate trust
- Standard certificates build reputation over time as more users install
- Consider submitting to Microsoft for analysis: https://www.microsoft.com/wdsi/filesubmission

---

## Linux Deployment

### Building .deb, .AppImage, .rpm

```bash
# Build all Linux bundles
cargo tauri build --bundles deb,appimage,rpm

# Build individual formats
cargo tauri build --bundles deb      # Debian/Ubuntu
cargo tauri build --bundles rpm      # Fedora/RHEL
cargo tauri build --bundles appimage # Universal
```

### Desktop Integration

Tauri automatically generates `.desktop` files. Customize in `tauri.conf.json`:

```json
{
  "bundle": {
    "linux": {
      "deb": {
        "depends": ["libwebkit2gtk-4.1-0", "libgtk-3-0"],
        "section": "games",
        "priority": "optional"
      },
      "rpm": {
        "release": "1",
        "epoch": "0"
      }
    },
    "category": "Game",
    "shortDescription": "Civilization-style 4X strategy game on Nostr"
  }
}
```

**Desktop Entry (`nostr-nations.desktop`):**

```ini
[Desktop Entry]
Name=Nostr Nations
Comment=Civilization-style 4X strategy game on Nostr
Exec=nostr-nations
Icon=nostr-nations
Terminal=false
Type=Application
Categories=Game;StrategyGame;
Keywords=nostr;strategy;4x;civilization;
```

### Dependencies

**Runtime Dependencies (Debian/Ubuntu):**

```
libwebkit2gtk-4.1-0
libgtk-3-0
libayatana-appindicator3-1
```

**Runtime Dependencies (Fedora):**

```
webkit2gtk4.1
gtk3
libappindicator-gtk3
```

---

## Cross-Compilation

### Building for Other Platforms

Cross-compilation is limited for Tauri due to platform-specific dependencies. Recommended approaches:

**Option 1: Native Builds via CI/CD**

Use GitHub Actions or similar to build on native runners:

```yaml
# .github/workflows/release.yml
jobs:
  build:
    strategy:
      matrix:
        include:
          - platform: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - platform: macos-latest
            target: x86_64-apple-darwin
          - platform: macos-latest
            target: aarch64-apple-darwin
          - platform: windows-latest
            target: x86_64-pc-windows-msvc
    runs-on: ${{ matrix.platform }}
```

**Option 2: Cross for Linux Targets**

```bash
# Install cross
cargo install cross

# Build for different Linux architectures
cross build --release --target aarch64-unknown-linux-gnu
```

**Option 3: Docker for Linux Builds**

```dockerfile
FROM rust:latest

RUN apt-get update && apt-get install -y \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev

WORKDIR /app
COPY . .
RUN cargo tauri build
```

### CI/CD Considerations

**GitHub Actions Example:**

```yaml
name: Build and Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: ubuntu-22.04
            args: ''
          - platform: macos-latest
            args: '--target universal-apple-darwin'
          - platform: windows-latest
            args: ''

    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 20

      - name: Install Rust stable
        uses: dtolnay/rust-action@stable

      - name: Install dependencies (Ubuntu)
        if: matrix.platform == 'ubuntu-22.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

      - name: Install frontend dependencies
        run: npm ci

      - name: Build Tauri app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
        with:
          tagName: v__VERSION__
          releaseName: 'Nostr Nations v__VERSION__'
          releaseBody: 'See the changelog for details.'
          releaseDraft: true
          prerelease: false
          args: ${{ matrix.args }}
```

---

## Environment Variables

### Required for Build

| Variable                             | Description                            | Required         |
| ------------------------------------ | -------------------------------------- | ---------------- |
| `TAURI_SIGNING_PRIVATE_KEY`          | Base64-encoded private key for updates | For auto-updates |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password for the private key           | For auto-updates |

### macOS Code Signing

| Variable                     | Description                     |
| ---------------------------- | ------------------------------- |
| `APPLE_CERTIFICATE`          | Base64-encoded .p12 certificate |
| `APPLE_CERTIFICATE_PASSWORD` | Certificate password            |
| `APPLE_SIGNING_IDENTITY`     | Signing identity name           |
| `APPLE_ID`                   | Apple ID email                  |
| `APPLE_PASSWORD`             | App-specific password           |
| `APPLE_TEAM_ID`              | Developer Team ID               |

### Windows Code Signing

| Variable                               | Description              |
| -------------------------------------- | ------------------------ |
| `TAURI_SIGNING_PRIVATE_KEY_PATH`       | Path to .pfx certificate |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`   | Certificate password     |
| `TAURI_SIGNING_CERTIFICATE_THUMBPRINT` | Certificate thumbprint   |

### Optional Configuration

| Variable      | Description              | Default |
| ------------- | ------------------------ | ------- |
| `RUST_LOG`    | Logging level            | `warn`  |
| `TAURI_DEBUG` | Enable debug mode        | `false` |
| `NO_STRIP`    | Disable binary stripping | `false` |

---

## Troubleshooting

### Common Build Errors

**Error: "failed to run custom build command for tauri"**

```bash
# Ensure all system dependencies are installed
# See Platform-Specific Dependencies section

# Clean and rebuild
cargo clean
npm run tauri build
```

**Error: "linking with cc failed"**

```bash
# Linux: Install build essentials
sudo apt install build-essential

# macOS: Install Xcode CLI tools
xcode-select --install
```

**Error: "WebView2 not found" (Windows)**

```powershell
# Download and install WebView2 Runtime
# https://developer.microsoft.com/en-us/microsoft-edge/webview2/
```

**Error: "npm run build failed"**

```bash
# Check for TypeScript errors
npm run build

# Clear Vite cache
rm -rf node_modules/.vite
npm run build
```

### Platform-Specific Issues

**macOS: "App is damaged and can't be opened"**

```bash
# Remove quarantine attribute
xattr -cr "/Applications/Nostr Nations.app"
```

**macOS: Code signing errors**

```bash
# Verify certificate is valid
security find-identity -v -p codesigning

# Check certificate expiration
security find-certificate -c "Developer ID Application" -p | openssl x509 -noout -dates
```

**Windows: NSIS installer fails**

```powershell
# Install NSIS
choco install nsis

# Or download from https://nsis.sourceforge.io/
```

**Linux: WebKitGTK version mismatch**

```bash
# Check installed version
pkg-config --modversion webkit2gtk-4.1

# Install correct version
sudo apt install libwebkit2gtk-4.1-dev
```

### Dependency Conflicts

**Rust dependency conflicts:**

```bash
# Update Cargo.lock
cargo update

# Check for conflicting versions
cargo tree -d
```

**Node.js dependency conflicts:**

```bash
# Clear npm cache
npm cache clean --force

# Remove and reinstall
rm -rf node_modules package-lock.json
npm install
```

**Bevy/Tauri conflicts:**

If Bevy and Tauri have conflicting dependencies:

```bash
# Check for multiple winit versions
cargo tree -i winit

# May need to align versions in Cargo.toml
```

---

## Release Checklist

### Version Bumping

Update version in all relevant files:

```bash
# Files to update:
# - package.json (version)
# - src-tauri/tauri.conf.json (version)
# - src-tauri/Cargo.toml (version)
# - Cargo.toml workspace (version)
```

**Automated version bump:**

```bash
# Using npm version (updates package.json)
npm version patch  # 0.1.0 -> 0.1.1
npm version minor  # 0.1.0 -> 0.2.0
npm version major  # 0.1.0 -> 1.0.0

# Manually sync to Cargo.toml and tauri.conf.json
```

### Changelog Updates

Maintain `CHANGELOG.md` following Keep a Changelog format:

```markdown
## [0.2.0] - 2024-XX-XX

### Added

- New feature description

### Changed

- Modified behavior description

### Fixed

- Bug fix description

### Security

- Security fix description
```

### Testing Requirements

Before release:

- [ ] All unit tests pass: `cargo test --workspace`
- [ ] Frontend tests pass: `npm test`
- [ ] Lint checks pass: `npm run lint && cargo clippy`
- [ ] Build succeeds on all platforms
- [ ] Manual testing of core functionality
- [ ] Test fresh installation on each platform
- [ ] Test upgrade from previous version
- [ ] Verify code signing and notarization

### Distribution Channels

**GitHub Releases:**

- Create release with tag `v0.1.0`
- Attach built artifacts for all platforms
- Include changelog in release notes

**Platform-Specific:**

- macOS: Consider Mac App Store (requires additional setup)
- Windows: Consider Microsoft Store
- Linux: Consider Flathub, Snapcraft, or AUR

**Auto-Update Configuration:**

Configure updater in `tauri.conf.json`:

```json
{
  "plugins": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://github.com/nostr-nations/nostr-nations/releases/latest/download/latest.json"
      ],
      "pubkey": "YOUR_PUBLIC_KEY"
    }
  }
}
```

Generate update signature keys:

```bash
# Generate keypair
cargo tauri signer generate -w ~/.tauri/nostr-nations.key

# The public key goes in tauri.conf.json
# Set TAURI_SIGNING_PRIVATE_KEY env var for CI
```

---

## Additional Resources

- [Tauri Documentation](https://tauri.app/v1/guides/)
- [Tauri GitHub](https://github.com/tauri-apps/tauri)
- [Apple Developer Documentation](https://developer.apple.com/documentation/)
- [Microsoft Code Signing](https://docs.microsoft.com/en-us/windows/win32/seccrypto/cryptography-tools)
- [Nostr Nations Repository](https://github.com/nostr-nations/nostr-nations)
