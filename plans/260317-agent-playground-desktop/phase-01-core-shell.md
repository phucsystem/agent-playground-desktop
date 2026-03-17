# Phase 01 — Core Shell

## Context

- [SRD.md](../../docs/SRD.md) — FR-01 through FR-06
- [API_SPEC.md](../../docs/API_SPEC.md) — Command specs
- [UI_SPEC.md](../../docs/UI_SPEC.md) — S-01, S-02, S-03 specs
- [DB_DESIGN.md](../../docs/DB_DESIGN.md) — E-01, E-02 schemas

## Overview

- **Priority:** P1
- **Status:** Complete ✅
- **Effort:** 8h
- **Description:** Initialize Tauri v2 project, configure webview to load remote web app, implement system tray with context menu, add native notifications via JS bridge, and set up local config store.

## Key Insights

- Tauri v2 uses `dangerousRemoteDomainIpcAccess` to allow IPC from remote URLs
- `initialization_script` runs on every page load — must be idempotent
- `withGlobalTauri: true` exposes `window.__TAURI__` to webview
- System tray API changed in v2: `TrayIcon::new()` + `Menu::new()`
- Notifications require permission request on first use

## Requirements

**Functional:** FR-01 (webview window), FR-02 (tray icon), FR-03 (minimize to tray), FR-04 (notifications), FR-05 (JS bridge), FR-06 (notification permission)

**Non-Functional:**
- Cold start < 3s
- Notification < 500ms after message received
- Memory < 150MB idle
- Binary < 15MB

## Architecture

```
[App Launch]
     │
     ▼
[lib.rs: setup()]
     ├── register plugins (store, notification)
     ├── register IPC commands
     ├── create main window (webview → remote URL)
     ├── inject initialization_script (bridge.js)
     └── create system tray (tray.rs)
            ├── icon + tooltip
            └── context menu (Show/Hide, Notifications toggle, Quit)

[bridge.js: injected into webview]
     ├── detect window.__TAURI__
     ├── listen for 'tauri:new-message' CustomEvent
     ├── invoke('notify_new_message', payload)
     └── invoke('report_user_active', payload)

[commands.rs: Rust handlers]
     ├── notify_new_message → check focus → send OS notification
     ├── report_user_active → store active conversation ID
     └── get/set_app_config → read/write plugin-store
```

## Related Code Files

### Files to Create

| File | Purpose |
|------|---------|
| `package.json` | Node deps: `@tauri-apps/cli`, `@tauri-apps/api` |
| `src-tauri/Cargo.toml` | Rust deps: tauri, tauri-plugin-notification, tauri-plugin-store |
| `src-tauri/tauri.conf.json` | App config: window, security, remote domain IPC |
| `src-tauri/capabilities/default.json` | Permission ACL for plugins |
| `src-tauri/src/main.rs` | Entry point (`fn main()`) |
| `src-tauri/src/lib.rs` | App setup: plugins, commands, tray, window |
| `src-tauri/src/commands.rs` | IPC command handlers: notify, badge, config |
| `src-tauri/src/tray.rs` | System tray icon + context menu |
| `src-tauri/src/bridge.rs` | JS bridge script as Rust string constant |
| `src/bridge.js` | JS bridge source (for readability; compiled into bridge.rs) |
| `src-tauri/icons/` | App icons for all platforms |

## Implementation Steps

### Step 1: Initialize Tauri v2 Project (1h)

1. Run `npm init -y` in project root
2. Install Tauri CLI: `npm install -D @tauri-apps/cli@^2`
3. Install Tauri JS API: `npm install @tauri-apps/api@^2`
4. Install Tauri plugins (JS side):
   ```bash
   npm install @tauri-apps/plugin-notification
   npm install @tauri-apps/plugin-store
   ```
5. Run `npx tauri init` to scaffold `src-tauri/`
6. Edit `src-tauri/Cargo.toml` — add plugin dependencies:
   ```toml
   [dependencies]
   tauri = { version = "2", features = ["tray-icon"] }
   tauri-plugin-notification = "2"
   tauri-plugin-store = "2"
   serde = { version = "1", features = ["derive"] }
   serde_json = "1"
   ```
7. Generate app icons: place SVG in project root, run `npx tauri icon ./icon.svg`

### Step 2: Configure tauri.conf.json (30m)

Configure based on API_SPEC.md § Tauri Configuration:

```json
{
  "productName": "Agent Playground",
  "identifier": "com.agent-playground.desktop",
  "version": "0.1.0",
  "build": {
    "beforeBuildCommand": "",
    "beforeDevCommand": "",
    "devUrl": "https://YOUR_WEB_APP_URL",
    "frontendDist": "../src"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "label": "main",
        "title": "Agent Playground",
        "url": "https://YOUR_WEB_APP_URL",
        "width": 1200,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600,
        "resizable": true,
        "visible": true
      }
    ],
    "security": {
      "dangerousRemoteDomainIpcAccess": [
        {
          "domain": "YOUR_WEB_APP_DOMAIN",
          "enableTauriAPI": true,
          "windows": ["main"],
          "plugins": ["notification", "store"]
        }
      ]
    }
  }
}
```

**Note:** Replace `YOUR_WEB_APP_URL` and `YOUR_WEB_APP_DOMAIN` with actual deployment URL.

### Step 3: Configure Capabilities (15m)

Create `src-tauri/capabilities/default.json` per API_SPEC.md § Capabilities:

```json
{
  "identifier": "default",
  "description": "Default permissions",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:window:allow-set-focus",
    "core:window:allow-close",
    "notification:default",
    "notification:allow-send-notification",
    "notification:allow-request-permission",
    "notification:allow-is-permission-granted",
    "store:default"
  ]
}
```

### Step 4: Implement Rust Backend — main.rs + lib.rs (1h)

**`src-tauri/src/main.rs`** — minimal entry:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    agent_playground_desktop_lib::run();
}
```

**`src-tauri/src/lib.rs`** — app setup:
- Register plugins: `tauri_plugin_notification`, `tauri_plugin_store`
- Register IPC commands: `notify_new_message`, `report_user_active`, `get_app_config`, `set_app_config`
- Set up system tray (delegate to `tray.rs`)
- Inject initialization script (from `bridge.rs`)
- Handle window close event → hide instead of quit (FR-03)

### Step 5: Implement System Tray (1h)

**`src-tauri/src/tray.rs`**

Build tray per UI_SPEC.md § S-02:
- Create `TrayIcon` with app icon
- Build context menu:
  - "Agent Playground" (disabled label)
  - Separator
  - "Show Window" / "Hide Window" (toggle based on visibility)
  - Separator
  - "Notifications" (checkbox, default checked)
  - Separator
  - "Quit"
- Handle menu item clicks:
  - Show/Hide → `toggle_window()` logic
  - Notifications → update `app_config.notifications_enabled` in store
  - Quit → `app.exit(0)`
- Handle tray icon double-click → toggle window
- Set tooltip: "Agent Playground"

### Step 6: Implement IPC Commands (1.5h)

**`src-tauri/src/commands.rs`**

Per API_SPEC.md § Command Details:

**`notify_new_message`:**
1. Deserialize `NotifyNewMessagePayload` (sender_name, message_text, conversation_id, conversation_name, is_group)
2. Check `app_config.notifications_enabled` → return if false
3. Check window focus state → return if focused AND active conversation matches
4. Truncate message_text to 100 chars
5. Build notification: title=sender_name, body=truncated text
6. Send via notification plugin

**`report_user_active`:**
1. Deserialize payload: `{ conversation_id: Option<String> }`
2. Store in app state (Mutex-wrapped)
3. Used by `notify_new_message` to suppress notifications for active conversation

**`get_app_config` / `set_app_config`:**
1. Read/write from plugin-store using key `"app_config"`
2. `set_app_config` merges partial updates
3. Return full AppConfig struct

### Step 7: Implement JS Bridge (1.5h)

**`src/bridge.js`** (source) → **`src-tauri/src/bridge.rs`** (embedded as string)

Bridge logic per UI_SPEC.md § JS Bridge Specification:

```javascript
(function() {
  // Guard: only run in Tauri
  if (!window.__TAURI__) return;

  const { invoke } = window.__TAURI__.core;

  // Strategy 1: Listen for custom events from web app (preferred)
  window.addEventListener('tauri:new-message', (event) => {
    const { sender, text, conversationId, conversationName, isGroup } = event.detail;
    invoke('notify_new_message', {
      sender_name: sender,
      message_text: text,
      conversation_id: conversationId,
      conversation_name: conversationName || null,
      is_group: isGroup || false
    });
  });

  window.addEventListener('tauri:active-conversation', (event) => {
    invoke('report_user_active', {
      conversation_id: event.detail.conversationId || null
    });
  });

  // Track visibility
  document.addEventListener('visibilitychange', () => {
    if (document.hidden) {
      invoke('report_user_active', { conversation_id: null });
    }
  });
})();
```

**`src-tauri/src/bridge.rs`:**
```rust
pub const BRIDGE_SCRIPT: &str = include_str!("../../src/bridge.js");
```

Register in `lib.rs` via `WebviewWindowBuilder::initialization_script(BRIDGE_SCRIPT)` or via `tauri.conf.json` `app.windows[0].initializationScript`.

### Step 8: Handle Window Close → Minimize to Tray (30m)

In `lib.rs`, register `on_window_event` handler:
- `WindowEvent::CloseRequested` → prevent default, hide window instead
- Update tray menu "Show Window" text
- This implements FR-03

### Step 9: Notification Permission Flow (30m)

On first app launch:
1. Check `isPermissionGranted()` via notification plugin
2. If not granted → `requestPermission()`
3. Store result in `app_config.notifications_enabled`
4. If denied → set to false, skip future notifications

### Step 10: Verify and Test (30m)

1. `npx tauri dev` — verify window loads web app
2. Test tray icon appears and menu works
3. Test close → minimize to tray
4. Test notification (manually dispatch CustomEvent in devtools)
5. Test tray double-click → toggle window
6. Verify `store.json` is created with defaults

## Todo List

- [x] Initialize Tauri v2 project (`npm init`, `npx tauri init`)
- [x] Configure `tauri.conf.json` with remote URL + security
- [x] Configure capabilities ACL
- [x] Implement `main.rs` + `lib.rs` (app setup, plugins, commands)
- [x] Implement `tray.rs` (system tray + context menu)
- [x] Implement `commands.rs` (notify, report_active, config CRUD)
- [x] Write `bridge.js` + embed in `bridge.rs`
- [x] Handle window close → hide to tray
- [x] Implement notification permission flow
- [x] Compile and test on macOS
- [x] Test: tray menu, notifications, minimize to tray

## Success Criteria

- [x] `npx tauri dev` launches window loading remote web app
- [x] System tray icon visible with working context menu
- [x] Close button hides to tray (app stays running)
- [x] Tray double-click / menu "Show" restores window
- [x] JS bridge detects `window.__TAURI__` and can invoke commands
- [x] Native notification appears when `notify_new_message` invoked
- [x] `store.json` created with default AppConfig values
- [x] Memory < 150MB idle, binary compiles < 15MB

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Remote URL CSP blocks `__TAURI__` injection | Bridge won't work | Configure `dangerousRemoteDomainIpcAccess` correctly |
| WebView renders web app differently than Chrome | Visual bugs | Test early; file CSS workarounds if needed |
| Notification permission denied by OS | No notifications | Graceful fallback; show in-tray indicator instead |
| `initialization_script` not re-injected on SPA navigation | Bridge breaks on route change | Script runs at page level; SPA doesn't reload page — should persist |

## Security Considerations

- `dangerousRemoteDomainIpcAccess` limited to specific domain only
- IPC commands limited to declared capabilities (no wildcard)
- No credentials stored locally — web app handles auth via cookies
- HTTPS-only for remote URL

## Next Steps

→ Phase 02: Window state persistence, auto-start, global shortcuts, unread badge
