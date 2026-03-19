# release-please v4 Quick Start Guide

**For:** agent-playground-desktop (Tauri v2 + conventional commits)

---

## TL;DR

Use **release-please v4**. It's Google-maintained, handles your 3 files natively, and requires only 2 config files + 1 GitHub Actions workflow.

---

## Step 1: Add Configuration Files

### `release-please-config.json` (root)

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

### `release-please-manifest.json` (root)

```json
{
  ".": "0.2.0"
}
```

Replace `0.2.0` with your current version.

---

## Step 2: Add GitHub Actions Workflow

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
    name: Release
    steps:
      - uses: googleapis/release-please-action@v4
        id: release
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          config-file: release-please-config.json
          manifest-file: release-please-manifest.json

      # Optional: Build & upload artifacts on release
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

      - name: Install dependencies
        if: ${{ steps.release.outputs.release_created == 'true' }}
        run: pnpm install

      - name: Build
        if: ${{ steps.release.outputs.release_created == 'true' }}
        run: pnpm build
```

**Key point:** Use `release_created` (singular), NOT `releases_created` (plural).

---

## Step 3: Verify Your File Structures

Before testing, confirm the jsonpath values work:

```bash
# Verify package.json
jq '.version' package.json
# Output: "0.2.0"

# Verify tauri.conf.json uses productVersion (not version)
jq '.productVersion' src-tauri/tauri.conf.json
# Output: "0.2.0"

# Verify Cargo.toml nested path
grep -A5 '^\[package\]' src-tauri/Cargo.toml | grep version
# Output: version = "0.2.0"
```

If any path doesn't match, update the jsonpath in `release-please-config.json`.

---

## Step 4: Test It

1. Commit and push the config files to main
2. Make a commit with conventional commit message: `feat: my feature` or `fix: bug`
3. Push to main
4. Go to GitHub → Pull Requests
5. A release PR should appear (e.g., "chore(main): release 0.2.1")
6. Review the PR to confirm all 3 files are updated correctly
7. Merge the PR
8. GitHub Release is created automatically

---

## Typical Release PR

The release PR will look like:

```
Title: chore(main): release 0.2.1

Body:
📦 Release Version 0.2.1
...

Changes:
- Updated package.json from 0.2.0 → 0.2.1
- Updated src-tauri/tauri.conf.json from 0.2.0 → 0.2.1
- Updated src-tauri/Cargo.toml from 0.2.0 → 0.2.1
- Added CHANGELOG.md entry
```

---

## Troubleshooting

### PR not created after commit

**Problem:** Pushed conventional commit, but no release PR appeared.

**Solution:**
1. Check workflow status: GitHub → Actions tab
2. Confirm commit message is valid conventional commit (feat:, fix:, chore:, etc.)
3. Confirm it's pushed to `main` branch
4. release-please may rate-limit on first run (wait 5 min)

### Files not updating in release PR

**Problem:** CHANGELOG.md updated, but tauri.conf.json and Cargo.toml unchanged.

**Solution:**
1. Check jsonpath in `release-please-config.json` (most common issue)
2. Verify file actually exists at path (no typos)
3. Verify JSON/TOML syntax is valid (test locally first)

Test jsonpath:

```bash
# Test JSON path
jq '.productVersion' src-tauri/tauri.conf.json

# Test TOML path
cat src-tauri/Cargo.toml | grep -A 20 '^\[package\]' | head -5
```

If jsonpath is wrong, release-please silently skips the file (no error).

### CHANGELOG not generated

**Problem:** CHANGELOG.md file not created/updated.

**Solution:**
1. Ensure `"changelog-path": "CHANGELOG.md"` in config (or path you prefer)
2. First release PR will create it
3. Subsequent PRs will append to it

### Version mismatch across files

**Problem:** Files updated to different versions.

**Solution:**
- This shouldn't happen if using release-please (it's atomic)
- If it does: delete release PR, fix config, push again
- Always test jsonpath first

---

## Daily Workflow

1. **Develop:** Commit with conventional messages (`feat:`, `fix:`, etc.)
2. **Push:** `git push origin main`
3. **Release PR:** GitHub auto-creates release PR with all 3 files bumped
4. **Review:** Check PR to confirm versions match across all files
5. **Merge:** Click merge → GitHub auto-creates Release + CHANGELOG
6. **Done:** No manual version bumping needed

---

## Advanced: Custom CHANGELOG Sections

If you want to customize CHANGELOG format (e.g., add breaking changes section):

```json
{
  "release-type": "simple",
  "changelog-sections": [
    {
      "type": "feat",
      "section": "Features",
      "hidden": false
    },
    {
      "type": "fix",
      "section": "Bug Fixes",
      "hidden": false
    },
    {
      "type": "perf",
      "section": "Performance",
      "hidden": false
    }
  ],
  "packages": {
    ".": {
      "changelog-path": "CHANGELOG.md",
      "version-file": "package.json",
      "extra-files": [
        // ... same as before
      ]
    }
  }
}
```

---

## Monitoring

After setup, release-please requires zero maintenance. GitHub Action runs on every push to main.

To verify it's working:
- GitHub → Actions tab → "Release" workflow
- Should show successful runs after each merge

To see releases:
- GitHub → Releases tab → Shows all releases + CHANGELOG

---

## One-Time Setup Checklist

- [ ] Create `release-please-config.json` (root)
- [ ] Create `release-please-manifest.json` (root)
- [ ] Create `.github/workflows/release.yml`
- [ ] Verify jsonpath for all 3 files (package.json, tauri.conf.json, Cargo.toml)
- [ ] Commit all 3 files to git
- [ ] Push to main
- [ ] Verify workflow runs in GitHub Actions tab
- [ ] Create test commit (e.g., `feat: test release`) and push
- [ ] Check that release PR was created
- [ ] Review PR to confirm all 3 files updated
- [ ] Merge PR and verify GitHub Release created
- [ ] Done! Future commits auto-trigger releases

---

## Real-World Example: agent-playground-desktop

After setup:

```
main branch
  ↓ (push commit: feat: add dark mode)
GitHub Actions trigger
  ↓
release-please detects minor version bump
  ↓
Release PR created:
  - package.json: 0.2.0 → 0.3.0
  - src-tauri/tauri.conf.json: 0.2.0 → 0.3.0
  - src-tauri/Cargo.toml: 0.2.0 → 0.3.0
  - CHANGELOG.md: entry added
  ↓ (you merge)
GitHub Release v0.3.0 created automatically
  ↓
Users can see release on GitHub with auto-generated CHANGELOG
```

All versions stay in sync. No manual bumping.

---

## See Also

- [release-please Docs](https://github.com/googleapis/release-please)
- [Semantic Versioning Spec](https://semver.org/)
- [Conventional Commits](https://www.conventionalcommits.org/)
