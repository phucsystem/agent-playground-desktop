# Agent Playground Desktop

Native desktop shell for [Agent Playground](https://github.com/phucsystem/agent-playground) — wraps the web app in a Tauri v2 window with system tray, native notifications, and OS integration.

## Features

- **System tray** — always-running tray icon with context menu (Show/Hide, Notifications, Auto-start, Quit)
- **Native notifications** — OS-level alerts for new messages via JS bridge
- **Minimize to tray** — close button hides window; app stays running
- **Window state persistence** — remembers position, size, maximized state across restarts
- **Global shortcut** — `Cmd+Shift+A` (macOS) / `Ctrl+Shift+A` (Windows) toggles window
- **Auto-start on login** — optional launch at OS startup
- **Auto-updater** — checks GitHub Releases for updates (requires signing keys)
- **Deep links** — `agentplay://conversation/{id}` protocol handler

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri v2 (Rust backend + system webview) |
| Web content | Remote-loaded Next.js app |
| State storage | Tauri plugin-store (local JSON) |
| Notifications | Tauri notification plugin |
| Distribution | GitHub Releases + Tauri updater |

## Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- [Node.js](https://nodejs.org/) (18+)
- The [Agent Playground](https://github.com/phucsystem/agent-playground) web app running

## Quick Start

```bash
# Install dependencies
npm install

# Start the web app (in another terminal)
cd /path/to/agent-playground && npm run dev

# Launch the desktop app (connects to localhost:3000)
npm run dev

# Or point to a deployed instance
AGENT_PLAYGROUND_URL=https://your-app.com npm run dev
```

## Build

```bash
# Debug build
npm run build

# Production build (URL baked in)
AGENT_PLAYGROUND_URL=https://your-app.com npm run build
```

Build outputs:
- macOS: `src-tauri/target/release/bundle/macos/Agent Playground.app`
- Windows: `src-tauri/target/release/bundle/msi/Agent Playground_*.msi`

## Project Structure

```
agent-playground-desktop/
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs          # App setup, plugins, window events
│   │   ├── commands.rs      # IPC handlers (notifications, config, badge)
│   │   ├── tray.rs          # System tray + context menu
│   │   └── main.rs          # Entry point
│   ├── capabilities/        # Tauri permission ACL
│   ├── icons/               # App icons (all platforms)
│   ├── Cargo.toml           # Rust dependencies
│   └── tauri.conf.json      # App config
├── src/
│   ├── bridge.js            # JS bridge (webview ↔ Rust IPC)
│   └── index.html           # Loading screen
├── scripts/
│   └── patch-remote-url.sh  # CI: injects production URL into capabilities
├── .github/workflows/
│   └── release.yml          # CI/CD: macOS + Windows builds
└── docs/                    # IPA documentation
```

## Architecture

```
┌─────────────────────────────────┐
│       Tauri v2 Shell (Rust)     │
│  ┌───────────────────────────┐  │
│  │   System Webview          │  │
│  │   (Remote Next.js App)    │  │
│  │                           │  │
│  │   JS Bridge:              │  │
│  │   new-message → notify    │  │
│  │   unread-count → badge    │  │
│  └───────────────────────────┘  │
│                                 │
│  Rust: tray, notifications,     │
│  window state, shortcuts        │
└────────────┬────────────────────┘
             │ HTTPS/WSS
             ▼
   Supabase Backend (shared)
```

The desktop shell loads the web app via remote URL. A JS bridge injected on page load forwards message events from Supabase Realtime to Rust for native notifications and badge updates.

## Release Pipeline

Triggered automatically when a PR is merged to `main`. The version tag is read from `src-tauri/tauri.conf.json`.

### GitHub Secrets Required

| Secret | Purpose |
|--------|---------|
| `AGENT_PLAYGROUND_URL` | Production web app URL (baked into binary) |
| `TAURI_SIGNING_PRIVATE_KEY` | Ed25519 key for update signature verification |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Key password |

Generate signing keys:
```bash
npx tauri signer generate -- -w ~/.tauri/agent-playground.key
```

### Build Matrix

| Platform | Output | Target |
|----------|--------|--------|
| macOS | `.dmg` | Universal (aarch64 + x86_64) |
| Windows | `.msi` + `.exe` | x86_64 |

## Web App Integration

For native notifications to work, the web app should dispatch custom events when running inside Tauri:

```javascript
// Detect Tauri environment
if (window.__TAURI__) {
  // On new message received
  window.dispatchEvent(new CustomEvent('tauri:new-message', {
    detail: { sender: 'Alice', text: 'Hello!', conversationId: '123', isGroup: false }
  }));

  // On unread count change
  window.dispatchEvent(new CustomEvent('tauri:unread-count', {
    detail: { count: 5 }
  }));
}
```

## Configuration

App preferences stored locally in `plugin-store` JSON:

| Setting | Default | Description |
|---------|---------|-------------|
| `notifications_enabled` | `true` | Show OS notifications |
| `auto_start` | `false` | Launch on login |
| `minimize_to_tray` | `true` | Close hides to tray |
| `global_shortcut` | `CmdOrCtrl+Shift+A` | Toggle window hotkey |
| `web_app_url` | (empty) | Override default URL |

## License

ISC
