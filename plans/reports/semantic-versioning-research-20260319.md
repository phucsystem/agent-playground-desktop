# Semantic Versioning Research: Tauri v2 Desktop App

**Date:** March 19, 2026
**Project:** agent-playground-desktop
**Status:** Complete

## Executive Summary

For a solo-dev Tauri v2 desktop app with **version in 3 files** (package.json, src-tauri/tauri.conf.json, src-tauri/Cargo.toml) and **conventional commits** workflow, **release-please v4 is the clear winner**.

| Option | Recommendation | Rationale |
|--------|---|---|
| **release-please v4** | ✅ **RECOMMENDED** | Native JSON/TOML support, minimal overhead, human-gated, Google-maintained |
| **semantic-release v24** | ⚠️ Overkill | High complexity, no native TOML, npm-centric (unused for desktop app) |
| **changesets** | ❌ Wrong tool | Designed for monorepos, requires custom scripts, manual changeset flow |

---

## Detailed Analysis

### 1. release-please v4 (Google)

**What it does:** Parses conventional commits → calculates semver bump → creates release PR with auto-updated files → merges PR to auto-create GitHub release + CHANGELOG.

#### Strengths
- ✅ **Native multi-file support:** Built-in JSON and TOML updaters (no plugins/scripts)
- ✅ **Low overhead:** 2 config files only (release-please-config.json + manifest)
- ✅ **Human gate:** Creates PR for you to review before release (safer than auto-publish)
- ✅ **Auto GitHub releases:** Release created + CHANGELOG generated in single step
- ✅ **Zero setup burden:** No npm packages in main project, runs as GitHub Action
- ✅ **Actively maintained:** Google team, updated 2025+, 27k+ stars
- ✅ **Conventional commits:** Works out-of-the-box (no config needed)

#### Weaknesses
- ⚠️ **V4 gotcha:** Use `release_created` output, NOT `releases_created` (unreliable)
- ⚠️ **Requires two config files:** More setup than semantic-release's single .releaserc.js
- ⚠️ **Less extensible:** Can't easily add custom publish steps (e.g., upload to S3)

#### How It Handles Multi-File Versioning

Uses `extra-files` configuration with type-specific updaters:

```json
{
  "packages": {
    ".": {
      "changelog-path": "CHANGELOG.md",
      "version-file": "package.json",
      "extra-files": [
        {
          "type": "json",
          "path": "src-tauri/tauri.conf.json",
          "jsonpath": "$.productVersion"
        },
        {
          "type": "toml",
          "path": "src-tauri/Cargo.toml",
          "jsonpath": "$.package.version"
        }
      ]
    }
  }
}
```

**Update flow:**
1. Detects version bump needed from git log (conventional commits)
2. Calculates new version (semver)
3. Updates all 3 files in single commit
4. Creates release PR with updated files
5. User merges PR → auto-creates GitHub release + CHANGELOG

**Result:** All files stay in sync automatically.

#### Implementation Complexity
- Setup time: ~10 minutes
- Cognitive load: Low (2 config files, declarative)
- Learning curve: Gentle (YAML config, clear structure)

#### Maintenance
- Zero: GitHub Action runs automatically
- Google maintains it actively
- Community large enough for support

---

### 2. semantic-release v24 (js-semantic-release team)

**What it does:** Parses commits → calculates semver → publishes (npm, GitHub releases, artifacts) via plugin chain.

#### Strengths
- ✅ **Highly extensible:** Plugin ecosystem allows custom behavior
- ✅ **Single config file:** .releaserc.js (more flexible than JSON)
- ✅ **npm publishing:** Can publish to npm if you ever need it
- ✅ **Larger ecosystem:** More third-party plugins available

#### Weaknesses
- ❌ **No native TOML updater:** Must write custom @semantic-release/exec shell script
- ❌ **Complex plugin orchestration:** 3-5 plugins needed (@github, @changelog, @exec, @git)
- ❌ **Higher cognitive load:** Need to understand plugin order, environment variables
- ❌ **npm-centric design:** Built for publishing packages (wasted for desktop app)
- ❌ **Auto-publishes:** Releases immediately (no human review before going live)
- ⚠️ **Monorepo plugin unmaintained:** since March 2022 (risk if you scale)
- ⚠️ **Requires npm packages:** @semantic-release/github, @semantic-release/changelog, etc.

#### How It Handles Multi-File Versioning

Requires custom @semantic-release/exec plugin + shell script:

```javascript
// .releaserc.js
module.exports = {
  plugins: [
    "@semantic-release/commit-analyzer",
    "@semantic-release/release-notes-generator",
    "@semantic-release/changelog",
    [
      "@semantic-release/exec",
      {
        "prepareCmd": "bash scripts/update-versions.sh ${nextRelease.version}"
      }
    ],
    "@semantic-release/git",
    "@semantic-release/github"
  ]
};
```

Then in `scripts/update-versions.sh`:
```bash
#!/bin/bash
VERSION=$1
# Manual file updates
sed -i "s/\"version\": \".*\"/\"version\": \"$VERSION\"/" src-tauri/tauri.conf.json
sed -i "s/version = \".*\"/version = \"$VERSION\"/" src-tauri/Cargo.toml
```

**Update flow:**
1. Parses commits (conventional)
2. Calculates version
3. Runs custom shell script to update files
4. Creates git commit + tag
5. Auto-creates GitHub release immediately
6. Publishes to npm (if configured)

**Result:** All files updated, but via fragile string substitution (sed). Error-prone if file format changes.

#### Implementation Complexity
- Setup time: ~15-20 minutes
- Cognitive load: High (plugin chain, environment variables, shell scripts)
- Learning curve: Steep (need to understand plugin ordering, when each runs)

#### Maintenance
- Moderate: Shell scripts can break if file formats change
- js-semantic-release team maintains core (good)
- Plugin ecosystem quality varies (risky)

---

### 3. changesets (Atlassian)

**What it does:** Manages versions via explicit .changeset files (per PR) → aggregates into version bumps → syncs versions across packages.

#### Strengths
- ✅ **Purpose-built for monorepos:** Excellent if you have 5+ packages
- ✅ **Decoupled from commits:** Explicit changeset files give control
- ✅ **Polyglot support:** Can manage JS + Rust versions in same tool
- ✅ **Human gate:** Requires explicit changeset files (intentional versioning)

#### Weaknesses
- ❌ **Not designed for single apps:** Overhead for solo project
- ❌ **No native Cargo.toml support:** Need custom version sync scripts
- ❌ **Manual per-PR flow:** Each PR requires .changeset/xxx.md file
- ❌ **Less polished CHANGELOG:** Requires custom setup
- ❌ **Requires custom scripts:** For syncing package.json → Cargo.toml
- ⚠️ **Designed for teams:** API is for multi-package coordination

#### How It Handles Multi-File Versioning

Uses custom version-sync scripts:

```bash
# .changeset/config.json
{
  "commit": true,
  "fixed": [["@my/package", "src-tauri"]],
  "changelog": ["@changesets/changelog-github", { /* */ }]
}
```

Then custom script (pre-commit or post-version):
```bash
#!/bin/bash
# Sync package.json version to Cargo.toml
VERSION=$(jq -r '.version' package.json)
sed -i "s/version = \".*\"/version = \"$VERSION\"/" src-tauri/Cargo.toml
# Same for tauri.conf.json
jq ".productVersion = \"$VERSION\"" src-tauri/tauri.conf.json > tmp && mv tmp src-tauri/tauri.conf.json
```

**Update flow:**
1. Developer creates .changeset/xxx.md file (manual step)
2. Runs `changeset version` → updates package.json
3. Custom script syncs to other files
4. `changeset publish` → creates GitHub release
5. User manages CHANGELOG manually (or via plugin)

**Result:** Complex workflow, not designed for single apps.

#### Implementation Complexity
- Setup time: ~20-30 minutes
- Cognitive load: High (multi-repo paradigms, custom scripts)
- Learning curve: Steep (changeset philosophy, polyglot patterns)

#### Maintenance
- Moderate-high: Custom scripts need upkeep
- Changesets team maintains core (good)
- Not ideal for desktop apps (design mismatch)

---

## Comparison Table

| Criterion | release-please v4 | semantic-release v24 | changesets |
|-----------|-------------------|----------------------|-----------|
| **Multi-file JSON/TOML** | ✅ Native | ⚠️ Custom script | ⚠️ Custom script |
| **Setup time** | ~10 min | ~15-20 min | ~20-30 min |
| **Config files** | 2 | 1 | 1-2 |
| **CHANGELOG quality** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Human gate** | ✅ PR review | ❌ Auto-publish | ✅ Manual changeset |
| **GitHub Actions ready** | ✅ Out-of-box | ⚠️ Requires setup | ⚠️ Requires setup |
| **Cognitive load** | ⭐⭐ Low | ⭐⭐⭐⭐ High | ⭐⭐⭐ Medium |
| **Extensibility** | Medium | ⭐⭐⭐⭐⭐ High | Medium |
| **Maintenance** | ⭐ Minimal | ⭐⭐ Moderate | ⭐⭐ Moderate |
| **Design fit** | ✅ Perfect | ⚠️ Over-engineered | ❌ Wrong tool |

---

## Critical Issues & Gotchas

### release-please v4

**⚠️ Issue #1: Use `release_created`, NOT `releases_created`**
- `release_created`: Boolean (true if release happened) ✅
- `releases_created`: Boolean (true if multiple releases) ❌ Unreliable in v4

```yaml
# ✅ CORRECT
if: ${{ steps.release.outputs.release_created }}

# ❌ WRONG
if: ${{ steps.release.outputs.releases_created }}
```

**⚠️ Issue #2: extra-files may silently fail**
- If jsonpath is wrong, file won't update (no error)
- Always test in a throwaway PR first
- Common mistake: wrong nested path (e.g., $.package.version vs $.version)

**⚠️ Issue #3: Tauri v2 structure**
- tauri.conf.json uses `productVersion` (not `version`)
- Cargo.toml nested under `[package]` section
- Verify paths before committing config

### semantic-release v24

**⚠️ Issue #1: Plugin ordering matters**
- @semantic-release/exec must come before @semantic-release/git
- Git plugin commits file changes from exec
- Wrong order = files not committed

**⚠️ Issue #2: sed is fragile**
- Won't work if whitespace changes
- Version strings with special chars cause issues
- TOML parsing especially tricky (sed is text-blind)

**⚠️ Issue #3: No human review**
- Releases go live automatically
- If script fails, release is broken
- No easy rollback

### changesets

**⚠️ Issue #1: Manual per-PR burden**
- Every PR needs .changeset/xxx.md file
- Easy to forget, creates friction
- Not suitable for high-volume development

**⚠️ Issue #2: Version sync complexity**
- Custom scripts must handle all 3 files
- Harder to debug than release-please's native updaters

---

## Recommended Implementation: release-please v4

### Step 1: Configuration Files

**File: `release-please-config.json`** (root)
```json
{
  "release-type": "simple",
  "bump-minor-pre-major": false,
  "bump-patch-for-minor-pre-major": false,
  "packages": {
    ".": {
      "changelog-path": "CHANGELOG.md",
      "version-file": "package.json",
      "extra-files": [
        {
          "type": "json",
          "path": "src-tauri/tauri.conf.json",
          "jsonpath": "$.productVersion"
        },
        {
          "type": "toml",
          "path": "src-tauri/Cargo.toml",
          "jsonpath": "$.package.version"
        }
      ]
    }
  }
}
```

**File: `release-please-manifest.json`** (root)
```json
{
  ".": "0.2.0"
}
```
(This is auto-created/updated by release-please. Commit it to git.)

### Step 2: GitHub Actions Workflow

**File: `.github/workflows/release.yml`**

```yaml
name: Release

on:
  push:
    branches:
      - main

permissions:
  contents: write
  pull-requests: write

jobs:
  release-please:
    runs-on: ubuntu-latest
    name: Release
    steps:
      - uses: googleapis/release-please-action@v4
        id: release
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          config-file: release-please-config.json
          manifest-file: release-please-manifest.json

      # Optional: Trigger Tauri build on release
      - uses: actions/checkout@v4
        if: ${{ steps.release.outputs.release_created }}
        with:
          fetch-depth: 0

      - name: Setup Node
        if: ${{ steps.release.outputs.release_created }}
        uses: actions/setup-node@v4
        with:
          node-version: "20"

      - name: Setup Rust
        if: ${{ steps.release.outputs.release_created }}
        uses: dtolnay/rust-toolchain@stable

      - name: Install pnpm
        if: ${{ steps.release.outputs.release_created }}
        uses: pnpm/action-setup@v2

      - name: Build & sign app
        if: ${{ steps.release.outputs.release_created }}
        run: |
          pnpm install
          pnpm build
          # Your Tauri build command here

      - name: Upload artifacts to release
        if: ${{ steps.release.outputs.release_created }}
        uses: actions/upload-release-asset@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          upload_url: ${{ steps.release.outputs.upload_url }}
          asset_path: ./src-tauri/target/release/bundle/msi/*.msi
          asset_name: app.msi
          asset_content_type: application/octet-stream
```

### Step 3: Verify File Paths

Before deploying, verify jsonpath values in your files:

```bash
# Check package.json structure
jq '.version' package.json

# Check tauri.conf.json structure (productVersion)
jq '.productVersion' src-tauri/tauri.conf.json

# Check Cargo.toml structure (nested under [package])
toml-cli get src-tauri/Cargo.toml package.version
# Or manually: grep "^version = " src-tauri/Cargo.toml
```

If jsonpath is wrong (e.g., using `$.version` instead of `$.productVersion`), the file won't update silently.

---

## Unresolved Questions

1. **Tauri auto-updater integration:** Does release-please output work with Tauri's built-in updater system? (Likely yes, but verify)
2. **macOS/Windows signing:** How do you want to integrate code signing with the GitHub Actions workflow? (Out of scope for versioning, but needed before production)
3. **Post-release notifications:** Do you want Slack/Discord notifications when releases go live? (Can add later via workflow_run trigger)
4. **Artifact hosting:** Where do you want to host built .dmg/.exe/.msi files? (GitHub Releases, S3, CrabNebula, etc.)

---

## Sources

- [release-please GitHub](https://github.com/googleapis/release-please)
- [release-please Action - GitHub Marketplace](https://github.com/marketplace/actions/release-please-action)
- [release-please Customization Docs](https://github.com/googleapis/release-please/blob/main/docs/customizing.md)
- [release-please Config Schema](https://github.com/googleapis/release-please/blob/main/schemas/config.json)
- [Beware Release Please v4 - Medium](https://danwakeem.medium.com/beware-the-release-please-v4-github-action-ee71ff9de151)
- [semantic-release GitHub](https://github.com/semantic-release/semantic-release)
- [semantic-release GitHub Actions Docs](https://semantic-release.gitbook.io/semantic-release/recipes/ci-configurations/github-actions)
- [changesets Documentation](https://changesets-docs.vercel.app/)
- [Using Changesets in a Polyglot Monorepo](https://luke.hsiao.dev/blog/changesets-polyglot-monorepo/)
- [Tauri v2 Configuration](https://v2.tauri.app/develop/configuration-files/)
- [Tauri v2 Release & Distribution Guide](https://www.oflight.co.jp/en/columns/tauri-v2-auto-update-distribution)
