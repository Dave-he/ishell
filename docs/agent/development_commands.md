# Development Commands

Complete guide to building, testing, and developing iShell.

## Build Commands

```bash
# Debug build (default)
cargo build

# Optimized release build
cargo build --release

# Clean build artifacts
cargo clean

# Check compilation without building binary
cargo check
```

## Running the Application

```bash
# Development run with logging (recommended)
./run.sh

# Direct cargo run (uses RUST_LOG environment)
cargo run

# Run optimized build
cargo run --release
```

**Note**: `./run.sh` auto-compiles if binary missing, sets `RUST_LOG=info`, and provides usage tips.

## Code Quality

```bash
# Format code
cargo fmt

# Check formatting without modifying
cargo fmt --check

# Lint with Clippy
cargo clippy

# More verbose Clippy output
cargo clippy -- -D warnings
```

## Dependency Management

```bash
# Update to latest compatible versions
cargo update

# Show dependency tree
cargo tree

# Show only direct dependencies
cargo tree --depth 1

# Show outdated dependencies
cargo outdated
```

## Release Preparation

```bash
# Full clean rebuild for release
cargo clean && cargo build --release

# Verify formatting before commit
cargo fmt --check

# Run lints
cargo clippy -- -D warnings

# Build binary size after release
ls -lh target/release/ishell
```

## Expected Output

- Debug build: ~12MB binary
- Release build: ~3MB binary (optimized)
- Compile time: ~2 minutes (first) / ~2 seconds (incremental)

## Environment Variables

- `RUST_LOG` - Control logging level (`debug`, `info`, `warn`, `error`)
  - Example: `RUST_LOG=debug cargo run`

## Platform Targets

Cross-platform support is automatic via egui/eframe:
- macOS (Intel & Apple Silicon)
- Linux (X11 & Wayland)
- Windows
- BSD systems

No platform-specific commands needed.
