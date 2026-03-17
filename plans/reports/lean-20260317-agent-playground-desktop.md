# Lean MVP Analysis: Agent Playground Desktop

## Problem Statement

Agent Playground is a web-based chat platform for human-agent collaboration. Web browsers lack persistent presence — users must keep a tab open, notifications require browser permission (often blocked), and there's no system tray or global shortcuts. A desktop app solves the "always available" problem: persistent connection, native notifications, and quick access via tray icon — critical for a real-time collaboration tool.

**Target users:** Same as web app — developers and teams collaborating with AI agents who want a dedicated, always-on desktop experience.

## Target Users (→ IPA User Roles)

| User Type | Description | Primary Need |
|-----------|-------------|--------------|
| Developer | Uses agent-playground daily for AI collaboration | Always-on desktop presence, instant message notifications |
| Team Lead / Admin | Manages agents and workspace members | Quick access to admin panel, webhook status visibility |

## MVP Features (→ IPA Feature List FR-xx)

| Priority | Feature | User Value | Screen | Assumption |
|----------|---------|------------|--------|------------|
| P1 | Native window wrapping web app | Dedicated app experience, not a browser tab | Main Window | Users prefer dedicated app over browser tab |
| P1 | System tray icon | Always running, minimize to tray, quick access | Tray Menu | Users want persistent presence without visible window |
| P1 | Native OS notifications | Reliable message alerts without browser permission | OS Notification | Browser notifications are unreliable/blocked |
| P1 | Window state persistence | Remember size, position, maximized state across restarts | Main Window | Users expect desktop apps to remember window state |
| P2 | Auto-start on login | App launches automatically, stays in tray | OS Setting | Power users want zero-friction availability |
| P2 | Global keyboard shortcut | Toggle window visibility from anywhere | OS Shortcut | Frequent users want instant access |
| P2 | Unread badge count | Show unread count on tray/dock icon | Tray/Dock | Users need at-a-glance unread status |
| P3 | Auto-updater | Seamless app updates without manual download | Update Dialog | Desktop apps need update mechanism |
| P3 | Deep link protocol | `agentplay://` URL handler for opening conversations | URL Handler | Team sharing of conversation links |

## Implementation Phases (Estimated)

| Phase | Focus | Key Features | Effort |
|-------|-------|--------------|--------|
| 1 | Core Shell | Tauri v2 window + load remote URL + tray icon + native notifications | M |
| 2 | Polish | Auto-start, global shortcut, unread badge, window state persistence | S |
| 3 | Distribution | Auto-updater, deep links, CI/CD for builds (macOS/Windows/Linux) | M |

## Plan Structure Preview

```
plans/{date}-agent-playground-desktop/
├── plan.md
├── phase-01-core-shell/
│   ├── core.md          # Tauri setup, tray, notifications
│   └── ui.md            # Window config, tray menu
├── phase-02-polish/
│   ├── core.md          # Auto-start, shortcuts, badge
│   └── ui.md            # Window state, UX tweaks
└── phase-03-distribution/
    ├── core.md          # Auto-updater, deep links
    └── data.md          # CI/CD, signing, release
```

## 🚦 GATE 1: Scope Validation

Before proceeding to `/ipa:spec`, complete this checklist:

- [ ] Confirmed web app is stable enough to wrap (no critical bugs)
- [ ] Confirmed deployment URL is accessible for remote loading
- [ ] MVP scope acceptable (3 phases ✅)
- [ ] Assumptions documented for later validation
- [ ] Team aligned on priorities

**⚠️ Scope is exactly 3 phases — on the edge. Keep P3 features minimal.**

## MVP Screens (→ IPA Screen List S-xx)

| Screen | Purpose | Features |
|--------|---------|----------|
| S-01 Main Window | Primary app window loading web app | Webview, title bar, window controls |
| S-02 Tray Menu | Right-click system tray menu | Show/Hide, Quit, Status indicator |
| S-03 Notification | Native OS notification for new messages | Title, body, click-to-open conversation |
| S-04 Update Dialog | Auto-update prompt | Version info, Install/Later buttons |

## Data Entities (→ IPA Entity List E-xx)

| Entity | Description | Key Fields |
|--------|-------------|------------|
| E-01 AppConfig | Local app preferences (persisted) | window_state, auto_start, global_shortcut, notification_enabled |
| E-02 WindowState | Window geometry persistence | x, y, width, height, maximized |

**Note:** All chat data lives in Supabase (web app backend). Desktop app stores only local preferences.

## User Flow (→ IPA Screen Flow)

```
[App Launch] → [Main Window: loads web app URL]
                    ↕
              [Tray Icon: always visible]
                    ↓
         [Close Window → minimize to tray]
                    ↓
         [New Message → OS Notification]
                    ↓
         [Click Notification → restore window + navigate to conversation]
```

## Tech Decisions (→ IPA Key Decisions D-xx)

| Decision | Context | Chosen | Rationale |
|----------|---------|--------|-----------|
| D-01 Desktop framework | Wrap web app in native shell | Tauri v2 | ~3MB binary, system webview, Rust backend, modern API, lower memory than Electron |
| D-02 Web app loading | Local bundle vs remote URL | Remote URL | Web app already deployed; no need to bundle Next.js. Simpler updates — web deploys independently |
| D-03 Notification bridge | How desktop knows about new messages | JS→Rust IPC via Tauri events | Web app JS detects new message → calls Tauri invoke → Rust sends OS notification |
| D-04 State storage | Where to persist app preferences | Tauri plugin-store (JSON file) | Simple key-value store, no database needed for local config |
| D-05 Build/release | How to distribute updates | Tauri updater plugin + GitHub Releases | Built-in updater with signature verification, CI builds via GitHub Actions |

## Architecture Overview

```
┌─────────────────────────────────────┐
│          Tauri v2 Shell             │
│  ┌───────────────────────────────┐  │
│  │     System Webview            │  │
│  │  ┌─────────────────────────┐  │  │
│  │  │  Agent Playground       │  │  │
│  │  │  (Remote Next.js App)   │  │  │
│  │  │                         │  │  │
│  │  │  JS Bridge:             │  │  │
│  │  │  - detect new messages  │  │  │
│  │  │  - invoke Tauri APIs    │  │  │
│  │  └─────────────────────────┘  │  │
│  └───────────────────────────────┘  │
│                                     │
│  Rust Backend:                      │
│  - System tray management           │
│  - Native notifications             │
│  - Window state persistence         │
│  - Auto-start configuration         │
│  - Global shortcut registration     │
│  - Auto-updater                     │
└─────────────────────────────────────┘
         │
         │ HTTPS/WSS
         ▼
┌─────────────────────┐
│  Supabase Backend   │
│  (existing, shared  │
│   with web app)     │
└─────────────────────┘
```

## JS↔Rust Bridge (Critical Integration Point)

The web app needs a small JS bridge to communicate with Tauri:

```
Web App (JS) ──invoke──→ Tauri (Rust)
  │                           │
  │ "new message arrived"     │ → sendNotification()
  │ "unread count: 5"         │ → updateTrayBadge(5)
  │ "window.focus requested"  │ → showWindow()
```

**Implementation:** Inject a small script via Tauri's `initialization_script` that detects `window.__TAURI__` and bridges events from Supabase Realtime to Tauri commands.

## Nice-to-Have (Post-MVP)

- **Idle detection** — auto-set presence to "away" based on OS idle time
- **File drag-drop** — native file drop onto chat (vs file picker)
- **Multiple windows** — pop-out conversations into separate windows
- **Offline queue** — queue messages when disconnected, send on reconnect
- **Custom themes** — dark/light following OS preference (beyond web app's own theme)

## Key Assumptions to Validate

1. **System webview compatibility** — Tauri uses system WebView (WebKit on macOS, WebView2 on Windows). Verify the Next.js app renders correctly. Validate by: testing on macOS Safari + Windows Edge.
2. **Remote URL performance** — Loading remote URL adds network latency vs bundled app. Validate by: measuring cold start time on various connections.
3. **JS bridge feasibility** — Tauri's `initialization_script` can inject bridge code into remote pages. Validate by: building POC with message detection.
4. **Notification click routing** — Clicking notification can deep-link to specific conversation. Validate by: testing Tauri notification action → window navigation.

## Out of Scope

- Rebuilding the chat UI natively (React Native Desktop, SwiftUI, etc.)
- Local message caching / offline mode (complex, low ROI for MVP)
- Mobile app (separate project, different framework)
- Self-hosted/local Supabase backend bundling
- End-to-end encryption (not in web app, not in desktop)

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| WebView compatibility issues | UI may render differently than Chrome | Test early on macOS/Windows/Linux webviews; add CSS workarounds |
| Remote URL blocked by CSP/CORS | App can't load web content | Configure Tauri security settings; whitelist domains |
| Tauri v2 ecosystem maturity | Plugins may have bugs or missing features | Pin stable versions; have fallback implementations |
| Code signing costs | macOS/Windows require signed binaries for distribution | Use Apple Developer + Windows code signing certs; budget ~$100-400/yr |
| Web app changes break bridge | JS bridge relies on DOM/event structure | Version the bridge API; add integration tests |

## Next Step

After GATE 1 validation:
→ Run `/ipa:spec` to generate SRD.md + UI_SPEC.md
→ Then `/ipa:detail` for API_SPEC.md (Tauri commands) + local config schema
