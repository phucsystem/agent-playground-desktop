---
title: "Agent Playground Desktop"
description: "Tauri v2 desktop shell wrapping Agent Playground web app with tray, notifications, and OS integration"
status: completed
priority: P1
effort: 16h
branch: main
tags: [desktop, tauri, rust, feature]
created: 2026-03-17
---

# Agent Playground Desktop — Implementation Plan

## Overview

Build a Tauri v2 desktop app that wraps the existing Agent Playground web app (remote URL) with system tray, native OS notifications, window persistence, and desktop integration. No chat logic — all chat handled by web app via webview.

## Architecture

```
Tauri v2 Shell (Rust)
├── main window (webview → remote Next.js app)
├── system tray (icon + context menu)
├── JS bridge (initialization_script → IPC commands)
├── plugin-store (local JSON config)
└── plugins: notification, autostart, global-shortcut, updater, deep-link
```

## Phases

| # | Phase | Status | Effort | Priority | Link |
|---|-------|--------|--------|----------|------|
| 1 | Core Shell | Complete ✅ | 8h | P1 | [phase-01](./phase-01-core-shell.md) |
| 2 | Polish | Complete ✅ | 4h | P2 | [phase-02-polish](./phase-02-polish.md) |
| 3 | Distribution | Complete ✅ | 4h | P3 | [phase-03-distribution](./phase-03-distribution.md) |

## Dependencies

- Agent Playground web app deployed and accessible via HTTPS
- Rust toolchain installed (rustup)
- Node.js 18+ (Tauri CLI)
- Tauri v2 CLI (`@tauri-apps/cli@^2`)

## Execution Order

```
Phase 1 (Core Shell) → Phase 2 (Polish) → Phase 3 (Distribution)
         ↓                    ↓                    ↓
     FR-01..06            FR-07..11            FR-12..15
```

Sequential — each phase builds on the previous.

## Key Files (Final State)

```
agent-playground-desktop/
├── src-tauri/
│   ├── Cargo.toml                    # Rust deps + Tauri plugins
│   ├── tauri.conf.json               # App config, window, security
│   ├── capabilities/
│   │   └── default.json              # Permission ACL
│   ├── icons/                        # App icons (all platforms)
│   ├── src/
│   │   ├── lib.rs                    # Tauri app setup + plugin registration
│   │   ├── main.rs                   # Entry point
│   │   ├── commands.rs               # IPC command handlers
│   │   ├── tray.rs                   # System tray setup + menu
│   │   ├── window.rs                 # Window state management
│   │   └── bridge.rs                 # JS initialization script content
│   └── build.rs                      # Tauri build script
├── src/
│   └── bridge.js                     # JS bridge (initialization_script source)
├── package.json                      # Node deps (Tauri CLI + JS API)
├── .github/
│   └── workflows/
│       └── release.yml               # CI/CD build + sign + publish
├── docs/                             # IPA docs (existing)
└── plans/                            # Plans (existing)
```

## Docs Reference

- [SRD.md](../../docs/SRD.md) — 15 functional requirements
- [UI_SPEC.md](../../docs/UI_SPEC.md) — Screen specs + JS bridge design
- [API_SPEC.md](../../docs/API_SPEC.md) — 11 Tauri IPC commands
- [DB_DESIGN.md](../../docs/DB_DESIGN.md) — Local JSON store schema
