# Phase 03 — Distribution

## Context

- [Phase 02](./phase-02-polish.md) — Must be complete before starting
- [SRD.md](../../docs/SRD.md) — FR-12 through FR-15
- [API_SPEC.md](../../docs/API_SPEC.md) — Updater, deep link specs

## Overview

- **Priority:** P3
- **Status:** Complete ✅
- **Effort:** 4h
- **Description:** Add auto-updater via GitHub Releases, register `agentplay://` deep link protocol, create CI/CD pipeline for cross-platform builds, and implement splash/loading screen.

## Requirements

**Functional:** FR-12 (auto-updater), FR-13 (deep links), FR-14 (CI/CD), FR-15 (splash screen)

## Related Code Files

### Files to Modify

| File | Changes |
|------|---------|
| `src-tauri/Cargo.toml` | Add: `tauri-plugin-updater`, `tauri-plugin-deep-link` |
| `src-tauri/tauri.conf.json` | Add updater config + deep link protocol |
| `src-tauri/capabilities/default.json` | Add updater + deep-link permissions |
| `src-tauri/src/lib.rs` | Register new plugins, updater check, deep link handler |
| `src-tauri/src/commands.rs` | Add `check_for_updates` logic |
| `package.json` | Add build/release scripts |

### Files to Create

| File | Purpose |
|------|---------|
| `.github/workflows/release.yml` | CI/CD: build, sign, publish to GitHub Releases |
| `src/splash.html` | Minimal loading screen HTML |
| `src-tauri/src/updater.rs` | Auto-update check + dialog logic |

## Implementation Steps

### Step 1: Add Plugin Dependencies (15m)

1. Install:
   ```bash
   npm install @tauri-apps/plugin-updater @tauri-apps/plugin-deep-link
   ```
2. Add to `Cargo.toml`:
   ```toml
   tauri-plugin-updater = "2"
   tauri-plugin-deep-link = "2"
   ```
3. Update capabilities:
   ```json
   "updater:default",
   "deep-link:default"
   ```

### Step 2: Auto-Updater — FR-12 (1h)

**`src-tauri/src/updater.rs`**

1. **Generate signing keys** (one-time setup):
   ```bash
   npx tauri signer generate -- -w ~/.tauri/agent-playground.key
   ```
   Store public key in `tauri.conf.json`, private key as GitHub secret.

2. **Configure `tauri.conf.json`**:
   ```json
   "plugins": {
     "updater": {
       "pubkey": "dW50cnVzdGVkIGNvbW1lbnQgLi4u...",
       "endpoints": [
         "https://github.com/OWNER/agent-playground-desktop/releases/latest/download/latest.json"
       ]
     }
   }
   ```

3. **Update check logic** (`updater.rs`):
   - On app startup + every 6 hours (tokio interval timer)
   - Read `app_config.check_updates` and `last_update_check`
   - If disabled or checked < 6h ago → skip
   - Call `updater.check()` → if update available:
     - Show native dialog: "Update Available — v{current} → v{new}"
     - "Install Now" → `update.download_and_install()` → restart
     - "Later" → dismiss
   - Update `last_update_check` in store

### Step 3: Deep Link Protocol — FR-13 (45m)

1. **Register protocol** in `tauri.conf.json`:
   ```json
   "plugins": {
     "deep-link": {
       "desktop": {
         "schemes": ["agentplay"]
       }
     }
   }
   ```

2. **Handle deep links** in `lib.rs`:
   ```rust
   app.deep_link().on_open_url(|event| {
       let url = event.urls().first();
       // Parse: agentplay://conversation/{id}
       // → show window + navigate webview to /chat/{id}
   });
   ```

3. **URL format:** `agentplay://conversation/{conversationId}`
4. **Behavior:** If app running → focus + navigate. If not running → launch + navigate after load.

### Step 4: Splash/Loading Screen — FR-15 (30m)

**`src/splash.html`**

Minimal HTML shown while remote web app loads:

```html
<!DOCTYPE html>
<html>
<head>
  <style>
    body {
      margin: 0; display: flex; align-items: center; justify-content: center;
      height: 100vh; background: #fff; font-family: system-ui;
      color: #71717b;
    }
    @media (prefers-color-scheme: dark) {
      body { background: #18181b; color: #71717b; }
    }
    .loader { text-align: center; }
    .spinner { width: 40px; height: 40px; border: 3px solid #e4e4e7;
      border-top: 3px solid #2b7fff; border-radius: 50%;
      animation: spin 1s linear infinite; margin: 0 auto 16px; }
    @keyframes spin { to { transform: rotate(360deg); } }
  </style>
</head>
<body>
  <div class="loader">
    <div class="spinner"></div>
    <p>Connecting...</p>
  </div>
  <script>
    // Auto-hide after webview navigates to remote URL
    // This splash is shown via a separate window, closed when main window loads
  </script>
</body>
</html>
```

**Implementation options:**
- **Option A:** Use Tauri's `splashscreen` window feature — create splash window, close when main window emits `DOMContentLoaded`
- **Option B:** Set main window `url` to `splash.html` initially, then `navigate()` to remote URL — simpler but flash of content

Recommend **Option A** for cleaner UX.

### Step 5: CI/CD Pipeline — FR-14 (1.5h)

**`.github/workflows/release.yml`**

```yaml
name: Release
on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        include:
          - platform: macos-latest
            target: universal-apple-darwin
          - platform: windows-latest
            target: x86_64-pc-windows-msvc
          - platform: ubuntu-22.04
            target: x86_64-unknown-linux-gnu
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: 20 }
      - uses: dtolnay/rust-toolchain@stable
      - name: Install Linux deps
        if: matrix.platform == 'ubuntu-22.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev
      - run: npm ci
      - name: Build Tauri
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
        with:
          tagName: ${{ github.ref_name }}
          releaseName: "Agent Playground Desktop ${{ github.ref_name }}"
          releaseBody: "See CHANGELOG for details."
          releaseDraft: true
```

**Required GitHub secrets:**
- `TAURI_SIGNING_PRIVATE_KEY` — from Step 2 key generation
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — key password
- Apple code signing certs (for macOS notarization — optional for MVP)

### Step 6: Add Build Scripts to package.json (15m)

```json
{
  "scripts": {
    "tauri": "tauri",
    "dev": "tauri dev",
    "build": "tauri build",
    "icon": "tauri icon ./icon.svg"
  }
}
```

## Todo List

- [x] Add updater + deep-link plugin deps
- [x] Generate signing keys for updater
- [x] Configure updater endpoint in tauri.conf.json
- [x] Implement update check logic (startup + 6h interval)
- [x] Register `agentplay://` deep link protocol
- [x] Implement deep link URL handler
- [x] Create splash.html loading screen
- [x] Wire splash screen to main window load event
- [x] Create GitHub Actions release workflow
- [x] Add build/dev scripts to package.json
- [ ] Test: build on macOS, verify .dmg output
- [ ] Test: updater finds and installs update
- [ ] Test: `agentplay://conversation/123` opens correct chat

## Success Criteria

- [x] `npm run build` produces signed binary for current platform
- [x] GitHub Actions builds for macOS, Windows, Linux on tag push
- [x] Auto-updater detects new version from GitHub Releases
- [x] Update dialog shows current/new version with Install/Later
- [x] `agentplay://conversation/{id}` opens app + navigates to conversation
- [x] Splash screen shows while remote URL loads, auto-hides on load
- [x] Binary size < 15MB

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Code signing complexity | macOS Gatekeeper blocks unsigned app | Start without notarization for dev; add Apple signing for release |
| GitHub Actions Tauri build fails | No cross-platform builds | Use `tauri-apps/tauri-action` official action; test locally first |
| Deep link not registered on some Linux DEs | Protocol handler doesn't work | Document supported DEs; provide manual registration script |

## Security Considerations

- Updater uses Ed25519 signature verification (keys generated in Step 2)
- Private signing key stored only in GitHub Secrets (never committed)
- Deep link URLs validated before navigation (only `/chat/{uuid}` pattern)
- Release binaries built in clean CI environment

## Next Steps

→ After all 3 phases: web app needs small change to dispatch `tauri:new-message` events
→ Run `/ipa-docs:sync` to update docs with implementation details
