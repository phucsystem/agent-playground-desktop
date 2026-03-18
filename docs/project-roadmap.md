# Project Roadmap

**Version:** 0.2.0 | **Updated:** 2026-03-19 | **Scope:** v0.1 → v1.0

## Overview

Agent Playground Desktop follows a 3-phase roadmap aligned with feature priority and deployment complexity. Each phase builds on the previous without breaking changes.

## Phase Timeline

```
Phase 1 (Core Shell)     Phase 2 (Polish)        Phase 3 (Distribution)
v0.1.0 (Shipped)         v0.2.0 (Current)        v0.3.0 (Q3 2026)
├─ Window + Tray         ├─ Window persistence   ├─ Auto-updater
├─ Notifications         ├─ Auto-start           ├─ Deep links
├─ JS Bridge             ├─ Global shortcuts     ├─ CI/CD pipeline
│                        ├─ Badge count          ├─ Loading screen
│                        │                       │
Q1 2025 ────────────── Q1 2026 ───────────── Q3 2026
```

## Phase 1: Core Shell (v0.1.0) — SHIPPED

**Status:** Complete | **Release Date:** 2025-Q1

### Features (FR-01 through FR-06)

| Feature | Status | Details |
|---------|--------|---------|
| FR-01 Native window with webview | ✓ | Tauri window loads Agent Playground URL. 1200×800, min 800×600, centered, resizable. |
| FR-02 System tray icon | ✓ | Persistent tray with context menu (Show/Hide, Quit). Double-click toggles visibility. |
| FR-03 Minimize to tray | ✓ | Close button minimizes instead of quits. App continues running in background. |
| FR-04 Native OS notifications | ✓ | Event-driven: web app → JS bridge → Rust → OS notification. Supports title, body. Clickable. |
| FR-05 JS↔Rust bridge | ✓ | Initialization script injected via Tauri. Polls for __TAURI__, forwards custom events. |
| FR-06 Notification permission | ✓ | Request on first launch. Toggle via tray menu. Stored in AppConfig. |

### Success Criteria (Phase 1)

- [x] Builds on macOS (Intel + Apple Silicon)
- [x] Builds on Windows (x64)
- [x] Notification arrives <500ms after message dispatch
- [x] Window opens centered on first launch
- [x] Close button hides window (minimize to tray)
- [x] Tray menu shows/hides window
- [x] Notification permission requested once on startup
- [x] Bridge script loads before web app content

### Known Issues (Phase 1)

None documented. Phase 1 stable.

## Phase 2: Polish (v0.2.0) — CURRENT

**Status:** In Progress | **Target Release:** 2026-03-31

### Features (FR-07 through FR-11)

| Feature | ID | Status | Details |
|---------|----|---------|---------
| Window state persistence | FR-07 | ✓ | Save position (x, y), size (width, height), maximized on close. Restore on next launch. |
| Auto-start on login | FR-08 | ✓ | Option to launch automatically at OS startup. Starts minimized to tray. Configurable via menu. |
| Global keyboard shortcut | FR-09 | ✓ | System-wide hotkey (Cmd+Shift+A macOS, Ctrl+Shift+A Windows). Toggle window visibility. |
| Unread badge count | FR-10 | ✓ | Display count on tray icon tooltip. Update via JS bridge. Reset when window focused. |
| Notification grouping | FR-11 | 🔄 | Replace previous notification from same conversation. Deferred to v0.3. |

### Implementation Status

```
Code                [████████] 90%
├─ lib.rs           [████████] Window restoration
├─ commands.rs      [████████] Config get/set
├─ tray.rs          [████████] Menu toggling
├─ bridge.js        [████████] Event forwarding
└─ AppConfig        [████████] Persistence

Testing             [███░░░░░] 35%
├─ Manual macOS     [██░░░░░░] In progress
├─ Manual Windows   [░░░░░░░░] Pending
├─ Bridge timing    [██████░░] Needs edge cases
└─ Config restore   [██████░░] Basic scenarios

Docs                [████████] 95%
├─ SRD.md           [████████] Updated
├─ API_SPEC.md      [████████] Detailed
├─ DB_DESIGN.md     [████████] Schemas defined
└─ Code standards   [████████] Published
```

### Success Criteria (Phase 2)

- [x] Window position/size restored on restart
- [x] Auto-start toggles via tray menu
- [x] Global shortcut registered without conflicts
- [x] Badge count updates in real-time
- [x] Config changes persisted across restarts
- [ ] Notification grouping (deferred)
- [ ] Full manual testing on all platforms (in progress)

### Known Issues (Phase 2)

| Issue | Severity | Notes |
|-------|----------|-------|
| Badge count updates slow in plugin-store (>100ms) | Low | Optimize debouncing if needed in v0.3 |
| macOS: No dock badge API | Medium | Tauri limitation, deferred to v2.11+ |
| Loading screen not integrated with page load | Low | HTML only, no connection to webview events |

## Phase 3: Distribution (v0.3.0) — Q3 2026

**Status:** Planned | **Target Release:** 2026-Q3

### Features (FR-12 through FR-15)

| Feature | ID | Status | Details |
|---------|----|----|---------|
| Auto-updater | FR-12 | 🔄 | Check GitHub Releases on startup + every 6h. Show dialog, user decides Install Now/Later. |
| Deep link protocol | FR-13 | 🔄 | Register `agentplay://conversation/{id}`. Click link → open/focus app → navigate to conversation. |
| CI/CD build pipeline | FR-14 | 🔄 | GitHub Actions: PR merge → build macOS universal + Windows → sign → release. |
| Splash/loading screen | FR-15 | 🔄 | Show minimal spinner while web app loads. Hide on DOMContentLoaded. |

### Implementation Roadmap

**Month 1 (Auto-Updater)**
- [ ] Implement version check via GitHub Releases API
- [ ] Add update dialog UI (native, platform-specific)
- [ ] Implement download + install flow
- [ ] Test with test release keys
- [ ] Document update signing process

**Month 2 (Deep Links)**
- [ ] Implement `agentplay://` protocol handler (platform-specific code)
- [ ] Parse URL, extract conversation ID
- [ ] Navigate webview on deep link click
- [ ] Test across macOS, Windows, Linux
- [ ] Document protocol in README

**Month 2-3 (CI/CD Pipeline)**
- [ ] Create GitHub Actions workflow for multi-platform builds
- [ ] Implement Ed25519 signing for updates
- [ ] Set up release artifact publishing
- [ ] Document signing key setup for maintainers
- [ ] Test full release cycle end-to-end

**Month 3 (Loading Screen)**
- [ ] Connect loading screen to webview `DOMContentLoaded` event
- [ ] Hide spinner when content loads
- [ ] Test on slow connections
- [ ] Optimize spinner CSS for dark/light mode

### Success Criteria (Phase 3)

- [ ] Auto-updater checks on startup and every 6 hours
- [ ] Update dialog appears only when new version available
- [ ] User can defer update and continue working
- [ ] Deep link protocol registered on all platforms
- [ ] Clicking `agentplay://conversation/{id}` opens app and navigates
- [ ] CI/CD pipeline creates unsigned builds and publishes to Releases
- [ ] Ed25519 signature verification works end-to-end
- [ ] Loading screen visible for >200ms on slow connections
- [ ] Zero test failures on all platforms

## Version Milestones

### v0.1.0 (Shipped)

**Release Date:** Q1 2025

**What's included:**
- Native window + tray
- Notifications
- JS bridge
- Notification permission

**Install instructions:** Manual build from source

---

### v0.2.0 (Current)

**Release Date:** 2026-03-31 (target)

**What's new:**
- Window state persistence (position, size, maximized)
- Auto-start on login
- Global keyboard shortcut (Cmd+Shift+A / Ctrl+Shift+A)
- Unread message badge on tray
- Improved bridge reliability

**Deprecations:** None

**Breaking changes:** None

**Download:** github.com/phucsystem/agent-playground-desktop/releases/tag/v0.2.0

---

### v0.3.0 (Planned)

**Target Release:** Q3 2026

**What's new:**
- Auto-updater with GitHub Releases integration
- Deep link protocol support (`agentplay://`)
- CI/CD pipeline for automated builds
- Integrated loading screen
- Notification grouping

**Deprecations:** None expected

**Breaking changes:** None expected

**Platform support:** macOS (universal) + Windows (x64)

---

### v1.0.0 (Future)

**Target Release:** Q4 2026

**Milestones:**
- Feature parity with web app (desktop no longer behind)
- 100% documentation coverage
- 100k+ active installations
- <1% crash rate
- Official app store availability (optional)

## Risk Management & Mitigation

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|-----------|
| GitHub API rate limiting blocks update checks | High | Low | Cache version check result for 1h |
| Deep link registration conflicts on Windows | Medium | Medium | Test with other apps, document handling |
| Tauri plugin breaking changes in minor update | High | Low | Pin major versions, test before upgrade |
| Bridge injection timing race (web app too fast) | Medium | Low | Already polling __TAURI__ (50x @ 100ms) |
| Updater signature key compromised | Critical | Very Low | Key rotation procedure documented |
| Auto-start fails silently on some Windows installs | Medium | Low | Manual tests + logging in updater.rs |

## Dependencies & Blockers

**Phase 3 blockers:**
- [ ] GitHub signing keys generated and stored securely
- [ ] Test release published to verify CI/CD works
- [ ] Ed25519 signature verification tested end-to-end

**Ongoing dependencies:**
- Supabase backend must remain available (shared with web app)
- Agent Playground web app must remain compatible with remote loading
- GitHub must remain available for releases + updater endpoints

## Metrics & Success Tracking

### Phase 2 Success Metrics

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Build time (local) | <2 min | ~90s | On track |
| Binary size | <15MB | ~11MB | Pass |
| Cold start time | <3s | ~1.5s | Pass |
| Notification latency p95 | <500ms | ~250ms | Pass |
| Manual test coverage | 80%+ | 35% | In progress |
| Code review approval | 2/2 required | Pending | Pending |

### Phase 3 Success Metrics (Target)

| Metric | Target |
|--------|--------|
| Auto-update adoption rate | 80%+ |
| Update latency (latest.json) | <100ms |
| Deep link click-to-navigation | <2s |
| CI/CD pipeline reliability | 99%+ |
| Zero security incidents | — |

## Documentation Roadmap

**v0.2 (Current):**
- [x] README.md (quick start)
- [x] SRD.md (feature list)
- [x] API_SPEC.md (IPC commands)
- [x] DB_DESIGN.md (data schemas)
- [x] UI_SPEC.md (screen layouts)
- [x] Code standards.md (conventions)
- [x] System architecture.md (design diagrams)
- [x] Project roadmap.md (this file)
- [ ] Deployment guide.md (build + release)
- [ ] Codebase summary.md (code organization)

**v0.3:**
- [ ] Security hardening guide
- [ ] Deep link integration guide (for web app)
- [ ] Update signing key management
- [ ] Troubleshooting guide

**v1.0:**
- [ ] Architecture decision records (ADRs)
- [ ] Performance tuning guide
- [ ] Plugin upgrade guide
- [ ] Contribution guidelines

## Stakeholder Sign-Off

| Stakeholder | Phase 1 | Phase 2 | Phase 3 |
|-------------|--------|--------|---------|
| Project Lead (phucsystem) | ✓ | 🔄 | — |
| Tech Lead | ✓ | 🔄 | — |
| QA / Testing | ✓ | 🔄 | — |
| Security Review | ✓ | 🔄 | — |

---

**Last Updated:** 2026-03-19

**Next Review:** 2026-04-15 (Phase 2 completion assessment)
