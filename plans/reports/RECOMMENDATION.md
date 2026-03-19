# Semantic Versioning Recommendation: release-please v4

**Project:** agent-playground-desktop (Tauri v2 desktop app)
**Date:** March 19, 2026
**Status:** Ready to implement

---

## Recommendation

### ✅ Use release-please v4 (Google)

**Why:**
1. **Native multi-file support** — JSON & TOML updaters built-in (no custom scripts)
2. **Minimal overhead** — 2 config files only (no npm packages in project)
3. **Human-gated releases** — Creates release PR for review before going live
4. **Auto CHANGELOG** — Generated automatically from conventional commits
5. **Zero setup burden** — Runs as GitHub Action, not a dev dependency
6. **Actively maintained** — Google team, 27k+ GitHub stars, updated 2025+
7. **Conventional commits** — Works out-of-box (already using this)

**Fits your workflow perfectly:**
- ✅ Solo developer (no team coordination overhead)
- ✅ Tauri v2 (multi-file versioning needed)
- ✅ GitHub Actions (integrates seamlessly)
- ✅ Conventional commits (leverages existing practice)
- ✅ PR-based workflow (human review before release)

---

## What You Get

**Before:** Manual version bumping in 3 files + manual CHANGELOG + manual GitHub release
```
❌ Edit package.json (0.2.0 → 0.3.0)
❌ Edit src-tauri/tauri.conf.json (0.2.0 → 0.3.0)
❌ Edit src-tauri/Cargo.toml (0.2.0 → 0.3.0)
❌ Write CHANGELOG.md entry
❌ Create git tag
❌ Manually create GitHub release
```

**After:** Automatic on every PR merge
```
✅ Push: feat: add dark mode
✅ GitHub: Auto-creates release PR with all 3 files bumped
✅ You: Review PR (1 click to merge)
✅ GitHub: Auto-creates release + CHANGELOG
✅ Done (2 minutes total)
```

---

## Head-to-Head Comparison

| Aspect | release-please v4 | semantic-release v24 | changesets |
|--------|-------------------|----------------------|-----------|
| Multi-file JSON/TOML | ✅ Native | ⚠️ Custom script | ⚠️ Custom script |
| Setup time | 10 min | 15-20 min | 20-30 min |
| Config complexity | Low (2 files) | Medium (1 file + plugins) | Medium (2 files + scripts) |
| Learning curve | Gentle | Steep | Medium-steep |
| Maintenance burden | None | Moderate | Moderate |
| Auto CHANGELOG | ✅ Yes | ✅ Yes | Partial |
| Human gate | ✅ Yes (PR) | ❌ Auto-publish | ✅ Yes (manual) |
| Design fit | ✅ Perfect | ⚠️ Over-engineered | ❌ Wrong tool |

---

## Why NOT the Others

### ❌ semantic-release v24

**The problem:** Designed for npm packages, overkill for desktop apps.

- ❌ No native TOML updater (need custom shell scripts)
- ❌ Auto-publishes immediately (no human review)
- ❌ Requires 3-5 npm packages (@semantic-release/github, @semantic-release/exec, etc.)
- ❌ Complex plugin orchestration (steep learning curve)
- ❌ Shell script fragility (sed breaks easily)

**When to use:** If you're publishing npm packages or need complex custom logic.

---

### ❌ changesets (Atlassian)

**The problem:** Designed for monorepos, wrong tool for single apps.

- ❌ Requires manual .changeset/xxx.md per PR (friction)
- ❌ No native Cargo.toml support (custom scripts needed)
- ❌ Designed for multi-package coordination (unused overhead)
- ❌ Less polished CHANGELOG generation
- ❌ Higher maintenance burden

**When to use:** If you have 5+ interdependent packages in a monorepo.

---

## Implementation Steps

### 1. Add 2 Config Files (5 min)

**`release-please-config.json`** at root:
```json
{
  "release-type": "simple",
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

**`release-please-manifest.json`** at root:
```json
{
  ".": "0.2.0"
}
```
(Replace 0.2.0 with your current version)

### 2. Add GitHub Actions Workflow (5 min)

Create `.github/workflows/release.yml`:

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
    steps:
      - uses: googleapis/release-please-action@v4
        id: release
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          config-file: release-please-config.json
          manifest-file: release-please-manifest.json

      # Optional: Build Tauri app on release
      - uses: actions/checkout@v4
        if: ${{ steps.release.outputs.release_created == 'true' }}

      - name: Setup Node
        if: ${{ steps.release.outputs.release_created == 'true' }}
        uses: actions/setup-node@v4
        with:
          node-version: "20"

      - name: Install pnpm
        if: ${{ steps.release.outputs.release_created == 'true' }}
        uses: pnpm/action-setup@v2

      - name: Install & build
        if: ${{ steps.release.outputs.release_created == 'true' }}
        run: |
          pnpm install
          pnpm build
```

### 3. Verify File Paths (5 min)

```bash
# Verify paths exist and structure is correct
jq '.productVersion' src-tauri/tauri.conf.json
grep "^version = " src-tauri/Cargo.toml
jq '.version' package.json
```

### 4. Test (10 min)

1. Push the 3 files to main
2. Create a test commit: `feat: test release`
3. Push to main
4. GitHub → Pull Requests → Should see "chore(main): release X.X.X"
5. Review PR to confirm all 3 files bumped
6. Merge → Verify GitHub Release created

**Total setup time: ~25 minutes**

---

## Gotchas to Watch

⚠️ **Use `release_created` (singular), NOT `releases_created` (plural)**
- `release_created` = safe to use ✅
- `releases_created` = unreliable in v4 ❌

⚠️ **Verify jsonpath values BEFORE deploying**
- tauri.conf.json uses `productVersion` (not `version`)
- Cargo.toml is nested under `[package]`
- Wrong path = file won't update (silent failure)

⚠️ **Commit manifest file to git**
- `release-please-manifest.json` tracks versions
- Without it, release-please won't work

---

## What You Keep

Your current workflow stays the same:
- ✅ Conventional commit messages (feat:, fix:, chore:, docs:)
- ✅ PR-based development
- ✅ Merge to main triggers release (now automated)

**New:** Automatic version bumping + CHANGELOG + GitHub Release

---

## After Implementation

Your day-to-day changes:

```
Before:
1. Feature code
2. Commit + push
3. Merge PR
4. ❌ Manual: bump 3 versions
5. ❌ Manual: write CHANGELOG
6. ❌ Manual: create release tag
7. ❌ Manual: create GitHub release

After:
1. Feature code
2. Commit + push
3. GitHub auto-creates release PR
4. ✅ Review & merge release PR
5. ✅ GitHub auto-creates release
Done (step 3-5 = 2 minutes)
```

---

## Files to Create/Modify

```
agent-playground-desktop/
├── .github/workflows/
│   └── release.yml                  [CREATE]
├── release-please-config.json       [CREATE]
├── release-please-manifest.json     [CREATE]
├── package.json                     [NO CHANGE - auto-updated]
├── src-tauri/tauri.conf.json        [NO CHANGE - auto-updated]
└── src-tauri/Cargo.toml             [NO CHANGE - auto-updated]
```

**No changes to application code or existing workflows.**

---

## Success Criteria

After setup:
1. ✅ Push conventional commit to main
2. ✅ Release PR appears within 1 minute
3. ✅ All 3 files have version bumped in PR
4. ✅ CHANGELOG.md has entry
5. ✅ Merge PR → GitHub Release created automatically

---

## Next Steps

1. **Implement:** Follow setup steps above (25 min)
2. **Test:** Create dummy release to verify all 3 files update
3. **Deploy:** Integrate with Tauri build step (optional, separate concern)
4. **Monitor:** Verify GitHub Actions runs successfully on next real commit

---

## Reference Documents

- **Full Analysis:** `semantic-versioning-research-20260319.md`
- **Quick Start:** `release-please-quick-start.md`
- **GitHub:** [release-please-action](https://github.com/googleapis/release-please-action)

---

**Recommendation Status:** Ready to implement
