# Semantic Versioning Research - Complete Analysis

**Project:** agent-playground-desktop (Tauri v2)
**Objective:** Find best tool for automated version bumping across package.json, src-tauri/tauri.conf.json, src-tauri/Cargo.toml
**Date:** March 19, 2026

---

## Quick Answer

**Use: release-please v4 (Google)**

- ✅ Native JSON/TOML support
- ✅ Zero npm package overhead
- ✅ Human-gated releases (PR review before going live)
- ✅ Auto CHANGELOG + GitHub Release
- ✅ 25 minutes to implement
- ✅ Google-maintained, actively updated

---

## Report Files

Read in this order:

### 1. **RECOMMENDATION.md** (Start here)
- Executive summary
- Why release-please wins
- Why others lose
- 4-step implementation plan
- Total setup time: ~25 minutes

### 2. **semantic-versioning-research-20260319.md** (Full analysis)
- Detailed pros/cons for all 3 options
- Multi-file handling explained
- Critical issues & gotchas
- Comparison table
- Implementation details for recommended approach

### 3. **release-please-quick-start.md** (Step-by-step guide)
- Exact configuration files to create
- Copy-paste YAML workflow
- Verification commands
- Daily workflow after setup
- Troubleshooting guide

### 4. **example-github-actions-workflow.md** (Code examples)
- Complete workflow files for all 3 options
- Configuration examples
- Testing checklist
- Monitoring guide

---

## TL;DR Comparison

| | release-please v4 | semantic-release v24 | changesets |
|---|---|---|---|
| **Best for** | Desktop apps, single codebase | npm packages, complex pipelines | Monorepos (5+ packages) |
| **Multi-file support** | ✅ Native | ⚠️ Custom scripts | ⚠️ Custom scripts |
| **Setup time** | 10 min | 15-20 min | 20-30 min |
| **Human gate** | ✅ PR review | ❌ Auto-publish | ✅ Manual |
| **Maintenance** | None | Moderate | Moderate |
| **Recommendation** | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐ |

---

## Key Decision Points

### ✅ Why release-please v4

1. **Native TOML/JSON updaters** — No shell scripts needed
   - Just configure jsonpath in release-please-config.json
   - Files updated atomically, error-safe

2. **Minimal overhead** — 2 config files only
   - No npm packages needed in project
   - No dev dependencies, no build impact

3. **Human gate** — PR-based releases
   - Create → Review → Merge → Release workflow
   - vs. semantic-release auto-publishes immediately

4. **Built for GitHub Actions** — Designed as Action, not tool
   - Runs directly as GitHub Action
   - Zero local setup needed

5. **Auto CHANGELOG** — Generated from commits
   - No manual CHANGELOG entries
   - Semantic versioning + conventional commits = perfect match

### ❌ Why NOT semantic-release v24

1. **Too powerful** — Designed for npm publishing
   - You're not publishing npm packages
   - Paying complexity tax for unused features

2. **No native TOML** — Requires custom shell scripts
   - sed/awk fragile and error-prone
   - Harder to debug when it breaks

3. **Auto-publish** — No human review gate
   - Releases go live immediately
   - If script fails, release is broken
   - No easy rollback

4. **Plugin hell** — 3-5 plugins needed
   - Plugin ordering matters (footgun)
   - Each plugin adds complexity
   - Ecosystem quality varies

### ❌ Why NOT changesets

1. **Wrong tool** — Designed for monorepos
   - You have single Tauri app
   - Monorepo features are overhead

2. **Manual per-PR** — .changeset/*.md files required
   - Extra friction in workflow
   - Easy to forget

3. **No native Cargo.toml** — Custom scripts needed
   - Same fragility issues as semantic-release

4. **Less polished** — CHANGELOG needs manual setup
   - release-please's CHANGELOG superior

---

## What Changes After Setup

### Before
```
1. Write code
2. Commit + push
3. Merge PR
4. ❌ MANUAL: Edit package.json (0.2.0 → 0.3.0)
5. ❌ MANUAL: Edit src-tauri/tauri.conf.json
6. ❌ MANUAL: Edit src-tauri/Cargo.toml
7. ❌ MANUAL: Write CHANGELOG.md entry
8. ❌ MANUAL: Create git tag v0.3.0
9. ❌ MANUAL: Create GitHub release

Total time: 15-20 minutes per release
```

### After
```
1. Write code
2. Commit (with conventional message: feat: / fix: / etc.)
3. Push to main
4. ✅ AUTOMATIC: release-please creates Release PR
5. ✅ Review PR (30 seconds): Confirm versions match
6. ✅ Merge Release PR
7. ✅ AUTOMATIC: GitHub Release created + CHANGELOG updated

Total time: 2-3 minutes per release
```

**Result: No manual version bumping ever again.**

---

## Implementation Roadmap

### Phase 1: Add Config Files (5 min)
```bash
# Create release-please-config.json
# Create release-please-manifest.json
# Commit both to git
```

### Phase 2: Add GitHub Actions (5 min)
```bash
# Create .github/workflows/release.yml
# Commit to git
```

### Phase 3: Verify Paths (5 min)
```bash
# Test jsonpath values work
jq '.productVersion' src-tauri/tauri.conf.json
jq '.package.version' src-tauri/Cargo.toml
jq '.version' package.json
```

### Phase 4: Test (10 min)
```bash
# Create dummy commit with feat: message
# Verify Release PR appears
# Verify all 3 files have correct version bump
# (Don't merge if just testing)
```

### Phase 5: Deploy (2 min)
```bash
# Next real feature commit
# Push to main
# release-please creates Release PR automatically
# Merge → GitHub Release created
# Done!
```

**Total time to live: ~25 minutes**

---

## Files to Create

All files go to repository root except workflow:

```
.
├── .github/workflows/release.yml         [NEW]
├── release-please-config.json            [NEW]
├── release-please-manifest.json          [NEW]
└── (No changes to existing files)
```

**Configuration is JSON — fully testable before deployment.**

---

## Gotchas & Prevention

### ⚠️ #1: Wrong jsonpath value
**Symptom:** File not updating in Release PR

**Prevention:** Test before deploying
```bash
jq '.productVersion' src-tauri/tauri.conf.json
jq '.package.version' src-tauri/Cargo.toml
jq '.version' package.json
```

### ⚠️ #2: Manifest file not committed
**Symptom:** release-please errors out

**Prevention:** Commit `release-please-manifest.json` to git

### ⚠️ #3: Using `releases_created` instead of `release_created`
**Symptom:** Conditional steps don't run

**Prevention:** Use exact spelling: `release_created` (singular)

### ⚠️ #4: Commits not following conventional format
**Symptom:** No Release PR appears

**Prevention:** Use `feat:`, `fix:`, `chore:`, `docs:` prefixes

---

## After Going Live

**Maintenance:** Essentially zero
- GitHub Action runs automatically
- Google maintains release-please
- No moving parts in your codebase

**Monitoring:**
- Check GitHub Actions tab occasionally
- Verify releases appear on Releases page
- Done!

---

## Quick References

- **Semantic Versioning:** https://semver.org/
- **Conventional Commits:** https://www.conventionalcommits.org/
- **release-please:** https://github.com/googleapis/release-please
- **Tauri Configuration:** https://v2.tauri.app/develop/configuration-files/

---

## Next Steps

1. ✅ Read **RECOMMENDATION.md** (this file)
2. ✅ Review **example-github-actions-workflow.md** for complete code
3. ✅ Follow **release-please-quick-start.md** step-by-step
4. ✅ Create config files (copy-paste from examples)
5. ✅ Test with dummy commit
6. ✅ Deploy on next real feature commit

**Questions?** Check **semantic-versioning-research-20260319.md** for detailed analysis.

---

**Research completed:** March 19, 2026
**Status:** Ready for implementation
