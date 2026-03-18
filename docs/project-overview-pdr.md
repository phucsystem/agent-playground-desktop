# Project Overview & Product Development Requirements (PDR)

**Project:** Agent Playground Desktop | **Version:** 0.2.0 | **Status:** Active Development

## 1. Project Vision

Agent Playground Desktop brings the collaborative Agent Playground web app into the native desktop environment with OS-level integration, persistent presence, and platform-native features. Users can maintain always-on desktop shell with notification alerts, tray icon, and keyboard shortcuts while accessing the same backend as the web app.

## 2. Business Goals

| Goal | Metric | Timeline |
|------|--------|----------|
| Increase user engagement | DAU/MAU | Ongoing |
| Reduce support friction | Native notifications reduce "missed messages" support tickets | v0.2 |
| Platform coverage | macOS + Windows support parity | v0.2 |
| Update reliability | Auto-update adoption > 80% | v0.3 |

## 3. Target Audience

**Primary:** Agent Playground users wanting always-on presence with native desktop experience
- Professionals in distributed teams
- Power users who keep chat apps open
- Users on macOS and Windows

**Secondary:** Enterprise deployments wanting OS-level integration (notifications, tray, shortcuts)

## 4. Key Differentiators

- **Minimal footprint:** 3-10MB binary vs Electron's 150MB+
- **Native experience:** System tray, OS notifications, keyboard shortcuts
- **Instant updates:** Web app changes reflect instantly (no Electron rebuild)
- **Low friction:** Remote URL + Tauri handles platform APIs
- **Type safety:** Rust backend reduces runtime crashes

## 5. Product Development Requirements

### 5.1 Functional Requirements (FR-xx)

See `/docs/SRD.md` for complete feature list. Summary:

**Phase 1 (Core Shell)** — v0.1.0
- Native window + webview
- System tray + context menu
- Minimize-to-tray behavior
- Native notifications
- JS↔Rust bridge
- Notification permission

**Phase 2 (Polish)** — v0.2.0
- Window state persistence
- Auto-start on login
- Global keyboard shortcut
- Unread badge count
- Notification grouping (deferred)

**Phase 3 (Distribution)** — v0.3+
- Auto-updater with GitHub Releases
- Deep link protocol handler
- CI/CD for macOS + Windows
- Loading/splash screen

### 5.2 Non-Functional Requirements

**Performance**
- Cold start to window: <3s (excluding web app)
- Notification latency: <500ms
- Idle memory: <150MB
- Binary size: <15MB

**Security**
- HTTPS-only remote content
- Tauri capability-based permissions
- No local credential storage
- Ed25519 signed updates
- Branch protection + CODEOWNERS review

**Compatibility**
- macOS 12+ (Monterey)
- Windows 10+ (with WebView2)
- Linux (Ubuntu 22.04+, Fedora 38+)

**Reliability**
- Network disconnection resilience
- Tray persistence even if webview crashes
- Window state recovery on crash

### 5.3 Acceptance Criteria

| Criterion | Status | Notes |
|-----------|--------|-------|
| Builds on macOS (Intel/Apple Silicon) | Pass | GitHub Actions universal binary |
| Builds on Windows (x64) | Pass | GitHub Actions x64 only |
| Notifications arrive <500ms after message | Pass | Event-driven bridge |
| Window position/size restored on restart | Pass | plugin-store persists state |
| Auto-update checks without user action | Pass | 6-hour interval background task |
| Global shortcut works system-wide | Pass | Tested Cmd+Shift+A |
| Zero credential storage locally | Pass | Web app cookies handled by browser |

### 5.4 Constraints & Dependencies

**Constraints**
- Remote URL must be HTTPS (CSP enforcement)
- Single-window app (design limitation)
- Tauri 2.10 has no dock badge API (macOS)
- Web app must run separately (not bundled)

**Dependencies**
- Agent Playground web app availability (same backend)
- Supabase Realtime for message events
- GitHub account for releases & signing keys
- Node.js 18+, Rust 1.70+

### 5.5 Out of Scope (v0.2)

- Rebuilding chat UI natively
- Local message caching / offline mode
- Multi-window / pop-out conversations
- Frameless/custom window chrome
- In-app settings page (tray menu only)
- Mobile app
- Self-hosted Supabase bundling

## 6. Architecture Overview

```
┌─────────────────────────────────────┐
│    Tauri v2 Desktop Shell (Rust)    │
│  ┌───────────────────────────────┐  │
│  │  System Webview               │  │
│  │  (Remote Next.js App)         │  │
│  │                               │  │
│  │  JS Bridge Script:            │  │
│  │  - Listen for message events  │  │
│  │  - Invoke Rust commands       │  │
│  └───────────────────────────────┘  │
│                                     │
│  Rust Layer:                        │
│  - Window mgmt (minimize-to-tray)   │
│  - System tray + menu               │
│  - Native notifications             │
│  - Window state persistence         │
│  - Auto-updater                     │
│  - Global shortcuts                 │
│  - Config/state store (JSON)        │
└──────────┬──────────────────────────┘
           │ HTTPS/WSS
           ▼
┌──────────────────────────────────────┐
│   Agent Playground Web App           │
│   (Next.js + Supabase Backend)       │
│   Chat UI, auth, business logic      │
└──────────────────────────────────────┘
```

**Key Design Decisions (D-xx)**

| ID | Decision | Rationale |
|----|----------|-----------|
| D-01 | Use Tauri not Electron | 3-10MB vs 150MB, lower memory, native APIs |
| D-02 | Load web app remotely | Independent deployment, instant web updates |
| D-03 | Tauri IPC for bridge | Native API, no extra server, secure |
| D-04 | JSON file for state | Small data volume, no SQL needed |
| D-05 | GitHub Releases + Tauri updater | Free hosting, signature verification built-in |
| D-06 | Event-driven notifications | Zero-latency, leverages web app's Realtime |

## 7. Success Metrics

**Launch Readiness (v0.2)**
- Builds successfully on all platforms
- 80%+ notification delivery within 500ms
- Zero crashers in 2-week internal testing
- All code reviewed by 2 engineers

**Post-Launch (3 months)**
- 50%+ of web app users install desktop version
- <1% crash rate
- <50ms notification latency p99
- <100MB memory footprint p95

**Year 1**
- 100k+ active installations
- Zero security incidents
- <5% monthly churn

## 8. Release Timeline

| Version | Phase | Target | Features |
|---------|-------|--------|----------|
| v0.1.0 | 1 | ✓ Shipped | Core shell, tray, notifications |
| v0.2.0 | 2 | Current | Window persistence, auto-start, shortcuts, badge count |
| v0.3.0 | 3 | Q3 2026 | Auto-updater, deep links, CI/CD, loading screen |
| v1.0.0 | — | Q4 2026 | Feature parity + docs completion |

## 9. Technology Choices

| Layer | Choice | Why |
|-------|--------|-----|
| Desktop | Tauri v2 | Small, fast, native APIs, Rust backend |
| Web | Next.js (existing) | Shared codebase, no duplication |
| Notifications | Tauri plugin | Native on all platforms, permission handling |
| Storage | plugin-store | Lightweight, no migrations needed |
| Distribution | GitHub Releases | Free, integrated with Tauri updater |
| Build | GitHub Actions | Free, works with Tauri official guides |

## 10. Risk Management

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|-----------|
| Web app URL unavailable at build | High | Low | env var + fallback, test in CI |
| Notification permission denied | Medium | Low | Graceful fallback, request on startup |
| Update signature verification fails | High | Very Low | Test keys in CI, docs for signing |
| Deep link handler doesn't register | Medium | Medium | Manual testing per OS, logs |
| WebView2 missing on Windows | Medium | Medium | Tauri installs automatically |
| Breaking Tauri plugin updates | High | Low | Pin versions, test before upgrade |

## 11. Documentation & Knowledge Transfer

**User-Facing**
- README.md: Quick start, features, tech stack
- Deployment guide: Build & release instructions

**Developer-Facing**
- SRD.md: Complete feature specifications
- API_SPEC.md: IPC command signatures & examples
- DB_DESIGN.md: Data entity schemas
- Code standards: Rust conventions, error handling
- System architecture: Diagrams, data flow
- Codebase summary: File-by-file breakdown

## 12. Ownership & Contact

**Project Lead:** phucsystem (GitHub CODEOWNERS)
**Repository:** github.com/phucsystem/agent-playground-desktop
**Related:** github.com/phucsystem/agent-playground (web app)

## 13. Approval & Sign-Off

| Stakeholder | Approval | Date |
|-------------|----------|------|
| Project Lead | — | — |
| Tech Lead | — | — |
| Security Review | — | — |

---

**PDR Version History**

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-19 | Initial documentation based on v0.2.0 implementation |
