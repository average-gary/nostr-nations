#
# Nostr Nations - Windows Build Script
# Builds the application for Windows with optional code signing
#
# Usage:
#   .\scripts\build-windows.ps1 [-Release] [-Sign] [-Verbose]
#
# Environment variables:
#   WINDOWS_CERTIFICATE_PATH      - Path to .pfx certificate file
#   WINDOWS_CERTIFICATE_PASSWORD  - Certificate password
#   WINDOWS_SIGN_TOOL_PATH        - Path to signtool.exe (optional)
#

param(
    [switch]$Release,
    [switch]$Sign,
    [switch]$Verbose,
    [switch]$Help
)

# Strict mode
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# Colors
function Write-ColorOutput($ForegroundColor) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    if ($args) {
        Write-Output $args
    }
    $host.UI.RawUI.ForegroundColor = $fc
}

function Write-Info($message) { Write-Host $message -ForegroundColor Cyan }
function Write-Success($message) { Write-Host $message -ForegroundColor Green }
function Write-Warning($message) { Write-Host $message -ForegroundColor Yellow }
function Write-Error($message) { Write-Host $message -ForegroundColor Red }

# Show help
if ($Help) {
    Write-Host @"
Nostr Nations - Windows Build Script

Usage:
    .\scripts\build-windows.ps1 [-Release] [-Sign] [-Verbose] [-Help]

Options:
    -Release    Build in release mode (optimized)
    -Sign       Sign the application with a certificate
    -Verbose    Show detailed output
    -Help       Show this help message

Environment Variables (for signing):
    WINDOWS_CERTIFICATE_PATH      Path to .pfx certificate file
    WINDOWS_CERTIFICATE_PASSWORD  Certificate password
    WINDOWS_SIGN_TOOL_PATH        Path to signtool.exe (auto-detected if not set)

Examples:
    # Development build
    .\scripts\build-windows.ps1

    # Release build with signing
    `$env:WINDOWS_CERTIFICATE_PATH = "C:\certs\certificate.pfx"
    `$env:WINDOWS_CERTIFICATE_PASSWORD = "your-password"
    .\scripts\build-windows.ps1 -Release -Sign
"@
    exit 0
}

Write-Host "========================================"  -ForegroundColor Blue
Write-Host "  Nostr Nations - Windows Build"         -ForegroundColor Blue
Write-Host "========================================"  -ForegroundColor Blue
Write-Host ""

# Get script and project paths
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir

# Change to project root
Set-Location $ProjectRoot

# Check prerequisites
Write-Info "Checking prerequisites..."

# Check for Rust
try {
    $rustVersion = rustc --version
    Write-Host "  Rust: $rustVersion"
} catch {
    Write-Error "Error: Rust is not installed. Install from https://rustup.rs"
    exit 1
}

# Check for Node.js
try {
    $nodeVersion = node --version
    Write-Host "  Node.js: $nodeVersion"
} catch {
    Write-Error "Error: Node.js is not installed."
    exit 1
}

# Check for Visual Studio Build Tools
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vsWhere) {
    $vsPath = & $vsWhere -latest -property installationPath
    if ($vsPath) {
        Write-Host "  Visual Studio: Found at $vsPath"
    }
} else {
    Write-Warning "Warning: Visual Studio Build Tools may not be installed."
    Write-Warning "If build fails, install Visual Studio Build Tools with C++ workload."
}

Write-Success "Prerequisites OK"
Write-Host ""

# Install frontend dependencies
Write-Info "Installing frontend dependencies..."
npm ci
if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to install npm dependencies"
    exit 1
}
Write-Success "Dependencies installed"
Write-Host ""

# Build frontend
Write-Info "Building frontend..."
npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to build frontend"
    exit 1
}
Write-Success "Frontend built"
Write-Host ""

# Build Tauri app
Write-Info "Building Tauri application..."

$buildArgs = @()
if ($Release) {
    $buildArgs += "--release"
}
if ($Verbose) {
    $buildArgs += "--verbose"
}

# Run Tauri build
npx tauri build @buildArgs
if ($LASTEXITCODE -ne 0) {
    Write-Error "Tauri build failed"
    exit 1
}

Write-Success "Tauri build complete"
Write-Host ""

# Get build artifact paths
if ($Release) {
    $buildDir = "$ProjectRoot\src-tauri\target\release"
    $bundleDir = "$buildDir\bundle"
} else {
    $buildDir = "$ProjectRoot\src-tauri\target\debug"
    $bundleDir = "$buildDir\bundle"
}

$exePath = "$bundleDir\nsis\Nostr Nations_*_x64-setup.exe"
$msiPath = "$bundleDir\msi\Nostr Nations_*_x64_en-US.msi"

# Code signing
if ($Sign) {
    Write-Info "Signing application..."
    
    # Check for certificate
    if (-not $env:WINDOWS_CERTIFICATE_PATH) {
        Write-Error "Error: WINDOWS_CERTIFICATE_PATH environment variable is not set"
        Write-Host "Set it to the path of your .pfx certificate file."
        exit 1
    }
    
    if (-not (Test-Path $env:WINDOWS_CERTIFICATE_PATH)) {
        Write-Error "Error: Certificate file not found: $env:WINDOWS_CERTIFICATE_PATH"
        exit 1
    }
    
    if (-not $env:WINDOWS_CERTIFICATE_PASSWORD) {
        Write-Error "Error: WINDOWS_CERTIFICATE_PASSWORD environment variable is not set"
        exit 1
    }
    
    # Find signtool
    $signTool = $env:WINDOWS_SIGN_TOOL_PATH
    if (-not $signTool) {
        # Try to find signtool in Windows SDK
        $sdkPaths = @(
            "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe",
            "${env:ProgramFiles(x86)}\Windows Kits\8.1\bin\x64\signtool.exe"
        )
        
        foreach ($pattern in $sdkPaths) {
            $found = Get-ChildItem -Path $pattern -ErrorAction SilentlyContinue | 
                     Sort-Object -Descending | 
                     Select-Object -First 1
            if ($found) {
                $signTool = $found.FullName
                break
            }
        }
    }
    
    if (-not $signTool -or -not (Test-Path $signTool)) {
        Write-Error "Error: signtool.exe not found. Install Windows SDK or set WINDOWS_SIGN_TOOL_PATH"
        exit 1
    }
    
    Write-Host "Using signtool: $signTool"
    
    # Sign each artifact
    $artifacts = @()
    $artifacts += Get-ChildItem -Path $exePath -ErrorAction SilentlyContinue
    $artifacts += Get-ChildItem -Path $msiPath -ErrorAction SilentlyContinue
    
    foreach ($artifact in $artifacts) {
        Write-Host "Signing: $($artifact.Name)"
        
        & $signTool sign /f $env:WINDOWS_CERTIFICATE_PATH `
                        /p $env:WINDOWS_CERTIFICATE_PASSWORD `
                        /tr http://timestamp.digicert.com `
                        /td sha256 `
                        /fd sha256 `
                        $artifact.FullName
        
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Failed to sign $($artifact.Name)"
            exit 1
        }
        
        # Verify signature
        & $signTool verify /pa $artifact.FullName
    }
    
    Write-Success "Application signed successfully"
    Write-Host ""
}

# Output summary
Write-Host "========================================" -ForegroundColor Blue
Write-Success "  Build Complete!"
Write-Host "========================================" -ForegroundColor Blue
Write-Host ""
Write-Host "Build artifacts:"

$nsisFiles = Get-ChildItem -Path "$bundleDir\nsis\*.exe" -ErrorAction SilentlyContinue
if ($nsisFiles) {
    Write-Host "  NSIS Installer:"
    foreach ($file in $nsisFiles) {
        Write-Host "    $($file.FullName)"
    }
}

$msiFiles = Get-ChildItem -Path "$bundleDir\msi\*.msi" -ErrorAction SilentlyContinue
if ($msiFiles) {
    Write-Host "  MSI Installer:"
    foreach ($file in $msiFiles) {
        Write-Host "    $($file.FullName)"
    }
}

Write-Host ""

if (-not $Sign) {
    Write-Warning "Note: Application is not code signed."
    Write-Host "For distribution, run with -Sign flag and set certificate environment variables."
}
