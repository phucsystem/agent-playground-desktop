# Basic Design (UI Specification)

## 1. Design System

### Design Inheritance

The desktop app **inherits all visual design** from the Agent Playground web app. No custom CSS, themes, or visual components are built in the desktop shell. The web app's design system (documented in the web app's `docs/UI_SPEC.md`) applies fully.

Desktop-specific UI elements use **native OS styling**:
- Window chrome: OS default title bar, minimize/maximize/close buttons
- Tray menu: OS native context menu
- Notifications: OS native notification banners
- Update dialog: OS native dialog (or Tauri dialog plugin)

### Desktop-Specific Tokens

| Token | macOS | Windows | Linux |
|-------|-------|---------|-------|
| Tray icon size | 22x22 @2x | 16x16 / 32x32 | 24x24 |
| App icon | .icns (512x512) | .ico (256x256) | .png (512x512) |
| Notification icon | App icon (auto) | App icon (auto) | App icon (auto) |
| Badge color | System red | N/A (tooltip only) | N/A (tooltip only) |

### App Icon

Single icon design used across all platforms:
- Speech bubble shape with agent/robot motif (matches web app favicon)
- High contrast for tray visibility on light and dark OS themes
- Tray icon: simplified monochrome version for small sizes
- Source formats: SVG → exported to .icns (macOS), .ico (Windows), .png (Linux)

## 2. Screen Flow

```
[App Launch]
     │
     ├── First launch? ──Yes──→ [Request notification permission]
     │                                    │
     │                                    ▼
     ▼                           [S-01: Main Window]
[S-05: Loading Screen]                    │
     │                                    │
     │ (web app loaded)                   │
     ▼                                    │
[S-01: Main Window] ◄────────────────────┘
     │
     ├── Close button ──→ [Minimize to tray] ──→ [S-02: Tray Icon active]
     │
     ├── New message (window hidden) ──→ [S-03: OS Notification]
     │                                         │
     │                                         ├── Click ──→ [Restore S-01 + navigate]
     │                                         └── Dismiss ──→ (stays in tray)
     │
     ├── Tray: Show/Hide ──→ [Toggle S-01 visibility]
     │
     ├── Tray: Quit ──→ [Exit app]
     │
     ├── Global shortcut ──→ [Toggle S-01 visibility]
     │
     └── Update available ──→ [S-04: Update Dialog]
                                    │
                                    ├── Install ──→ [Download + restart]
                                    └── Later ──→ [Dismiss, check again in 6h]
```

## 3. Screen Specifications

### S-01: Main Window

**Purpose:** Primary application window containing the web app webview.

**Layout:**
- OS-native title bar with app name "Agent Playground"
- Full-window webview below title bar (no custom toolbar/frame)
- No borders, padding, or chrome beyond OS default

**Behavior:**
- Default size: 1200 x 800 px
- Minimum size: 800 x 600 px
- Resizable, maximizable, minimizable
- Position and size persisted across sessions (E-02)
- Close button → minimize to tray (not quit)
- Focus → reset unread badge count to 0

**Webview Configuration:**
- URL: Agent Playground deployment URL (from E-01.web_app_url)
- JavaScript enabled
- localStorage/cookies enabled (web app auth)
- File download enabled (chat attachments)
- Context menu: disabled (use web app's own)
- Zoom: OS default (Cmd+/- on macOS, Ctrl+/- on Windows)
- DevTools: enabled in dev builds only

**Loading States:**
- While loading: show S-05 Loading Screen
- On load complete: hide loading screen, show webview
- On network error: show inline error with "Retry" button

---

### S-02: Tray Menu

**Purpose:** System tray context menu for app control when window is hidden.

**Platform behavior:**
- macOS: Menu bar icon (top-right), left-click shows menu
- Windows: System tray icon (bottom-right), right-click shows menu
- Linux: System tray / app indicator

**Menu Structure:**

```
┌────────────────────────┐
│ Agent Playground       │  ← App name (disabled label)
│ ──────────────────     │
│ ● Show Window          │  ← Toggle: "Show" when hidden, "Hide" when visible
│ ──────────────────     │
│ ☑ Notifications        │  ← Toggle: enable/disable notifications
│ ☑ Start on Login       │  ← Toggle: auto-start on OS login
│ ──────────────────     │
│ ✕ Quit                 │  ← Quit app entirely
└────────────────────────┘
```

**Tray Icon States:**

| State | Icon | Tooltip |
|-------|------|---------|
| Normal (0 unread) | App icon (monochrome) | "Agent Playground" |
| Unread messages | App icon + red dot | "Agent Playground — 3 unread" |

**macOS Dock Badge:**
- Show unread count as dock badge number
- Clear when window receives focus

---

### S-03: OS Notification

**Purpose:** Alert user to new messages when window is hidden or unfocused.

**Trigger conditions (ALL must be true):**
1. New message received via JS bridge
2. Window is hidden OR unfocused
3. Message sender is not the current user
4. Notifications are enabled (E-01.notifications_enabled)

**Content:**

| Field | Source | Example |
|-------|--------|---------|
| Title | Sender display name | "Alice Chen" |
| Subtitle | Conversation name (groups only) | "Team Alpha" |
| Body | Message text (truncated 100 chars) | "Hey, can you check the webhook logs? The agent seems..." |
| Icon | App icon | (auto) |

**Actions:**
- Click notification → restore window + navigate webview to conversation URL
- Dismiss → no action

**Grouping (P2):**
- Group by conversation ID
- Replace previous notification from same conversation
- Show count: "3 new messages in Team Alpha"

---

### S-04: Update Dialog

**Purpose:** Inform user about available update and prompt to install.

**Trigger:** Auto-update check finds newer version on GitHub Releases.

**Layout:** Native OS dialog (Tauri dialog plugin)

```
┌──────────────────────────────────────┐
│  Update Available                     │
│                                       │
│  A new version of Agent Playground    │
│  is available.                        │
│                                       │
│  Current: v1.0.0                      │
│  New: v1.1.0                          │
│                                       │
│  [Later]              [Install Now]   │
└──────────────────────────────────────┘
```

**Actions:**
- Install Now → download update, show progress, restart app
- Later → dismiss, schedule re-check in 6 hours

---

### S-05: Loading Screen

**Purpose:** Show feedback while web app loads over network.

**Layout:** Centered on window, minimal design

```
┌──────────────────────────────────────┐
│                                       │
│                                       │
│            [App Icon]                 │
│                                       │
│          Connecting...                │
│          ───── (progress bar)         │
│                                       │
│                                       │
└──────────────────────────────────────┘
```

**Behavior:**
- Show on app launch before webview content is visible
- Background: white (#FFFFFF) or dark (#18181b) matching OS theme
- App icon: 64x64 centered
- Text: "Connecting..." in system font, neutral-500 color
- Progress: indeterminate bar or spinner
- Auto-hide when webview fires `DOMContentLoaded` or `load` event
- Timeout: if load takes > 15s, show "Connection slow. Retry?" with button

## 4. JS Bridge Specification

### Bridge Architecture

The JS bridge is injected into the webview via Tauri's `initialization_script`. It runs in the web app's JS context and communicates with the Rust backend via `window.__TAURI__.core.invoke()`.

### Bridge Responsibilities

| Event | Detection Method | Tauri Command |
|-------|-----------------|---------------|
| New message received | Listen to Supabase Realtime `postgres_changes` on `messages` table | `notify_new_message` |
| Unread count changed | Count messages where conversation is not focused | `update_badge_count` |
| Window focus requested | User clicks notification (handled by Rust) | `navigate_to_conversation` |
| App visibility | Bridge reports when web app considers user "active" | `report_user_active` |

### Bridge Injection Strategy

```
1. Tauri injects initialization_script on every page load
2. Script checks for window.__TAURI__ (confirms running in Tauri)
3. Script patches into existing Supabase Realtime subscriptions
4. On new message: extract sender, text, conversation_id
5. Call invoke("notify_new_message", { sender, text, conversationId })
6. Rust side: check if window focused → if not, send OS notification
```

### Fallback (Web App Not Modified)

If the web app cannot be modified to explicitly support the bridge:
- Bridge observes DOM mutations on the message list
- Detects new message elements appearing
- Extracts text content from DOM
- Less reliable but zero web app changes needed

### Preferred (Web App Cooperation)

Web app detects `window.__TAURI__` and dispatches custom events:
```
window.dispatchEvent(new CustomEvent('tauri:new-message', {
  detail: { sender, text, conversationId }
}));
```
Bridge listens for these events — cleaner, more reliable.

## 5. Design Rationale

| Decision | Why |
|----------|-----|
| OS-native window chrome | Familiar UX. No custom title bar complexity. Consistent across platforms. |
| Tray menu for settings | Avoids building in-app settings page. Tray is standard for desktop apps. Minimal scope. |
| Native notifications over custom | OS handles DND, grouping, history. No reinventing notification UI. |
| Loading screen | Prevents jarring blank window on slow network. Sets expectation that content is loading. |
| No frameless window | Reduces complexity. Custom traffic lights/window controls are error-prone and platform-specific. |
| Remote URL over bundled | Web app updates deploy instantly without desktop app update. Decoupled release cycles. |

## 6. Platform-Specific Behaviors

| Feature | macOS | Windows | Linux |
|---------|-------|---------|-------|
| Tray location | Menu bar (top-right) | System tray (bottom-right) | App indicator / tray |
| Tray click | Left-click → menu | Right-click → menu | Left-click → menu |
| Badge count | Dock badge number | Tooltip only | Tooltip only |
| Close behavior | Hide to tray (Cmd+W) | Hide to tray (X button) | Hide to tray (X button) |
| Quit | Cmd+Q | Alt+F4 or tray menu | Ctrl+Q or tray menu |
| Auto-start | Login Items (launchd) | Registry / Startup folder | Autostart .desktop file |
| Global shortcut | Cmd+Shift+A | Ctrl+Shift+A | Ctrl+Shift+A |
| Notifications | macOS Notification Center | Windows Action Center | libnotify / D-Bus |

## 🚦 GATE 2: Requirements Validation

Before proceeding to `/ipa:design`:

- [ ] SRD.md reviewed — features match lean analysis scope
- [ ] Feature priorities (P1/P2/P3) confirmed
- [ ] JS bridge strategy decided (DOM observation vs web app cooperation)
- [ ] Web app deployment URL confirmed and accessible
- [ ] Scope still matches /lean output (no creep)
- [ ] Platform targets confirmed (macOS + Windows + Linux, or subset)

**Next:** `/ipa:detail` to generate API_SPEC.md (Tauri commands) + DB_DESIGN.md (local config schema)
