# Dual-Branch Development Workflow

This document describes rappct's automated dual-branch workflow optimized for AI-assisted development and stable releases.

## Branch Strategy

### `dev` Branch (Development / Pre-releases)
- **Purpose**: Active development, AI agent experimentation, rapid iteration
- **Stability**: May contain breaking changes, experimental features
- **Releases**: Automated pre-releases to crates.io (e.g., `v0.11.2-dev.1`)
- **Publishing**: Automatic with `-dev` suffix
- **Protection**: None - direct pushes allowed for rapid development

### `main` Branch (Stable / Production)
- **Purpose**: Production-ready, stable releases only
- **Stability**: Guaranteed stable, follows semantic versioning strictly
- **Releases**: Stable releases to crates.io (e.g., `v0.11.2`)
- **Publishing**: Automatic stable releases only
- **Protection**: Requires PR review, CI must pass

---

## Complete Automated Workflows

### Dev Branch Workflow (Rapid Iteration)

```mermaid
graph LR
    A[Push to dev] --> B[CI Runs]
    B --> C{CI Pass?}
    C -->|No| D[Fix Issues]
    D --> A
    C -->|Yes| E[release-please-dev]
    E --> F[Creates PR with version bump]
    F --> G[Merge PR]
    G --> H[GitHub Pre-Release Created]
    H --> I[Auto-Publish to crates.io]
    I --> J[v0.x.x-dev.N on crates.io]
```

**Steps**:
1. **Work on dev branch**:
   ```bash
   git checkout dev
   # Make changes
   git commit -m "feat: add awesome feature"
   git push origin dev
   ```

2. **CI runs automatically** (4 matrix jobs):
   - Tests: `[]`, `[introspection]`, `[net]`, `[introspection,net]`
   - Clippy with `-D warnings`

3. **If CI passes**, `release-please-dev` creates/updates a PR:
   - Version: `0.11.2-dev.1` (pre-release)
   - Changelog: `CHANGELOG-DEV.md`

4. **Merge the release PR**:
   ```bash
   # Review and merge via GitHub UI
   ```

5. **Automatic publishing**:
   - GitHub pre-release created (marked as pre-release)
   - Published to crates.io with `-dev` suffix
   - Example: `rappct = "0.11.2-dev.1"`

### Main Branch Workflow (Stable Releases)

```mermaid
graph LR
    A[Merge dev → main] --> B[CI Runs]
    B --> C{CI Pass?}
    C -->|No| D[Fix on dev first]
    D --> A
    C -->|Yes| E[release-please]
    E --> F[Creates PR with version bump]
    F --> G[Review & Merge PR]
    G --> H[GitHub Release Created]
    H --> I[Auto-Publish to crates.io]
    I --> J[v0.x.x on crates.io]
```

**Steps**:
1. **Create PR from dev → main**:
   ```bash
   # On GitHub: Create Pull Request
   # Base: main ← Compare: dev
   ```

2. **CI runs on PR** (all feature combinations)

3. **Merge PR when CI passes**

4. **If CI passes**, `release-please` creates/updates a PR:
   - Version: `0.12.0` (stable, no suffix)
   - Changelog: `CHANGELOG.md`

5. **Merge the release PR**:
   ```bash
   # Review and merge via GitHub UI
   ```

6. **Automatic publishing**:
   - GitHub stable release created
   - Published to crates.io as stable version
   - Example: `rappct = "0.12.0"`

7. **Sync main → dev**:
   ```bash
   git checkout dev
   git merge main
   git push origin dev
   ```

---

## Conventional Commits

Both workflows use [Conventional Commits](https://www.conventionalcommits.org/) to determine version bumps:

| Commit Prefix | Version Bump | Example |
|---------------|--------------|---------|
| `feat:` | Minor (0.x.0) | `feat: add new capability` |
| `fix:` | Patch (0.0.x) | `fix: resolve token parsing` |
| `feat!:` or `BREAKING CHANGE:` | Major (x.0.0) | `feat!: remove deprecated API` |
| `chore:`, `docs:`, `style:` | No bump | `docs: update README` |

**Examples**:
```bash
# Minor version bump (0.11.x → 0.12.0)
git commit -m "feat: add async support for profile creation"

# Patch version bump (0.11.1 → 0.11.2)
git commit -m "fix: handle edge case in capability derivation"

# Major version bump (0.11.x → 1.0.0) - use sparingly
git commit -m "feat!: redesign launch API

BREAKING CHANGE: launch_in_container now requires explicit stdio config"

# No version bump
git commit -m "docs: improve examples documentation"
```

---

## Version Numbering

### Dev Branch
- Format: `MAJOR.MINOR.PATCH-dev.N`
- Example: `0.11.2-dev.1`, `0.11.2-dev.2`
- Each merge to dev increments the dev counter
- Pre-release flag ensures it's not installed by default

### Main Branch
- Format: `MAJOR.MINOR.PATCH`
- Example: `0.11.2`, `0.12.0`, `1.0.0`
- Follows semantic versioning strictly
- Stable, production-ready

---

## CI Configuration

### Triggers
```yaml
on:
  push:
    branches: [ main, dev, feat/**, fix/** ]
  pull_request:
    branches: [ main, dev ]
```

### Matrix Testing
- **OS**: windows-latest only (Windows-specific crate)
- **Features**: `[]`, `[introspection]`, `[net]`, `[introspection,net]`
- **Lint**: Clippy with `-D warnings` (warnings = errors)

---

## Publishing to crates.io

### Dev Pre-releases
```toml
[dependencies]
# Latest dev pre-release
rappct = "0.11.2-dev.1"

# Or use version requirement to allow any 0.11.x-dev
rappct = ">=0.11.0-dev, <0.12.0"
```

**Users opt-in** to dev releases explicitly. `cargo add rappct` will NOT install dev versions.

### Stable Releases
```toml
[dependencies]
# Latest stable release (default)
rappct = "0.11"

# Or specific version
rappct = "0.11.2"
```

**Default behavior** - users get stable releases unless they explicitly request dev.

---

## AI Coding Agent Best Practices

### When Working on Dev
1. ✅ **Commit frequently** - each feature gets its own commit
2. ✅ **Use conventional commits** - enables automatic versioning
3. ✅ **Push to dev directly** - no PR needed for rapid iteration
4. ✅ **Merge release PRs promptly** - keep dev releases flowing
5. ✅ **Test with dev versions** - dogfood your own pre-releases

### When Stabilizing to Main
1. ✅ **Create PR dev → main** - requires human review
2. ✅ **Wait for CI** - must pass all feature combinations
3. ✅ **Review changelog** - ensure release notes are clear
4. ✅ **Merge release PR** - triggers stable publish
5. ✅ **Sync main → dev** - keep branches in sync

### Avoiding Common Pitfalls
- ❌ **Don't commit directly to main** - always go through dev first
- ❌ **Don't skip CI** - automated checks catch issues early
- ❌ **Don't ignore failing tests** - fix them before releasing
- ❌ **Don't forget to sync main → dev** - prevents divergence

---

## Branch Protection Rules (Recommended)

### Main Branch
Go to `https://github.com/cpjet64/rappct/settings/branches` and configure:

- ✅ **Require pull request before merging**
  - Require approvals: 1
  - Dismiss stale reviews when new commits are pushed

- ✅ **Require status checks to pass before merging**
  - Require branches to be up to date
  - Status checks required: `test-windows`

- ✅ **Require conversation resolution before merging**

- ✅ **Do not allow bypassing the above settings**

### Dev Branch
- ❌ **No protection** - allows direct pushes for rapid development
- ⚠️ **CI still runs** - automated quality checks

---

## Example Complete Workflow

### Scenario: Add New Feature via AI Agent

**Day 1-3: Development**
```bash
# Agent works on dev branch
git checkout dev

# Agent implements feature
git commit -m "feat: add capability suggestion fuzzy matching"
git push origin dev

# CI passes → release-please-dev creates PR
# Merge PR → v0.11.2-dev.1 published

# Continue iterating
git commit -m "fix: improve suggestion threshold"
git push origin dev

# CI passes → release-please-dev updates PR
# Merge PR → v0.11.2-dev.2 published
```

**Day 4: Stabilization**
```bash
# Human reviews dev work, creates PR dev → main
# CI runs on PR
# Human merges PR

# release-please creates stable release PR
# Human reviews changelog, merges PR
# v0.12.0 published to crates.io (stable)

# Sync main back to dev
git checkout dev
git merge main
git push origin dev
```

---

## Troubleshooting

### Release PR Not Created
- ✅ Check CI passed: https://github.com/cpjet64/rappct/actions
- ✅ Verify conventional commits used (must have `feat:` or `fix:`)
- ✅ Check for existing release PR (may be updating existing)

### CI Failing
- ✅ Run locally: `cargo test --all-targets --all-features`
- ✅ Check Clippy: `cargo clippy --all-targets --all-features -- -D warnings`
- ✅ Fix warnings - CI treats them as errors

### Publish Failing
- ✅ Verify `CARGO_REGISTRY_TOKEN` is set in GitHub Secrets
- ✅ Check crates.io for version conflicts
- ✅ Ensure Cargo.toml metadata is complete

### Branches Diverged
```bash
# Sync main → dev
git checkout dev
git merge main
git push origin dev
```

---

## References

- [Conventional Commits](https://www.conventionalcommits.org/)
- [Semantic Versioning](https://semver.org/)
- [release-please](https://github.com/googleapis/release-please)
- [GitHub Actions Workflow Syntax](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions)
