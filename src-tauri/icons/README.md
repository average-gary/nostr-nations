# Nostr Nations Icon Assets

This directory contains all icon assets required for building Nostr Nations across all supported platforms.

## Required Icon Files

### Source Image

Start with a high-resolution PNG (1024x1024 or larger) with transparency support.

### Generated Icons

| File             | Size       | Platform | Description                             |
| ---------------- | ---------- | -------- | --------------------------------------- |
| `32x32.png`      | 32x32 px   | All      | Small icon for taskbars/menus           |
| `128x128.png`    | 128x128 px | All      | Standard application icon               |
| `128x128@2x.png` | 256x256 px | macOS    | Retina display icon                     |
| `icon.png`       | 512x512 px | Linux    | Default Linux icon                      |
| `icon.icns`      | Multiple   | macOS    | macOS icon bundle (contains 16-512px)   |
| `icon.ico`       | Multiple   | Windows  | Windows icon bundle (contains 16-256px) |

### Optional High-Resolution Icons

| File          | Size       | Platform      | Description         |
| ------------- | ---------- | ------------- | ------------------- |
| `256x256.png` | 256x256 px | Windows/Linux | High-DPI displays   |
| `512x512.png` | 512x512 px | Linux         | Large icon displays |

## Generating Icons

### Using Tauri CLI (Recommended)

```bash
# From project root, generate all icons from a source image
npx tauri icon src-tauri/icons/source-icon.png
```

This will automatically generate all required icon sizes and formats.

### Manual Generation

#### macOS (.icns)

```bash
# Create iconset directory
mkdir icon.iconset

# Generate required sizes
sips -z 16 16     source.png --out icon.iconset/icon_16x16.png
sips -z 32 32     source.png --out icon.iconset/icon_16x16@2x.png
sips -z 32 32     source.png --out icon.iconset/icon_32x32.png
sips -z 64 64     source.png --out icon.iconset/icon_32x32@2x.png
sips -z 128 128   source.png --out icon.iconset/icon_128x128.png
sips -z 256 256   source.png --out icon.iconset/icon_128x128@2x.png
sips -z 256 256   source.png --out icon.iconset/icon_256x256.png
sips -z 512 512   source.png --out icon.iconset/icon_256x256@2x.png
sips -z 512 512   source.png --out icon.iconset/icon_512x512.png
sips -z 1024 1024 source.png --out icon.iconset/icon_512x512@2x.png

# Convert to icns
iconutil -c icns icon.iconset -o icon.icns
```

#### Windows (.ico)

Use ImageMagick:

```bash
convert source.png -define icon:auto-resize=256,128,64,48,32,16 icon.ico
```

Or use online tools like:

- https://icoconvert.com/
- https://convertio.co/png-ico/

#### Linux PNGs

```bash
# Using ImageMagick
convert source.png -resize 32x32 32x32.png
convert source.png -resize 128x128 128x128.png
convert source.png -resize 256x256 256x256.png
convert source.png -resize 512x512 512x512.png
```

## Icon Design Guidelines

### General

- Use a simple, recognizable design that works at small sizes
- Ensure good contrast for visibility
- Include transparency where appropriate
- Test at multiple sizes before finalizing

### macOS

- Follow Apple Human Interface Guidelines
- Use subtle shadows and gradients
- Avoid text in icons
- Consider the rounded rectangle mask applied by macOS

### Windows

- Follow Microsoft Fluent Design guidelines
- Ensure icon is visible on both light and dark taskbars
- Include 16x16 for system tray visibility

### Linux

- Follow freedesktop.org icon theme specification
- Provide SVG if possible for scalability
- Test with various desktop environments (GNOME, KDE, etc.)

## Current Status

- [x] `icon.png` - Source/placeholder icon
- [ ] `32x32.png` - Generate from source
- [ ] `128x128.png` - Generate from source
- [ ] `128x128@2x.png` - Generate from source
- [ ] `icon.icns` - Generate for macOS
- [ ] `icon.ico` - Generate for Windows

## Quick Start

1. Create or obtain a 1024x1024 PNG source icon
2. Place it in this directory as `source-icon.png`
3. Run: `npx tauri icon src-tauri/icons/source-icon.png`
4. Verify all icons are generated correctly
5. Test the build on each platform
