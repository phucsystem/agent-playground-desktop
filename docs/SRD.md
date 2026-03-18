# System Requirement Definition (SRD)

## 1. System Overview

**Project:** Agent Playground Desktop
**Purpose:** Native desktop shell for Agent Playground — a chat-based human-agent collaboration platform. Wraps the existing web app in a Tauri v2 window with system tray, native notifications, and desktop integration features. The web app handles all chat functionality; the desktop shell adds persistent presence and OS-level conveniences.

**Tech Stack:**
- Desktop Shell: Tauri v2 (Rust backend + system webview)
- Web Content: Remote-loaded Next.js app (existing Agent Playground)
- State Storage: Tauri plugin-store (local JSON)
- Notifications: Tauri notification plugin (native OS)
- Distribution: GitHub Releases + Tauri updater plugin
- Build: GitHub Actions (cross-platform: macOS, Windows, Linux)

**Key Constraints:**
- No chat logic in desktop app — all chat handled by web app
- Remote URL loading (not bundled Next.js)
- System webview only (WebKit on macOS, WebView2 on Windows, WebKitGTK on Linux)
- JS bridge injected via `initialization_script` for IPC between web app and Tauri shell
- Single-window app (no multi-window)

**Relationship to Web App:**
- Web app repo: `/Users/phuc/Code/04-llms/agent-playground`
- Desktop app loads web app via HTTPS URL
- Shared Supabase backend (no duplication)
- Desktop app adds: tray icon, native notifications, window persistence, auto-start, shortcuts

## 2. Actors (User Roles)

| ID | Role | Description | Desktop-Specific Permissions |
|----|------|-------------|------------------------------|
| R-01 | **Desktop User** | Any Agent Playground user (admin/user) running the desktop app | Manage app preferences (auto-start, shortcuts, notifications). Interact via tray menu. Receive native notifications. |

Note: Admin/User/Agent role distinctions are handled by the web app. Desktop shell treats all users equally.

## 3. Functional Requirements (FR-xx)

### Phase 1 — Core Shell (P1)

| ID | Feature | Priority | Description |
|----|---------|----------|-------------|
| FR-01 | Native window with webview | P1 | Launch Tauri window loading the Agent Playground web app URL. Window has standard OS chrome (title bar, minimize/maximize/close). Default size: 1200x800, min size: 800x600. |
| FR-02 | System tray icon | P1 | Persistent tray icon with right-click context menu. Menu items: Show/Hide Window, Separator, Quit. Tray icon uses app logo. Double-click tray icon toggles window visibility. |
| FR-03 | Minimize to tray | P1 | Closing the window minimizes to system tray instead of quitting. App continues running in background. Quit only via tray menu or keyboard shortcut (Cmd+Q / Alt+F4). |
| FR-04 | Native OS notifications | P1 | When web app detects a new message (via JS bridge), send native OS notification with: sender name as title, message preview as body (truncated to 100 chars). Clicking notification restores window and navigates to conversation. |
| FR-05 | JS↔Rust bridge | P1 | Inject initialization script into webview that: (a) detects `window.__TAURI__` availability, (b) listens for Supabase Realtime message events, (c) invokes Tauri commands for notifications and badge updates. Bridge must handle page reloads gracefully. |
| FR-06 | Notification permission | P1 | On first launch, request OS notification permission. Store preference in local config. User can toggle notifications on/off via tray menu. |

### Phase 2 — Polish (P2)

| ID | Feature | Priority | Description |
|----|---------|----------|-------------|
| FR-07 | Window state persistence | P2 | Save window position (x, y), size (width, height), and maximized state on close. Restore on next launch. Store in local JSON via plugin-store. |
| FR-08 | Auto-start on login | P2 | Option to launch app automatically on OS login. Starts minimized to tray. Configurable via tray menu toggle. Uses Tauri autostart plugin. |
| FR-09 | Global keyboard shortcut | P2 | Register system-wide hotkey (default: Cmd+Shift+A on macOS, Ctrl+Shift+A on Windows/Linux) to toggle window visibility. Configurable in future. Uses Tauri global-shortcut plugin. |
| FR-10 | Unread badge count | P2 | Display unread message count on: (a) tray icon tooltip. Web app JS bridge reports unread count → Rust updates tray tooltip. Reset count when window is focused. **Note:** macOS dock badge not available in Tauri 2.10 (deferred to future version). |
| FR-11 | Notification grouping | P2 | Group multiple notifications from same conversation. Replace previous notification from same conversation with updated one (prevent notification spam). |

### Phase 3 — Distribution (P3)

| ID | Feature | Priority | Description |
|----|---------|----------|-------------|
| FR-12 | Auto-updater | P3 | Check for updates on app launch (and periodically every 6 hours). Show update dialog with version info and changelog excerpt. User chooses Install Now or Later. Uses Tauri updater plugin with GitHub Releases as update server. |
| FR-13 | Deep link protocol | P3 | Register `agentplay://` protocol handler. Format: `agentplay://conversation/{id}`. When clicked externally, opens/focuses app and navigates webview to the conversation. |
| FR-14 | CI/CD build pipeline | P3 | GitHub Actions workflow that builds and signs desktop app for macOS (universal binary), Windows (x64), and Linux (AppImage). Triggered on PR merge to main. Uploads artifacts to GitHub Releases. |
| FR-15 | Splash/loading screen | P3 | Show minimal loading indicator while remote web app loads. Display app icon + "Connecting..." text. Hide once webview `DOMContentLoaded` fires. Prevents blank window on slow connections. |

## 4. Screen List (S-xx)

| ID | Screen Name | Description | Phase |
|----|-------------|-------------|-------|
| S-01 | Main Window | Tauri webview window loading Agent Playground web app. Standard OS window chrome. | P1 |
| S-02 | Tray Menu | System tray context menu with app controls | P1 |
| S-03 | OS Notification | Native notification banner for new messages | P1 |
| S-04 | Update Dialog | Auto-update prompt with version info and Install/Later actions | P3 |
| S-05 | Loading Screen | Splash screen shown while web app loads | P3 |

Note: All chat screens (login, sidebar, conversations, admin) are part of the web app — not managed by the desktop shell.

## 5. Entity List (E-xx)

| ID | Entity | Description | Storage |
|----|--------|-------------|---------|
| E-01 | AppConfig | User preferences for desktop app behavior | Local JSON (plugin-store) |
| E-02 | WindowState | Window geometry and display state | Local JSON (plugin-store) |

### E-01: AppConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| notifications_enabled | boolean | true | Whether to show OS notifications |
| auto_start | boolean | false | Launch on OS login |
| minimize_to_tray | boolean | true | Close button minimizes instead of quits |
| global_shortcut | string | "CmdOrCtrl+Shift+A" | Global hotkey to toggle window |
| web_app_url | string | (env) | URL of the Agent Playground web app |
| check_updates | boolean | true | Auto-check for updates |
| last_update_check | string | null | ISO timestamp of last update check |

### E-02: WindowState

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| x | number | null | Window X position (null = OS default) |
| y | number | null | Window Y position |
| width | number | 1200 | Window width in pixels |
| height | number | 800 | Window height in pixels |
| maximized | boolean | false | Whether window is maximized |

## 6. Non-Functional Requirements

### Performance
- Cold start to visible window: < 3s (excluding web app load time)
- Notification delivery: < 500ms after message received by webview
- Memory usage: < 150MB idle (system webview + Rust runtime)
- Binary size: < 15MB (Tauri typical: 3-10MB)

### Security
- Remote URL loaded over HTTPS only
- Tauri CSP configured to allow only the web app domain
- No local storage of credentials (handled by web app cookies)
- Updater uses Ed25519 signature verification
- JS bridge limited to specific Tauri commands (no arbitrary IPC)

### Compatibility
- macOS 12+ (Monterey) — WebKit
- Windows 10+ — WebView2 (auto-installs if missing)
- Linux: Ubuntu 22.04+, Fedora 38+ — WebKitGTK

### Reliability
- Graceful handling of network disconnection (web app handles reconnect; desktop shows status)
- Tray icon persists even if webview crashes
- Window state saves on every move/resize (debounced 500ms)

## 7. Key Decisions (D-xx)

| ID | Decision | Context | Chosen | Rationale |
|----|----------|---------|--------|-----------|
| D-01 | Desktop framework | Electron vs Tauri | Tauri v2 | 3-10MB binary vs 150MB+. Lower memory. System webview. Rust backend for reliability. |
| D-02 | Content loading | Bundle Next.js vs remote URL | Remote URL | Web app deploys independently. No build coupling. Instant web updates reflected in desktop. |
| D-03 | IPC mechanism | Custom WebSocket vs Tauri invoke | Tauri initialization_script + invoke | Native IPC, no extra server. Secure command-based API. |
| D-04 | Local storage | SQLite vs JSON file | Tauri plugin-store (JSON) | Only storing ~10 config values. No query needs. Simple read/write. |
| D-05 | Update distribution | App store vs self-hosted | GitHub Releases + Tauri updater | Free hosting. Signature verification built-in. CI/CD friendly. |
| D-06 | Notification bridge | Polling vs event-driven | Event-driven via JS bridge | Zero latency. Web app already has Realtime subscription — just forward events. |

## 8. Out of Scope

- Rebuilding chat UI natively
- Local message caching / offline mode
- Mobile app
- Self-hosted Supabase bundling
- End-to-end encryption
- Multi-window / pop-out conversations
- Custom window chrome / frameless window (use OS default)
- In-app settings page (use tray menu for MVP)
