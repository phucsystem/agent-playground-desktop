# GitHub Actions Workflow Examples

Complete, production-ready workflows for semantic versioning in your Tauri v2 desktop app.

---

## ✅ RECOMMENDED: release-please v4

**File:** `.github/workflows/release.yml`

```yaml
name: Release

on:
  push:
    branches:
      - main

permissions:
  contents: write
  pull-requests: write

env:
  RUST_BACKTRACE: 1

jobs:
  release-please:
    runs-on: ubuntu-latest
    name: Release
    steps:
      # Step 1: Create release PR or publish release
      - uses: googleapis/release-please-action@v4
        id: release
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          config-file: release-please-config.json
          manifest-file: release-please-manifest.json

      # Step 2: Checkout code if release was created
      - uses: actions/checkout@v4
        if: ${{ steps.release.outputs.release_created == 'true' }}
        with:
          fetch-depth: 0

      # Step 3: Setup Node.js
      - name: Setup Node.js
        if: ${{ steps.release.outputs.release_created == 'true' }}
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'pnpm'

      # Step 4: Setup Rust
      - name: Setup Rust
        if: ${{ steps.release.outputs.release_created == 'true' }}
        uses: dtolnay/rust-toolchain@stable

      # Step 5: Install pnpm
      - name: Install pnpm
        if: ${{ steps.release.outputs.release_created == 'true' }}
        uses: pnpm/action-setup@v2
        with:
          version: 9

      # Step 6: Install dependencies
      - name: Install dependencies
        if: ${{ steps.release.outputs.release_created == 'true' }}
        run: pnpm install --frozen-lockfile

      # Step 7: Build the app
      - name: Build application
        if: ${{ steps.release.outputs.release_created == 'true' }}
        run: pnpm build

      # Step 8: (Optional) Build Tauri app
      - name: Install and build Tauri
        if: ${{ steps.release.outputs.release_created == 'true' }}
        run: |
          cd src-tauri
          cargo build --release

      # Step 9: (Optional) Upload artifacts
      - name: Upload release artifacts
        if: ${{ steps.release.outputs.release_created == 'true' }}
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          # Example: upload built artifacts to release
          # This depends on your Tauri build outputs
          echo "Release artifacts would be uploaded here"
```

**What it does:**
1. On push to main, release-please checks for version bump needed
2. Creates PR if bump detected (or publishes release if PR was merged)
3. If release published, builds Tauri app
4. Optionally uploads artifacts to GitHub Release

**Key points:**
- Uses `release_created == 'true'` (safest check)
- Conditional steps only run on actual releases (not every push)
- Node + Rust setup for full Tauri build
- Can add artifact upload after build completes

---

## Configuration Files

### `release-please-config.json`

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

**Explanation:**
- `release-type: simple` — No monorepo, single app
- `version-file: package.json` — Primary version source
- `extra-files` — Sync to tauri.conf.json and Cargo.toml

### `release-please-manifest.json`

```json
{
  ".": "0.2.0"
}
```

Update `0.2.0` to your current version.

---

## Alternative: semantic-release v24

**File:** `.github/workflows/release.yml`

```yaml
name: Semantic Release

on:
  push:
    branches:
      - main

permissions:
  contents: write
  issues: write
  pull-requests: write

jobs:
  semantic-release:
    runs-on: ubuntu-latest
    name: Semantic Release
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - uses: actions/setup-node@v4
        with:
          node-version: '20'

      # Install semantic-release + plugins
      - name: Install dependencies
        run: |
          npm install \
            semantic-release \
            @semantic-release/changelog \
            @semantic-release/git \
            @semantic-release/github \
            @semantic-release/exec

      # Run semantic-release
      - name: Release
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: npx semantic-release
```

**With `.releaserc.js` config:**

```javascript
module.exports = {
  branches: ['main'],
  plugins: [
    '@semantic-release/commit-analyzer',
    '@semantic-release/release-notes-generator',
    '@semantic-release/changelog',
    [
      '@semantic-release/exec',
      {
        'prepareCmd': 'bash scripts/update-versions.sh ${nextRelease.version}'
      }
    ],
    '@semantic-release/git',
    '@semantic-release/github'
  ]
};
```

**Plus custom script** `scripts/update-versions.sh`:

```bash
#!/bin/bash
set -e
VERSION=$1

echo "Updating version to $VERSION..."

# Update tauri.conf.json
jq ".productVersion = \"$VERSION\"" src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp
mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json

# Update Cargo.toml
sed -i "s/^version = .*/version = \"$VERSION\"/" src-tauri/Cargo.toml

echo "✓ Version updated to $VERSION"
```

**⚠️ Problems:**
- Requires npm packages in your project
- Custom script is fragile (sed breaks easily)
- Auto-publishes (no human review gate)
- More complex than release-please

---

## Alternative: changesets

**File:** `.github/workflows/release.yml`

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
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      # changesets CLI
      - uses: actions/setup-node@v4
        with:
          node-version: '20'

      - run: npm install -g changesets

      # Update versions
      - name: Version
        run: |
          changesets version
          # Custom sync script
          bash scripts/sync-versions.sh

      # Create release
      - name: Publish
        run: changesets publish

      # Commit changes
      - name: Commit and push
        run: |
          git add .
          git commit -m "chore: release versions"
          git push
```

**Custom sync script** `scripts/sync-versions.sh`:

```bash
#!/bin/bash
# Sync package.json version to Cargo.toml and tauri.conf.json

VERSION=$(jq -r '.version' package.json)

# Update Cargo.toml
sed -i "s/^version = .*/version = \"$VERSION\"/" src-tauri/Cargo.toml

# Update tauri.conf.json
jq ".productVersion = \"$VERSION\"" src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp
mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json
```

**⚠️ Problems:**
- Requires manual changeset files per PR
- Custom scripts needed for version sync
- Less automated than release-please
- Designed for monorepos

---

## Comparison: Outputs After Workflow Runs

### release-please v4

**On push to main with `feat: ` commit:**

```
Step 1: release-please checks commits
  → Detects minor version bump needed
  → Creates Release PR

Step 2: User reviews PR
  → All 3 files updated correctly
  → Merge PR

Step 3: release-please creates release
  → GitHub Release v0.3.0 created
  → CHANGELOG.md updated
  → No build step runs (optional)

Total time: 5-10 minutes (waiting + 1 click)
```

### semantic-release v24

**On push to main with `feat: ` commit:**

```
Step 1: semantic-release runs
  → Parses commits
  → Calculates version
  → Runs prepare script (update files)
  → Commits + tags
  → Creates GitHub release
  → (Auto-publishes immediately)

Total time: 2-5 minutes (automatic)

Risk: If script fails, release is broken
```

### changesets

**On push to main with new changeset file:**

```
Step 1: Changesets detects .changeset/* files
Step 2: Creates PR with version bumps
Step 3: User reviews + merges PR
Step 4: Changesets publishes (requires manual trigger or 2nd workflow)

Total time: 10-15 minutes (waiting + review + manual publish)
```

---

## Recommended Setup Summary

**File structure after setup:**

```
.
├── .github/workflows/
│   └── release.yml                      [CREATE]
├── release-please-config.json           [CREATE]
├── release-please-manifest.json         [CREATE]
├── CHANGELOG.md                         [AUTO-CREATED]
├── package.json
├── src-tauri/
│   ├── tauri.conf.json
│   └── Cargo.toml
├── src-tauri/src/
└── ...
```

**Workflow:**
1. Dev commits with conventional message (`feat:`, `fix:`, etc.)
2. Push to main
3. GitHub Actions: release-please creates Release PR
4. Dev: Review PR (30 seconds) and merge
5. GitHub Actions: Creates GitHub Release + CHANGELOG
6. Done

**No manual version bumping. No manual CHANGELOG. No manual releases.**

---

## Testing Your Setup

Before going live:

```bash
# 1. Verify configuration files are valid
jq . release-please-config.json    # Should not error
jq . release-please-manifest.json  # Should not error

# 2. Verify jsonpath values work
jq '.productVersion' src-tauri/tauri.conf.json
jq '.package.version' src-tauri/Cargo.toml
jq '.version' package.json

# 3. Commit all 3 files
git add release-please-config.json
git add release-please-manifest.json
git add .github/workflows/release.yml
git commit -m "chore: add release-please automation"
git push origin main

# 4. Create test commit
git commit --allow-empty -m "feat: test release automation"
git push origin main

# 5. Check GitHub Actions
# Go to https://github.com/your-user/agent-playground-desktop/actions
# Look for "Release" workflow run
# Should complete successfully and create Release PR

# 6. Review Release PR
# Should show all 3 files updated to next version

# 7. (Optional) Merge to test full flow
# Merging should trigger GitHub Release creation
```

---

## Production Checklist

Before using in production:

- [ ] All 3 config files committed to git
- [ ] GitHub Actions workflow file exists and is valid
- [ ] Verified jsonpath values are correct
- [ ] Tested with dummy commit (feat: test)
- [ ] Release PR appeared with correct version bumps
- [ ] Verified all 3 files updated in PR
- [ ] Merged PR successfully
- [ ] GitHub Release was created automatically
- [ ] CHANGELOG.md has entry for the release
- [ ] All subsequent commits follow conventional commit format

---

## Monitoring & Debugging

**Check workflow runs:**
```
GitHub → Actions → Release workflow
Should show green checkmarks on successful runs
```

**If Release PR doesn't appear:**
1. Check Actions tab for errors
2. Verify commit message is valid (feat:, fix:, etc.)
3. Verify push is to main branch
4. Wait 2-5 minutes (may be rate-limited)

**If files don't update in PR:**
1. Check jsonpath is correct (test with `jq`)
2. Verify file paths exist (no typos)
3. Check file is valid JSON/TOML
4. Delete PR and retry with fixed config

---

## See Also

- [release-please GitHub](https://github.com/googleapis/release-please)
- [Semantic Versioning Spec](https://semver.org/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
