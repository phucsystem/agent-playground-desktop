# Codebase Summary

**Version:** 0.2.0 | **Last Updated:** 2026-03-19

## Overview

Agent Playground Desktop is a Tauri v2 desktop shell wrapping a remote Next.js web app (Agent Playground) with native OS integration. Total codebase: 75 files, ~160k tokens.

## Directory Structure

```
agent-playground-desktop/
├── .claude/                    # AI task automation
│   └── skills/
├── .github/
│   ├── workflows/release.yml   # CI/CD: PR merge → build & release
│   └── CODEOWNERS
├── docs/                       # IPA + standard project docs
│   ├── SRD.md                 # System requirements
│   ├── API_SPEC.md            # Tauri IPC commands
│   ├── DB_DESIGN.md           # Data entities (AppConfig, WindowState)
│   ├── UI_SPEC.md             # UI components & screens
│   ├── project-overview-pdr.md # Project overview & PDR
│   ├── code-standards.md      # Code conventions & patterns
│   ├── system-architecture.md # Architecture diagrams
│   ├── project-roadmap.md     # Phases & progress
│   ├── deployment-guide.md    # Build & release instructions
│   └── codebase-summary.md    # This file
├── plans/                      # Implementation plans
│   ├── 260317-agent-playground-desktop/
│   │   ├── plan.md
│   │   ├── phase-01-core-shell.md
│   │   ├── phase-02-polish.md
│   │   └── phase-03-distribution.md
│   └── reports/
├── src/                        # Frontend (web context)
│   ├── bridge.js              # JS bridge (71 lines)
│   └── index.html             # Loading screen (37 lines)
├── src-tauri/                 # Rust backend
│   ├── src/
│   │   ├── main.rs            # Entry point (6 lines)
│   │   ├── lib.rs             # App setup (136 lines)
│   │   ├── commands.rs        # IPC handlers (230 lines)
│   │   ├── tray.rs            # System tray (106 lines)
│   │   └── updater.rs         # Auto-updater (98 lines)
│   ├── capabilities/          # Tauri permission ACL
│   │   ├── default.json
│   │   └── remote-access.json
│   ├── icons/                 # Multi-platform app icons
│   ├── Cargo.toml             # Rust dependencies
│   └── tauri.conf.json        # App configuration
├── scripts/
│   └── patch-remote-url.sh    # Build-time URL injection
├── package.json               # Node dependencies (npm)
└── README.md                  # Quick start guide
```

## Tech Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **Desktop Shell** | Tauri v2 (Rust) | Window, tray, native APIs |
| **Web Content** | Remote Next.js | Chat UI, business logic |
| **State Storage** | plugin-store (JSON) | Persistent config & window state |
| **Notifications** | plugin-notification | Native OS alerts |
| **Distribution** | GitHub Releases | Update delivery |
| **Build** | GitHub Actions | Cross-platform CI/CD |
| **Package Mgmt** | Cargo + npm | Rust + Node dependencies |

## Key Modules

### Rust Backend (src-tauri/src/)

**main.rs** (6 lines)
- Entry point, delegates to lib.rs

**lib.rs** (136 lines)
- App setup, plugin initialization
- Window creation with state restoration
- JS bridge injection via initialization_script
- Global shortcut handler
- URL resolution (env var → config → localhost:3000)

**commands.rs** (230 lines)
- 5 IPC commands:
  - `notify_new_message`: Send native notification
  - `update_badge_count`: Update tray tooltip
  - `report_user_active`: Suppress notifications for active conversation
  - `get_app_config` / `set_app_config`: Persistent preferences
- Data structures: AppConfig, WindowState, AppState
- Plugin-store integration

**tray.rs** (106 lines)
- System tray icon with context menu:
  - Show/Hide Window
  - Notifications toggle
  - Auto-start toggle
  - Check Updates
  - Quit
- Menu state tracking (visible/hidden, notifications on/off)

**updater.rs** (98 lines)
- Background update checking (on timer, 6-hour interval)
- Manual update checking from tray menu
- Dialog display with version info
- GitHub Releases integration

### Frontend (src/)

**bridge.js** (71 lines)
- Polls for `window.__TAURI__` (50 retries @ 100ms)
- Listens for 3 custom events:
  - `tauri:new-message` → invoke notify_new_message
  - `tauri:unread-count` → invoke update_badge_count
  - `tauri:active-conversation` → invoke report_user_active
- Forwards events to Rust IPC

**index.html** (37 lines)
- Loading screen (spinner + "Connecting...")
- Light/dark mode detection
- Loaded before web app page

### Configuration

**tauri.conf.json**
- Version: 0.2.0
- Window: 1200×800, min 800×600, centered, resizable
- CSP: null (for remote content)
- Plugins: notification, store, autostart, global-shortcut, updater, deep-link
- Capabilities: Remote URL whitelisting via capabilities/remote-access.json

**Capabilities**
- **default.json**: Local permissions (window, store, shortcuts, etc.)
- **remote-access.json**: Whitelists remote URLs for Tauri API access
  - Currently: localhost:3000 (dev) + https://* (production, patched at build time)

### Build & Release

**.github/workflows/release.yml**
- Trigger: PR merged to main (not tag push)
- Matrix: macOS universal (aarch64 + x86_64), Windows x86_64
- Version source: tauri.conf.json
- Outputs:
  - macOS: DMG installer
  - Windows: MSI + EXE
- Requires: AGENT_PLAYGROUND_URL, TAURI_SIGNING_PRIVATE_KEY, TAURI_SIGNING_PRIVATE_KEY_PASSWORD

**scripts/patch-remote-url.sh**
- Runs before build
- Injects production URL from AGENT_PLAYGROUND_URL env var into remote-access.json capabilities
- Ensures binary points to correct backend

## Feature Status

### Implemented (Phase 1-2)
- Native window with webview
- System tray with context menu
- Minimize-to-tray on close
- Native notifications with click handling
- JS bridge for webview ↔ Rust IPC
- Notification permission prompt + toggle
- Window state persistence (position, size, maximized)
- Auto-start on login (optional)
- Global shortcut (Cmd+Shift+A / Ctrl+Shift+A)
- Unread badge count (tray tooltip only)
- Auto-updater with GitHub Releases
- CI/CD pipeline (macOS + Windows)

### Partial
- Deep links (plugin initialized, not tested)
- Badge count (no dock badge, only tray tooltip)
- Loading screen (HTML only, no integration with page load events)

### Not Implemented
- Notification grouping
- macOS dock badge (Tauri API not available in v2.10)

## Plugins Used

| Plugin | Version | Purpose |
|--------|---------|---------|
| tauri-plugin-notification | 2 | Native OS notifications |
| tauri-plugin-store | 2 | Persistent JSON storage |
| tauri-plugin-autostart | 2 | Launch on OS login |
| tauri-plugin-global-shortcut | 2 | System-wide keyboard shortcuts |
| tauri-plugin-updater | 2 | Auto-update via GitHub Releases |
| tauri-plugin-deep-link | 2 | `agentplay://` protocol handler |
| tray-icon (built-in) | — | System tray icon & menu |

## Data Entities

**AppConfig** (persistent JSON via plugin-store)
- notifications_enabled: boolean (default: true)
- auto_start: boolean (default: false)
- minimize_to_tray: boolean (default: true)
- global_shortcut: string (default: "CmdOrCtrl+Shift+A")
- web_app_url: string (empty, env-based)
- check_updates: boolean (default: true)
- last_update_check: ISO timestamp | null

**WindowState** (persistent JSON)
- x, y: window position (null = OS default)
- width, height: 1200×800 default
- maximized: boolean

## Dependencies

**Rust (Cargo.toml)**
```
tauri 2.0
tauri-plugin-* (7 plugins)
serde, serde_json (JSON serialization)
tokio (async runtime)
url (URL parsing)
```

**Node (package.json)**
- Minimal: vite, @tauri-apps/api, @tauri-apps/cli
- No framework (HTML/JS only in frontend)

## Key Patterns

**Remote URL Loading**
- Env var (AGENT_PLAYGROUND_URL) → build-time injection → tauri.conf.json → runtime fallback (localhost:3000)
- Scripts patch capabilities at build time for production URL

**Bridge Injection**
- Tauri initializes_script injects bridge.js into webview on page load
- Polls for __TAURI__ availability (handles timing race)
- Forwards custom events from web app to Rust IPC

**State Persistence**
- plugin-store (JSON) for config and window geometry
- Debounced save on window move/resize (500ms)
- Auto-restore on startup

**Close = Hide**
- Window close minimizes to tray instead of quitting
- Only way to quit: tray menu or Cmd+Q
- Global shortcut toggles visibility

**Update Flow**
- Background check on startup + every 6 hours
- GitHub Releases as source
- User chooses Install Now or Later
- Ed25519 signature verification

## Security & Compliance

- HTTPS-only remote content
- Tauri capabilities for permission control
- No local credential storage (web app cookies)
- Ed25519 signed updates
- Branch protection on main
- CODEOWNERS (phucsystem) for review

## Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Cold start | <3s | Excludes web app load time |
| Notification latency | <500ms | After message received |
| Memory (idle) | <150MB | Rust runtime + webview |
| Binary size | <15MB | Typical Tauri size |

## Development Workflow

1. **Feature development** → branch off main
2. **Code review** → PR to main, CODEOWNERS approval
3. **Merge to main** → GitHub Actions triggers build
4. **Build completes** → Creates draft release with macOS DMG + Windows MSI
5. **Manual release** → GitHub Releases auto-publish (converted from draft)

## Testing Notes

- No unit tests in current setup
- Manual testing on macOS (Intel/Apple Silicon) and Windows recommended
- Integration tests for bridge injection (timing-sensitive)
- Update verification requires test release signing keys

## Known Limitations

- Tauri 2.10: No dock badge support (macOS)
- Single-window only (design constraint)
- Remote URL must be accessible at build time for CSP configuration
- Deep links not fully tested
- Loading screen is HTML/CSS only (no web app integration)

## Dependencies Between Modules

```
main.rs
  └─> lib.rs
        ├─> bridge.js (injected)
        ├─> commands.rs (IPC handlers)
        ├─> tray.rs (menu + events)
        └─> updater.rs (background checks)

commands.rs
  └─> plugin-store (AppConfig, WindowState)

tray.rs
  └─> commands.rs (config toggles)

updater.rs
  └─> tauri-plugin-updater (GitHub API)

bridge.js
  └─> commands.rs (JS → Rust IPC)
```

## Documentation Files

- **SRD.md**: Feature list (FR-xx), screens (S-xx), entities (E-xx), NFRs, decisions
- **API_SPEC.md**: IPC command signatures, JS bridge events, capability details
- **DB_DESIGN.md**: AppConfig & WindowState schemas with field documentation
- **UI_SPEC.md**: Screen layouts, tray menu, update dialog, loading screen design
- **project-overview-pdr.md**: High-level project goals, PDR, target audience
- **code-standards.md**: Rust conventions, error handling, naming, testing
- **system-architecture.md**: Architecture diagrams, data flow, module interactions
- **project-roadmap.md**: Phases 1-3, progress, success criteria, timeline
- **deployment-guide.md**: Build setup, local development, CI/CD pipeline, signing keys
