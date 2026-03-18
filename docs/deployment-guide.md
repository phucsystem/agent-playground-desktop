# Deployment & Build Guide

**Version:** 0.2.0 | **Updated:** 2026-03-19 | **Audience:** Developers, maintainers, CI/CD operators

## 1. Local Development Setup

### 1.1 Prerequisites

Ensure these are installed and in your PATH:

```bash
# Check Rust
rustup --version      # 1.70.0+
rustc --version       # Matches rustup version

# Check Node
node --version        # 18.0.0+
npm --version         # 9.0.0+

# Check build essentials
xcode-select --version     # macOS
# or
gcc --version              # Linux
# or
choco list | grep build    # Windows (if using Chocolatey)
```

### 1.2 Clone & Install

```bash
# Clone repository
git clone https://github.com/phucsystem/agent-playground-desktop.git
cd agent-playground-desktop

# Install Node dependencies
npm install

# Install Rust dependencies (automatic via Cargo.toml)
cargo build --manifest-path src-tauri/Cargo.toml
```

### 1.3 Point to Web App

The desktop app loads a remote web app. Specify the URL:

**Development (localhost):**
```bash
# Uses default localhost:3000 from tauri.conf.json
npm run dev
```

**Staging/Production:**
```bash
# Inject production URL
AGENT_PLAYGROUND_URL=https://agent-playground-staging.example.com npm run dev

# For building release binary
AGENT_PLAYGROUND_URL=https://agent-playground.example.com npm run build
```

**Env var resolution:**
1. Build-time: `AGENT_PLAYGROUND_URL` env var (if set)
2. Runtime fallback: `tauri.conf.json` → localhost:3000

### 1.4 Start Development Server

```bash
# Terminal 1: Start the web app (if local)
cd /path/to/agent-playground
npm run dev
# Listens on http://localhost:3000

# Terminal 2: Start the desktop app
cd agent-playground-desktop
npm run dev
# Builds Rust backend, starts Tauri dev server
# Opens window connecting to localhost:3000
```

Expected output:
```
✔ Rust project built successfully
✔ Webview frontend built successfully
   Running web app on http://localhost:3000
   Desktop app window opened
```

## 2. Building for Distribution

### 2.1 Production Build Command

Build optimized binaries for release:

```bash
# macOS (universal binary - arm64 + x86_64)
AGENT_PLAYGROUND_URL=https://your-app.com npm run build

# Output:
# src-tauri/target/release/bundle/macos/Agent\ Playground.app
# src-tauri/target/release/bundle/macos/Agent\ Playground.dmg
```

```bash
# Windows (x64 only)
set AGENT_PLAYGROUND_URL=https://your-app.com && npm run build

# Output:
# src-tauri/target/release/bundle/msi/Agent\ Playground_*.msi
# src-tauri/target/release/Agent\ Playground.exe
```

### 2.2 Build Configuration

**tauri.conf.json** (version reference):
```json
{
  "version": "0.2.0",
  "productName": "Agent Playground",
  "build": {
    "beforeBuildCommand": "bash scripts/patch-remote-url.sh"
  }
}
```

**Key behavior:**
- Reads version from `src-tauri/Cargo.toml` (not package.json)
- Runs `patch-remote-url.sh` before build to inject production URL
- Creates code-signed binaries (requires signing keys on macOS)

### 2.3 Build Artifacts

After successful build:

| Platform | Output Path | Artifact | Size |
|----------|-------------|----------|------|
| macOS | `src-tauri/target/release/bundle/macos/` | `Agent Playground.dmg` | ~50-70MB |
| Windows | `src-tauri/target/release/bundle/msi/` | `Agent Playground_v0.2.0_x64.msi` | ~30-40MB |

Note: DMG and MSI sizes include web content cache and are larger than binary.

## 3. Code Signing & Notarization

### 3.1 macOS Code Signing

Tauri requires valid Apple Developer identity:

```bash
# List available signing identities
security find-identity -v -p codesigning

# Build with specific identity (macOS only)
npm run build
# Tauri auto-detects default signing identity
```

**For CI/CD:** Store certificate + password in GitHub Secrets:
- `APPLE_SIGNING_IDENTITY` (e.g., "Developer ID Application: Your Name (TEAM_ID)")
- `APPLE_SIGNING_CERTIFICATE_BASE64` (base64 encoded .p12 file)
- `APPLE_SIGNING_CERTIFICATE_PASSWORD`

### 3.2 Tauri Updater Signing

The auto-updater (v0.3+) requires Ed25519 signature verification:

```bash
# Generate signing keys (one-time setup)
npx tauri signer generate -- -w ~/.tauri/agent-playground.key

# Output:
# Secret key: (base64, keep secure!)
# Public key: (base64, add to tauri.conf.json)
```

**Store in GitHub Secrets:**
- `TAURI_SIGNING_PRIVATE_KEY` (full key content)
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (if password-protected)

**Public key goes in tauri.conf.json:**
```json
{
  "plugins": {
    "updater": {
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDRFMkYyOTY2ODAwRTc4MzcKUldRM2VBNkFaaWt2VGtIaWE0QlhIeERQdDRDVG9lRitoSHFNbmRLZlZjcG1NL0xBZDhFaUhSd3EK"
    }
  }
}
```

### 3.3 Key Rotation Procedure

When signing keys need rotation (security event, key compromise):

1. Generate new keys: `npx tauri signer generate -- -w ~/.tauri/agent-playground-new.key`
2. Test new keys with test release build
3. Update GitHub Secrets with new private key
4. Update public key in tauri.conf.json
5. Create git commit + PR (mark as security fix)
6. Document key rotation in CHANGELOG

## 4. CI/CD Pipeline

### 4.1 GitHub Actions Workflow

File: `.github/workflows/release.yml`

**Trigger:** PR merge to `main` branch

```yaml
name: Release
on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
          AGENT_PLAYGROUND_URL: ${{ secrets.AGENT_PLAYGROUND_URL }}
```

**Environment variables required in GitHub:**
| Secret | Value | Purpose |
|--------|-------|---------|
| `GITHUB_TOKEN` | Auto (GitHub Actions) | Upload artifacts |
| `TAURI_SIGNING_PRIVATE_KEY` | Ed25519 private key | Sign update manifest |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Key password | Decrypt private key |
| `AGENT_PLAYGROUND_URL` | https://your-app.com | Production URL |

### 4.2 Release Workflow

```
1. Developer commits feature on branch
2. Push to GitHub, create PR
3. PR approved + merged to main
4. GitHub Actions automatically triggers
5. Builds macOS universal + Windows x64
6. Signs binaries with Ed25519 key
7. Creates GitHub Release (draft)
8. Uploads DMG, MSI, latest.json to release
```

Manual step: Publish release from draft to public (no auto-publish yet).

### 4.3 Monitoring CI/CD

Check build status:

```bash
# View recent workflow runs
gh run list --repo phucsystem/agent-playground-desktop

# View specific run details
gh run view <RUN_ID> --log --repo phucsystem/agent-playground-desktop

# View logs for specific job
gh run view <RUN_ID> --job <JOB_ID> --log
```

## 5. Release Management

### 5.1 Creating a Release

**Manual steps (until full automation):**

1. Verify all tests pass locally:
   ```bash
   npm run build
   ```

2. Update version in `src-tauri/Cargo.toml`:
   ```toml
   [package]
   version = "0.2.1"
   ```

3. Commit + tag:
   ```bash
   git add src-tauri/Cargo.toml
   git commit -m "chore(release): bump version to 0.2.1"
   git push origin main
   ```

4. PR will trigger GitHub Actions automatically
5. Once build completes, navigate to GitHub Releases
6. Edit draft release:
   - Add release notes
   - Verify all artifacts present (DMG, MSI)
   - Click "Publish release"

### 5.2 Release Notes Template

```markdown
# Agent Playground Desktop v0.2.1

## What's Changed
- Fixed bridge timing race on slow connections
- Improved notification permission handling
- Updated Tauri to v2.1.0

## Installation
- **macOS:** Download Agent Playground.dmg, drag to Applications
- **Windows:** Download Agent Playground_v0.2.1_x64.msi, run installer

## Known Issues
- None

## Contributors
- @phucsystem

---
**SHA-256 Checksums:** [See release artifacts]
```

### 5.3 Version Numbering

Follow semantic versioning (MAJOR.MINOR.PATCH):
- `0.1.0` → `0.2.0`: New features, API changes (minor version)
- `0.2.0` → `0.2.1`: Bug fixes only (patch version)
- `1.0.0` → `2.0.0`: Breaking changes (major version)

## 6. Update Distribution

### 6.1 Latest.json Feed

The auto-updater checks for updates via `latest.json`:

**Format:**
```json
{
  "version": "0.2.1",
  "url": "https://github.com/phucsystem/agent-playground-desktop/releases/download/v0.2.1/Agent%20Playground.dmg",
  "signature": "...Ed25519 signature..."
}
```

**Location:** `https://github.com/phucsystem/agent-playground-desktop/releases/latest/download/latest.json`

**Tauri auto-generates** this during release workflow. Ensure it's included in release assets.

### 6.2 Update Checking

Users' apps check for updates:
- On app startup
- Every 6 hours (background)
- Manually via tray menu ("Check for Updates")

No user action required for checking; dialog only appears if update available.

## 7. Troubleshooting

### Build Errors

**"Could not find Rust toolchain"**
```bash
rustup update
cargo --version  # Verify
npm run build    # Retry
```

**"Web app URL not injected"**
```bash
# Check if AGENT_PLAYGROUND_URL was set
echo $AGENT_PLAYGROUND_URL

# Check remote-access.json was patched
cat src-tauri/capabilities/remote-access.json | grep https
```

**"Code signature invalid" (macOS)**
```bash
# Check available signing identities
security find-identity -v -p codesigning

# Force specific identity (if multiple)
TAURI_SIGNING_IDENTITY="Developer ID Application: Name (TEAM_ID)" npm run build
```

### Runtime Issues

**"Bridge script not loaded"**
- Check: `src/bridge.js` exists
- Check: Tauri initializes with `withGlobalTauri: true` in lib.rs
- Check: Network connectivity to web app

**"Notifications not showing"**
- Check: `notifications_enabled` in AppConfig
- Check: macOS/Windows permissions (Settings → Notifications)
- Check: Notification permission was granted (first launch)

**"Update check fails silently"**
- Check: GitHub API reachable
- Check: `latest.json` exists in release assets
- Check: Ed25519 signature valid
- Check: Version in tauri.conf.json is valid semver

## 8. Rollback Procedure

If a release has critical bugs:

1. Mark release as "prerelease" on GitHub (no auto-update)
2. Revert tauri.conf.json version to previous stable
3. Fix bug on git branch
4. Test locally
5. Create new release with bumped version
6. Users auto-update to new stable version

## 9. Platform-Specific Notes

### macOS

- **Signing:** Requires Apple Developer ID Application certificate
- **Notarization:** Tauri v2 auto-notarizes, but submit issue if fails
- **Minimum:** macOS 12 (Monterey)
- **Architecture:** Universal binary (arm64 + x86_64)

### Windows

- **MSVC requirement:** Visual Studio Build Tools or full Visual Studio
- **WebView2:** Auto-installed if missing (handled by Tauri)
- **Minimum:** Windows 10
- **Architecture:** x64 only (no ARM64 yet)

### Linux (Future)

- Not yet supported
- Planned for v0.3+ as secondary target
- Will use WebKitGTK for webview

## 10. Deployment Checklist

Before releasing to production:

- [ ] All PR reviews approved
- [ ] Unit tests passing (if any)
- [ ] Manual testing on macOS + Windows
- [ ] Version bumped in Cargo.toml
- [ ] CHANGELOG updated with new features
- [ ] Signing keys valid and up-to-date
- [ ] Release notes drafted
- [ ] GitHub Secrets verified (TAURI_SIGNING_PRIVATE_KEY, etc.)
- [ ] Build artifacts signed and checksummed
- [ ] latest.json generated and verified
- [ ] Release published to GitHub
- [ ] Announcement sent to users (if major feature)

---

**Last Updated:** 2026-03-19

**Contact:** phucsystem@github

**Related:**
- [GitHub Releases](https://github.com/phucsystem/agent-playground-desktop/releases)
- [Tauri Docs](https://tauri.app/v1/docs/)
- [Code Signing Guide](https://tauri.app/v1/guides/distribution/sign)
