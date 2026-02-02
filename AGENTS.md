# AGENTS.md

This file provides guidance to codeflicker when working with code in this repository.

## WHY: Purpose and Goals

iShell is a cross-platform SSH terminal manager with integrated AI assistance. Built as an MVP (v0.1.0) using Rust and egui, it provides a modern GUI for managing SSH connections, terminal sessions, and AI-powered command assistance.

## WHAT: Technical Stack

- Runtime/Language: Rust 2021 Edition
- Framework: egui (immediate mode GUI) + eframe (window management)
- Key dependencies: chrono (timestamps), env_logger (debug logging)
- Architecture: Immediate mode GUI, panel-based layout, stateful UI components

## HOW: Core Development Workflow

```bash
# Development
./run.sh

# Building
cargo build --release

# Code Quality
cargo clippy
cargo fmt
```

## Progressive Disclosure

For detailed information, consult these documents as needed:

- `docs/agent/development_commands.md` - All build, test, lint, release commands
- `docs/agent/architecture.md` - Module structure and architectural patterns

**When working on a task, first determine which documentation is relevant, then read only those files.**
