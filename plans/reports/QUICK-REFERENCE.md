# Semantic Versioning Quick Reference Card

**Decision:** Use **release-please v4**
**Setup Time:** 25 minutes
**Maintenance:** Zero

---

## In 60 Seconds

release-please v4 (by Google) automatically bumps versions in 3 files when you merge PRs to main.

| Before | After |
|--------|-------|
| Manual edit 3 files + changelog + git tag | Automatic via Release PR |
| 15-20 min per release | 2-3 min per release |

---

## Files to Create

### 1. `release-please-config.json` (root)

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

### 2. `release-please-manifest.json` (root)

```json
{
  ".": "0.2.0"
}
```
(Replace 0.2.0 with your current version)

### 3. `.github/workflows/release.yml`

```yaml
name: Release

on:
  push:
    branches: [main]

permissions:
  contents: write
  pull-requests: write

jobs:
  release-please:
    runs-on: ubuntu-latest
    steps:
      - uses: googleapis/release-please-action@v4
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          config-file: release-please-config.json
          manifest-file: release-please-manifest.json
```

---

## Critical Details

### jsonpath Values (DON'T GET THESE WRONG)

```
package.json              → $.version
tauri.conf.json           → $.productVersion  (NOT $.version)
Cargo.toml                → $.package.version
```

Test before deploying:
```bash
jq '.version' package.json
jq '.productVersion' src-tauri/tauri.conf.json
cat src-tauri/Cargo.toml | grep "^version = "
```

### GitHub Actions Conditional

```yaml
# CORRECT (use this):
if: ${{ steps.release.outputs.release_created == 'true' }}

# WRONG (never use this):
if: ${{ steps.release.outputs.releases_created }}
```

---

## Daily Workflow

```
1. Code feature
2. Commit: git commit -m "feat: my feature"  (use feat:, fix:, chore:, docs:)
3. Push: git push origin main
4. GitHub auto-creates Release PR
5. Review PR (30 sec): confirm all 3 files updated
6. Merge PR
7. GitHub auto-creates Release + CHANGELOG
```

That's it. Fully automated after step 4.

---

## Testing Before Live

```bash
# 1. Create config files + workflow
# 2. Commit all 3 files
# 3. Push to main
# 4. Create dummy commit
git commit --allow-empty -m "feat: test release"
git push origin main

# 5. Check GitHub Actions tab
# Should see Release workflow running

# 6. Expect Release PR on Pull Requests tab
# Should show version bumps in all 3 files

# 7. Don't merge yet - just verify it's correct
# If correct: delete branch, ready for real use
# If wrong: fix config and retry
```

---

## If Something Goes Wrong

| Symptom | Cause | Fix |
|---------|-------|-----|
| No Release PR appears | Commit not `feat:`/`fix:` | Use conventional commit prefix |
| Files not updating | Wrong jsonpath | Test jq/grep, update config |
| Workflow errors | Missing manifest file | Create release-please-manifest.json |
| Can't test locally | Not needed | Only runs on GitHub Actions |

---

## Why release-please v4?

```
✅ Native JSON/TOML support (no shell scripts)
✅ Google-maintained (reliable)
✅ Zero npm overhead (GitHub Action)
✅ Human gate (PR review before release)
✅ Auto CHANGELOG (from commits)

vs

❌ semantic-release: No TOML support, requires 3+ npm packages, auto-publishes
❌ changesets: Designed for monorepos, manual per-PR, custom scripts
```

---

## Success Criteria After Setup

- [ ] Push conventional commit to main
- [ ] Release PR appears within 1 min
- [ ] All 3 files have bumped versions
- [ ] CHANGELOG.md has entry
- [ ] Merge PR → GitHub Release auto-created
- [ ] Future releases work automatically

---

## Zero Maintenance After Setup

That's the goal. release-please runs automatically on every push to main.

Monitor: GitHub → Actions tab (should show green checkmarks)

---

## References

- **Full report:** `/plans/reports/semantic-versioning-research-20260319.md`
- **Step-by-step:** `/plans/reports/release-please-quick-start.md`
- **Examples:** `/plans/reports/example-github-actions-workflow.md`
- **Analysis:** `/plans/reports/RECOMMENDATION.md`

---

## Next Step

Follow `/plans/reports/release-please-quick-start.md` for exact setup.

**Estimated time:** 25 minutes from zero to live.
