# Architecture

iShell's architecture follows egui's immediate mode GUI pattern with panel-based layout.

## Project Structure

```
src/
├── main.rs    (26 lines)  - Entry point, window config, icon loading
└── app.rs     (516 lines) - Application state, UI rendering, mock logic

assets/
└── icon.png   - Application icon (compile-time embedded)
```

**Current Status**: MVP fits in single file (542 lines total). For v0.2.0+, refactor into modules when exceeding 1000 lines.

## Core Data Structures

### App Struct (18 fields)

```rust
pub struct App {
    // Connection Management
    connections: Vec<Connection>,
    selected_connection: Option<usize>,
    show_new_connection: bool,

    // Connection Form
    new_conn_name: String,
    new_conn_host: String,
    new_conn_port: String,
    new_conn_user: String,
    new_conn_password: String,

    // Terminal State
    terminal_output: String,
    command_input: String,

    // AI Assistant
    ai_messages: Vec<(String, String)>,
    ai_input: String,
    ai_provider: AiProvider,

    // System Monitoring
    cpu_usage: f32,
    mem_usage: f32,
}
```

### Supporting Types

```rust
struct Connection {
    name: String,
    host: String,
    port: u16,
    username: String,
    connected: bool,
}

enum AiProvider {
    Ollama,
    OpenAI,
    Google,
}
```

## Architectural Patterns

### 1. Immediate Mode GUI (IMGUI)

- Single `update()` method called every frame (~10 FPS at 100ms intervals)
- State stored entirely in `App` struct
- UI redrawn completely each frame
- No retained widget state
- All rendering in `app.rs:402` (78% of file)

### 2. Panel-Based Layout System

egui panels for responsive UI:
- **TopBottomPanel::top** - Menu bar (File, Tools, Help)
- **SidePanel::left** - Connection list
- **SidePanel::right** - AI assistant
- **CentralPanel** - Terminal area
- **TopBottomPanel::bottom** - System monitor
- **Windows** - Modal dialogs (new connection form)

### 3. Stateful UI Components

Each panel manages:
- Input state (text fields, selections)
- Output state (terminal buffer, chat history)
- Computed state (dynamic CPU/memory via oscillating calculation)

### 4. Mock/Simulation Pattern

Functions simulate real functionality for MVP:
- `execute_command()` - Mock terminal commands
- `mock_ai_response()` - Pattern-matched AI responses
- Static system monitoring data

**Benefit**: Fully functional MVP without backend integration

### 5. Provider/Strategy Pattern

AI provider selection allows runtime switching via `match self.ai_provider`.

## Data Flow

```
User Input
    ↓
Terminal Commands → execute_command() → terminal_output
AI Messages → mock_ai_response() → ai_messages
Connections → Connection state → UI display
    ↓
Internal State
    ↓
CPU/Memory → Oscillating calculation → Monitor display
Selected Connection → Connection state → Terminal activation
Panel State → Layout management → Responsive rendering
```

## Key Functions

### `new()` - Initialization
- Creates demo "Demo Server" connection
- Initializes empty terminal and AI state
- Sets default provider (Ollama)

### `execute_command()` - Terminal Handler
Supported mock commands:
- `help` - Show available commands
- `clear` - Clear terminal
- `date` - Show current date/time (chrono)
- `whoami`, `pwd`, `ls` - Hardcoded responses
- `echo <text>` - Print text
- Default: "command not found"

### `mock_ai_response()` - AI Mock
Pattern-matched responses based on keywords:
- "find", "文件", "large" → Find large files command
- "backup", "备份", "script" → MySQL backup script
- "permission", "权限", "denied" → Permission fix guide
- Default: Generic response + provider name

### `update()` - Main Render Loop
- Updates dynamic state (oscillating CPU usage)
- Renders all panels and modal dialogs
- Requests repaint at 100ms intervals

## Extensibility Points

### Easy to Extend
1. **Terminal Commands** - Add cases to `execute_command()` match statement
2. **AI Responses** - Extend pattern matching in `mock_ai_response()`
3. **UI Panels** - Add new egui panel sections to `update()`
4. **Quick Actions** - Add buttons to quick action toolbar
5. **Monitoring** - Add new stats to system monitor panel
6. **Colors** - Modify color constants for theming

### Hard to Extend (Requires Refactoring)
1. Multiple Windows - Single-window design throughout
2. Persistence - No serialization framework
3. Real SSH - Requires async/await refactoring
4. Real APIs - Requires networking layer
5. Undo/Redo - No command pattern implementation
6. Plugin System - Monolithic structure

## Performance Characteristics

- **Frame Rate**: 10 FPS (100ms intervals) - adequate for terminal
- **Rendering**: Complete UI redraw each frame (IMGUI standard)
- **Memory**: Terminal output buffer grows without limit - needs scrollback management
- **Startup**: ~2 minutes (first build), ~2 seconds (incremental)

## Scalability Limitations

**Current Constraints**:
- Terminal output buffer unbounded
- Single SSH connection at a time
- All UI in single file
- No async support (blocks on I/O)

**Mitigation Strategies** (v0.2.0+):
- Implement terminal scrollback buffer (e.g., 10,000 lines max)
- Add connection tabs/multiple terminals
- Refactor into modules when code exceeds 1000 lines
- Implement tokio async for I/O

## Dependencies Architecture

```
egui (immediate mode GUI)
├── emath (2D math)
├── epaint (rendering)
└── ab_glyph (fonts)

eframe (window management)
├── egui (re-exported)
├── glow (OpenGL rendering)
└── glutin (windowing)
```

**Key**: All dependencies are pure Rust with cross-platform support handled automatically.
