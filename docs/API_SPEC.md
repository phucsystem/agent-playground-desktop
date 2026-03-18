# Interface Specification (API)

## Note on Architecture

This desktop app has **no HTTP API endpoints**. Instead, it uses **Tauri IPC commands** — Rust functions invocable from the webview JavaScript via `window.__TAURI__.core.invoke()`. These commands are the "API" between the JS bridge (frontend) and the Rust backend.

Additionally, the app uses **Tauri plugins** that provide their own APIs (notifications, autostart, global-shortcut, updater, store).

## 1. Command Matrix

| Command | Direction | Feature (FR-xx) | Screen (S-xx) | Phase |
|---------|-----------|-----------------|----------------|-------|
| `notify_new_message` | JS → Rust | FR-04, FR-05 | S-03 | P1 |
| `update_badge_count` | JS → Rust | FR-10, FR-05 | S-02 | P2 |
| `navigate_to_conversation` | Rust → JS | FR-04 | S-01, S-03 | P1 |
| `get_app_config` | JS → Rust | FR-06, FR-08, FR-09 | S-02 | P1 |
| `set_app_config` | JS → Rust | FR-06, FR-08, FR-09 | S-02 | P1 |
| `get_window_state` | Internal (Rust) | FR-07 | S-01 | P2 |
| `save_window_state` | Internal (Rust) | FR-07 | S-01 | P2 |
| `toggle_window` | Internal (Rust) | FR-02, FR-09 | S-01, S-02 | P1 |
| `check_for_updates` | Internal (Rust) | FR-12 | S-04 | P3 |
| `handle_deep_link` | OS → Rust → JS | FR-13 | S-01 | P3 |
| `report_user_active` | JS → Rust | FR-04 | S-01 | P1 |

## 2. Command Details

### `notify_new_message`

**Direction:** JS Bridge → Rust
**Feature:** FR-04 (Native OS notifications), FR-05 (JS bridge)
**Screen:** S-03 (OS Notification)

**Description:** Web app JS bridge calls this when a new message is received via Supabase Realtime. Rust backend checks if window is focused — if not, sends native OS notification.

**Request (JS → Rust):**
```typescript
interface NotifyNewMessagePayload {
  sender_name: string;        // Display name of message sender
  sender_avatar_url?: string; // Optional avatar URL (future use)
  message_text: string;       // Message body (will be truncated to 100 chars)
  conversation_id: string;    // UUID of the conversation
  conversation_name?: string; // Group name (null for DMs)
  is_group: boolean;          // true if group conversation
}
```

**Rust behavior:**
1. Check `app_config.notifications_enabled` — if false, return early
2. Check if main window is focused — if focused, skip notification
3. Truncate `message_text` to 100 characters + "..."
4. Build notification:
   - Title: `sender_name`
   - Subtitle: `conversation_name` (groups only)
   - Body: truncated `message_text`
5. Send via `tauri-plugin-notification`
6. Increment internal unread counter
7. Update tray tooltip with unread count

**Response:** None (fire-and-forget from JS side)

**Error handling:**
- Notification permission denied → log warning, skip silently
- Invalid payload → log error, skip

---

### `update_badge_count`

**Direction:** JS Bridge → Rust
**Feature:** FR-10 (Unread badge count)
**Screen:** S-02 (Tray Menu)

**Description:** JS bridge reports the current unread message count. Rust updates tray tooltip with unread count.

**Request:**
```typescript
interface UpdateBadgeCountPayload {
  count: number; // Unread message count (0 = clear badge)
}
```

**Rust behavior:**
1. Update tray icon tooltip: `"Agent Playground — {count} unread"` (or just `"Agent Playground"` if 0)
2. If count > 0 and tray supports it: show red dot indicator on tray icon
3. Note: macOS dock badge API not available in Tauri 2.10 (deferred to future version)

**Response:** None

---

### `navigate_to_conversation`

**Direction:** Rust → JS (via webview `evaluate_script`)
**Feature:** FR-04 (Click notification → open conversation)
**Screen:** S-01 (Main Window)

**Description:** When user clicks a notification, Rust shows the window and navigates the webview to the specific conversation.

**Rust behavior:**
1. Show and focus main window
2. Execute JS in webview: `window.location.href = '/chat/{conversationId}'`

**Parameters:**
```rust
struct NavigatePayload {
    conversation_id: String, // UUID
}
```

**JS executed:**
```javascript
window.location.href = `/chat/${conversationId}`;
```

---

### `get_app_config`

**Direction:** JS Bridge → Rust (or internal Rust use)
**Feature:** FR-06, FR-08, FR-09
**Screen:** S-02

**Description:** Read current app configuration from plugin-store.

**Request:** None (no parameters)

**Response:**
```typescript
interface AppConfig {
  notifications_enabled: boolean;
  auto_start: boolean;
  minimize_to_tray: boolean;
  global_shortcut: string;
  web_app_url: string;
  check_updates: boolean;
  last_update_check: string | null;
}
```

---

### `set_app_config`

**Direction:** JS Bridge → Rust (or tray menu → Rust)
**Feature:** FR-06, FR-08, FR-09

**Description:** Update one or more app config fields. Partial update — only provided fields are changed.

**Request:**
```typescript
interface SetAppConfigPayload {
  notifications_enabled?: boolean;
  auto_start?: boolean;
  minimize_to_tray?: boolean;
  global_shortcut?: string;
  web_app_url?: string;
  check_updates?: boolean;
}
```

**Rust behavior:**
1. Read current config from store
2. Merge provided fields
3. Apply side effects:
   - `auto_start` changed → enable/disable via `tauri-plugin-autostart`
   - `global_shortcut` changed → unregister old, register new via `tauri-plugin-global-shortcut`
   - `notifications_enabled` changed → update tray menu checkbox state
4. Persist to store

**Response:** Updated `AppConfig` (full object)

**Errors:**
- Invalid shortcut string → return error, keep old value

---

### `get_window_state` / `save_window_state`

**Direction:** Internal Rust (not exposed to JS)
**Feature:** FR-07

**Description:** Read/write window geometry. Called automatically by Rust on window events.

**Triggers for `save_window_state`:**
- Window moved (debounced 500ms)
- Window resized (debounced 500ms)
- Window close event (before hide)

**Triggers for `get_window_state`:**
- App startup (to restore position/size)

**Data:** See E-02 WindowState in DB_DESIGN.md

---

### `toggle_window`

**Direction:** Internal Rust
**Feature:** FR-02 (Tray icon), FR-09 (Global shortcut)
**Screen:** S-01, S-02

**Description:** Toggle main window visibility. Called from tray menu "Show/Hide" and global shortcut handler.

**Rust behavior:**
1. If window is visible and focused → hide window
2. If window is hidden → show window, focus it, reset badge count
3. Update tray menu item text ("Show Window" ↔ "Hide Window")

---

### `check_for_updates`

**Direction:** Internal Rust (triggered on timer)
**Feature:** FR-12
**Screen:** S-04

**Description:** Check GitHub Releases for newer version using Tauri updater plugin.

**Rust behavior:**
1. Check `app_config.check_updates` — if false, return
2. Check `app_config.last_update_check` — if < 6 hours ago, skip
3. Call Tauri updater `check()` against GitHub Releases endpoint
4. If update available:
   - Show native dialog (S-04) with current vs new version
   - "Install Now" → download + install + restart
   - "Later" → dismiss
5. Update `last_update_check` timestamp in store

**Update endpoint (tauri.conf.json):**
```
https://github.com/{owner}/{repo}/releases/latest/download/latest.json
```

---

### `handle_deep_link`

**Direction:** OS → Rust → JS
**Feature:** FR-13
**Screen:** S-01

**Description:** Handle `agentplay://` protocol URLs from external sources.

**URL format:** `agentplay://conversation/{conversationId}`

**Rust behavior:**
1. Parse URL, extract conversation ID
2. Show and focus main window
3. Navigate webview to `/chat/{conversationId}`

**Registration:** Configured in `capabilities/remote-access.json` under `remote.urls` field for platform-specific protocol handlers.

---

### `report_user_active`

**Direction:** JS Bridge → Rust
**Feature:** FR-04 (Suppress notifications when active)
**Screen:** S-01

**Description:** JS bridge reports that the user is actively viewing a conversation. Used to suppress notifications for the currently-viewed conversation.

**Request:**
```typescript
interface ReportUserActivePayload {
  conversation_id: string | null; // Currently viewed conversation, null if none
}
```

**Rust behavior:**
1. Store `active_conversation_id` in memory (not persisted)
2. When `notify_new_message` is called, skip notification if `conversation_id` matches active

---

## 3. Plugin APIs (External)

These are provided by Tauri plugins — not custom commands. Listed for completeness.

| Plugin | API | Used By |
|--------|-----|---------|
| `tauri-plugin-notification` | `sendNotification()`, `requestPermission()`, `isPermissionGranted()` | FR-04, FR-06 |
| `tauri-plugin-autostart` | `enable()`, `disable()`, `isEnabled()` | FR-08 |
| `tauri-plugin-global-shortcut` | `register()`, `unregister()` | FR-09 |
| `tauri-plugin-updater` | `check()`, `downloadAndInstall()` | FR-12 |
| `tauri-plugin-store` | `get()`, `set()`, `save()` | E-01, E-02 |
| `tauri-plugin-deep-link` | `onOpenUrl()` | FR-13 |

## 4. JS Bridge Events (Webview → Rust)

The initialization script listens for these DOM/custom events and invokes Tauri commands:

| JS Event | Tauri Command | Trigger |
|----------|---------------|---------|
| `CustomEvent('tauri:new-message')` | `notify_new_message` | Supabase Realtime INSERT on messages |
| `CustomEvent('tauri:unread-count')` | `update_badge_count` | Unread count recalculated |
| `CustomEvent('tauri:active-conversation')` | `report_user_active` | User navigates to/from conversation |
| `visibilitychange` (document) | `report_user_active(null)` | Tab becomes hidden |

## 5. Tauri Configuration (tauri.conf.json)

Key configuration sections (see actual `src-tauri/tauri.conf.json` for current values):

```json
{
  "productName": "Agent Playground",
  "identifier": "com.agent-playground.desktop",
  "version": "0.2.0",
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "label": "main",
        "title": "Agent Playground",
        "width": 1200,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600,
        "resizable": true,
        "maximizable": true,
        "visible": true,
        "center": true
      }
    ],
    "security": {
      "csp": null
    }
  }
}
```

**Note:** Version is read from `src-tauri/Cargo.toml`, not here. Remote URL and CSP configuration are handled via capabilities files (see Section 6), not in tauri.conf.json.

## 6. Capabilities

### default.json (src-tauri/capabilities/default.json)

```json
{
  "identifier": "default",
  "description": "Default permissions for Agent Playground Desktop",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:window:allow-set-focus",
    "core:window:allow-close",
    "core:window:allow-is-visible",
    "core:window:allow-is-focused",
    "notification:default",
    "notification:allow-show",
    "notification:allow-request-permission",
    "notification:allow-is-permission-granted",
    "store:default",
    "global-shortcut:default",
    "global-shortcut:allow-register",
    "global-shortcut:allow-unregister",
    "autostart:default",
    "autostart:allow-enable",
    "autostart:allow-disable",
    "autostart:allow-is-enabled",
    "updater:default",
    "deep-link:default"
  ]
}
```

**Note:** Correct permission name for notifications is `notification:allow-show` (not `notification:allow-send-notification`).

### remote-access.json (src-tauri/capabilities/remote-access.json)

Controls remote web app access to Tauri APIs via the `remote.urls` field:

```json
{
  "identifier": "remote-api-access",
  "description": "Allow remote web app to invoke Tauri APIs",
  "windows": ["main"],
  "remote": {
    "urls": ["http://localhost:3000/*", "https://*"]
  },
  "permissions": [
    "core:default",
    "notification:default",
    "notification:allow-show",
    "notification:allow-request-permission",
    "notification:allow-is-permission-granted",
    "store:default"
  ]
}
```

**Migration note:** Tauri v2 uses capability-based permissions with `remote.urls` instead of the deprecated `dangerousRemoteDomainIpcAccess` setting.

## 7. Traceability Matrix

| FR | Commands / Plugins | Screens |
|----|--------------------|---------|
| FR-01 | tauri.conf.json window config | S-01 |
| FR-02 | `toggle_window`, TrayIcon API | S-01, S-02 |
| FR-03 | Window close event handler | S-01 |
| FR-04 | `notify_new_message`, `navigate_to_conversation`, `report_user_active` | S-01, S-03 |
| FR-05 | initialization_script, all JS→Rust commands | S-01 |
| FR-06 | `get_app_config`, `set_app_config`, plugin-notification | S-02 |
| FR-07 | `get_window_state`, `save_window_state` | S-01 |
| FR-08 | `set_app_config`, plugin-autostart | S-02 |
| FR-09 | `set_app_config`, plugin-global-shortcut | S-02 |
| FR-10 | `update_badge_count` | S-02 |
| FR-11 | Notification grouping logic in `notify_new_message` | S-03 |
| FR-12 | `check_for_updates`, plugin-updater | S-04 |
| FR-13 | `handle_deep_link`, plugin-deep-link | S-01 |
| FR-14 | GitHub Actions (external) | — |
| FR-15 | Webview load event handler | S-05 |
