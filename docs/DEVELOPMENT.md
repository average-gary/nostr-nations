# Development Guide

## Prerequisites

### Required Tools

| Tool          | Version | Installation                      |
| ------------- | ------- | --------------------------------- |
| **Rust**      | 1.70+   | [rustup.rs](https://rustup.rs/)   |
| **Node.js**   | 18+     | [nodejs.org](https://nodejs.org/) |
| **Tauri CLI** | 2.0+    | `cargo install tauri-cli`         |

### Platform-Specific Requirements

**macOS:**

```bash
xcode-select --install
```

**Linux (Ubuntu/Debian):**

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

**Windows:**

- Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- Install [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

## Setup Instructions

### 1. Clone the Repository

```bash
git clone https://github.com/nostr-nations/nostr-nations.git
cd nostr-nations
```

### 2. Install Rust Dependencies

```bash
# Install Rust toolchain (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Tauri CLI
cargo install tauri-cli
```

### 3. Install Node.js Dependencies

```bash
npm install
```

### 4. Verify Setup

```bash
# Check Rust
cargo --version
rustc --version

# Check Node
node --version
npm --version

# Check Tauri
cargo tauri --version
```

## Running the Project

### Development Mode

Run the full application with hot-reloading:

```bash
cargo tauri dev
```

This will:

1. Build the Rust backend
2. Start the Vite dev server for the frontend
3. Launch the Tauri application window
4. Enable hot-reload for frontend changes

### Frontend Only

To develop the frontend without Tauri:

```bash
npm run dev
```

Open http://localhost:5173 in your browser. Note: Tauri commands will be unavailable.

### Backend Only

To build/test just the Rust crates:

```bash
# Build all crates
cargo build

# Build specific crate
cargo build -p nostr-nations-core
```

## Running Tests

### Rust Tests

```bash
# Run all Rust tests
cargo test

# Run tests for specific crate
cargo test -p nostr-nations-core

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_combat_resolution
```

### Frontend Tests

```bash
# Run frontend tests (if configured)
npm test
```

### Coverage

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

## Building for Production

### Development Build

```bash
cargo tauri build --debug
```

### Release Build

```bash
cargo tauri build
```

Built applications are located in:

- **macOS**: `target/release/bundle/dmg/`
- **Linux**: `target/release/bundle/deb/` or `appimage/`
- **Windows**: `target/release/bundle/msi/`

## Code Style Guidelines

### Rust

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` before committing
- Use `cargo clippy` to catch common issues
- Document public APIs with doc comments

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings
```

### TypeScript/React

- Use ESLint and Prettier for formatting
- Follow React hooks conventions
- Use TypeScript strict mode

```bash
# Lint
npm run lint

# Fix linting issues
npm run lint:fix

# Format
npm run format

# Check formatting
npm run format:check
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add technology tree visualization
fix: correct combat damage calculation
docs: update API documentation
refactor: simplify map generation algorithm
test: add unit tests for diplomacy system
```

## Project Structure Conventions

### Rust Crates

- Keep `nostr-nations-core` free of any UI/network dependencies
- Use feature flags for optional functionality
- Public API should be re-exported from `lib.rs`

### Frontend Components

- One component per file
- Use functional components with hooks
- Co-locate styles and tests with components

### State Management

- Use Zustand for global state
- Keep Tauri IPC in dedicated hooks
- Derive computed values, don't store them

## PR/Contribution Process

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
```

### 2. Make Changes

- Write tests for new functionality
- Update documentation as needed
- Ensure all tests pass

### 3. Commit

```bash
git add .
git commit -m "feat: description of changes"
```

### 4. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then open a Pull Request on GitHub.

### PR Requirements

- [ ] All tests pass (`cargo test` and `npm test`)
- [ ] Code is formatted (`cargo fmt` and `npm run format`)
- [ ] Linting passes (`cargo clippy` and `npm run lint`)
- [ ] Documentation updated if needed
- [ ] Meaningful commit messages

### Review Process

1. At least one maintainer approval required
2. All CI checks must pass
3. Squash merge preferred for feature branches

## Common Development Tasks

### Adding a New Tauri Command

1. Define the command in `src-tauri/src/commands/`:

```rust
#[tauri::command]
pub fn my_command(arg: String) -> Result<Response, AppError> {
    // Implementation
}
```

2. Register in `main.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    commands::my_command,
])
```

3. Call from frontend:

```typescript
import { invoke } from '@tauri-apps/api/core'

const result = await invoke('my_command', { arg: 'value' })
```

### Adding a New Tauri Event

1. Define payload in `src-tauri/src/events.rs`:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MyEventPayload {
    pub data: String,
}
```

2. Emit from backend:

```rust
app_handle.emit("my_event", payload)?;
```

3. Listen in frontend:

```typescript
import { listen } from '@tauri-apps/api/event'

await listen('my_event', (event) => {
  console.log(event.payload)
})
```

### Adding a New Unit Type

1. Add variant to `UnitType` enum in `crates/nostr-nations-core/src/unit.rs`
2. Define stats in the `stats()` method
3. Add to technology unlocks if applicable
4. Update frontend unit rendering

### Adding a New Technology

1. Add to appropriate era method in `crates/nostr-nations-core/src/technology.rs`:

```rust
self.add(
    Technology::new("tech_id", "Tech Name", Era::Ancient, 50)
        .with_prerequisites(&["prereq_tech"])
        .unlocks_units(&[UnitType::NewUnit])
        .with_quote("Flavor text"),
);
```

## Debugging

### Rust Backend

```bash
# Run with debug logging
RUST_LOG=debug cargo tauri dev

# Use println! for quick debugging (visible in terminal)
println!("Debug: {:?}", value);
```

### Frontend

```typescript
// Browser DevTools console
console.log('Debug:', value)

// React DevTools for component inspection
```

### Tauri DevTools

Press `Cmd+Option+I` (macOS) or `Ctrl+Shift+I` (Windows/Linux) to open DevTools in the Tauri window.

## Environment Variables

| Variable      | Description             | Default |
| ------------- | ----------------------- | ------- |
| `RUST_LOG`    | Rust logging level      | `info`  |
| `TAURI_DEBUG` | Enable Tauri debug mode | `false` |

## Resources

- [Tauri Documentation](https://tauri.app/v1/guides/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [React Documentation](https://react.dev/)
- [Three.js Documentation](https://threejs.org/docs/)
- [Nostr Protocol](https://github.com/nostr-protocol/nips)
- [Cashu Protocol](https://github.com/cashubtc/nuts)
