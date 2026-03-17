# Phase 02 — Polish

## Context

- [Phase 01](./phase-01-core-shell.md) — Must be complete before starting
- [SRD.md](../../docs/SRD.md) — FR-07 through FR-11
- [API_SPEC.md](../../docs/API_SPEC.md) — Window state, badge, config commands
- [DB_DESIGN.md](../../docs/DB_DESIGN.md) — E-02 WindowState schema

## Overview

- **Priority:** P2
- **Status:** Complete ✅
- **Effort:** 4h
- **Description:** Add window state persistence, auto-start on login, global keyboard shortcut, unread badge count, and notification grouping.

## Requirements

**Functional:** FR-07 (window state), FR-08 (auto-start), FR-09 (global shortcut), FR-10 (unread badge), FR-11 (notification grouping)

## Related Code Files

### Files to Modify

| File | Changes |
|------|---------|
| `src-tauri/Cargo.toml` | Add: `tauri-plugin-autostart`, `tauri-plugin-global-shortcut` |
| `src-tauri/capabilities/default.json` | Add autostart + global-shortcut permissions |
| `src-tauri/src/lib.rs` | Register new plugins + commands |
| `src-tauri/src/commands.rs` | Add `update_badge_count` command |
| `src-tauri/src/tray.rs` | Add "Start on Login" menu item, badge tooltip |
| `src/bridge.js` | Add unread count event listener |

### Files to Create

| File | Purpose |
|------|---------|
| `src-tauri/src/window.rs` | Window state save/restore logic |

## Implementation Steps

### Step 1: Add Plugin Dependencies (15m)

1. Install JS packages:
   ```bash
   npm install @tauri-apps/plugin-autostart @tauri-apps/plugin-global-shortcut
   ```
2. Add to `Cargo.toml`:
   ```toml
   tauri-plugin-autostart = "2"
   tauri-plugin-global-shortcut = "2"
   ```
3. Update `capabilities/default.json` — add permissions:
   ```json
   "global-shortcut:default",
   "global-shortcut:allow-register",
   "global-shortcut:allow-unregister",
   "autostart:default",
   "autostart:allow-enable",
   "autostart:allow-disable",
   "autostart:allow-is-enabled"
   ```

### Step 2: Window State Persistence — FR-07 (1h)

**`src-tauri/src/window.rs`**

1. Define `WindowState` struct matching E-02:
   ```rust
   #[derive(Serialize, Deserialize, Clone)]
   pub struct WindowState {
       pub x: Option<f64>,
       pub y: Option<f64>,
       pub width: f64,
       pub height: f64,
       pub maximized: bool,
   }
   ```

2. **Save on window events** (in `lib.rs`):
   - Listen for `WindowEvent::Moved` → save position (debounced 500ms)
   - Listen for `WindowEvent::Resized` → save size (debounced 500ms)
   - Listen for `WindowEvent::CloseRequested` → save final state before hide
   - Use a debounce timer (tokio::time::sleep) to avoid excessive writes

3. **Restore on startup** (in `lib.rs`, before window creation):
   - Read `window_state` from plugin-store
   - If exists → apply position, size, maximized to window builder
   - If position is off-screen (monitor disconnected) → reset to OS default
   - Validation: width ≥ 800, height ≥ 600

### Step 3: Auto-Start on Login — FR-08 (30m)

1. Register `tauri_plugin_autostart` in `lib.rs`
2. In `tray.rs`, add "Start on Login" checkbox menu item:
   - On toggle → call `autostart.enable()` or `autostart.disable()`
   - Sync state with `app_config.auto_start` in store
3. On app startup:
   - Read `app_config.auto_start`
   - Sync autostart plugin state with stored preference
   - If `auto_start: true` and app started via login → start hidden (minimize to tray)

### Step 4: Global Keyboard Shortcut — FR-09 (30m)

1. Register `tauri_plugin_global_shortcut` in `lib.rs`
2. On app startup:
   - Read `app_config.global_shortcut` (default: "CmdOrCtrl+Shift+A")
   - Register shortcut → on trigger: `toggle_window()`
3. Handle registration failure gracefully:
   - Shortcut already taken by another app → log warning, skip
   - Invalid shortcut string → use default

### Step 5: Unread Badge Count — FR-10 (1h)

1. **JS bridge update** (`src/bridge.js`):
   - Listen for `tauri:unread-count` CustomEvent
   - Invoke `update_badge_count` with `{ count: number }`

2. **Rust command** (`commands.rs`):
   ```rust
   #[tauri::command]
   fn update_badge_count(app: AppHandle, count: u32) {
       // Update tray tooltip
       let tooltip = if count > 0 {
           format!("Agent Playground — {} unread", count)
       } else {
           "Agent Playground".to_string()
       };
       // Set tooltip on tray icon
       // macOS: set dock badge
       #[cfg(target_os = "macos")]
       app.set_badge_label(if count > 0 { Some(count.to_string()) } else { None });
   }
   ```

3. **Reset on focus**:
   - When window receives focus → invoke `update_badge_count(0)` from Rust
   - Clear dock badge + reset tray tooltip

### Step 6: Notification Grouping — FR-11 (45m)

1. Track last notification per conversation in Rust state:
   ```rust
   HashMap<String, NotificationId>  // conversation_id → last notification
   ```
2. When `notify_new_message` called:
   - If existing notification for same conversation → replace it
   - Update body: "N new messages" if multiple
   - This prevents notification spam from rapid messages

## Todo List

- [x] Add autostart + global-shortcut plugin deps
- [x] Update capabilities with new permissions
- [x] Implement `window.rs` — save/restore window state
- [x] Wire window events to state save (debounced)
- [x] Add "Start on Login" tray menu item + autostart toggle
- [x] Register global shortcut on startup
- [x] Add `update_badge_count` command
- [x] Update bridge.js with unread count listener
- [x] Implement notification grouping (replace per conversation)
- [x] Test window state persists across restart
- [x] Test auto-start creates login item
- [x] Test global shortcut toggles window

## Success Criteria

- [x] Window remembers position + size after restart
- [x] Off-screen window state handled gracefully (resets to center)
- [x] "Start on Login" toggle creates/removes OS login item
- [x] Global shortcut (Cmd+Shift+A) toggles window from any app
- [x] Tray tooltip shows unread count
- [x] macOS dock badge shows unread number
- [x] Badge clears on window focus
- [x] Rapid messages from same conversation show single notification (not spam)

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Global shortcut conflicts with other apps | Shortcut won't register | Catch error, log warning, continue without shortcut |
| Autostart plugin behaves differently across OS | Login item not created on some Linux distros | Test on target platforms; document unsupported cases |
| Debounced window save loses state on crash | Window position lost | Save immediately on close event (not debounced) |

## Next Steps

→ Phase 03: Auto-updater, deep links, CI/CD pipeline
