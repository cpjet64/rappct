# Architectural Blockers

> Historical audit snapshot. Findings may have been remediated; use `RELEASE-CHECKLIST.md`, `CHANGELOG.md`, and fresh validation evidence for current status.

## B1. Source Integrity Is Compromised

Classification: broken  
Files: `src/profile.rs`, `src/launch/mod.rs`, likely `src/token.rs`

The current checkout contains NUL/corrupted content in core modules. These files are architectural load-bearing points:

- `src/profile.rs`: profile creation/open/delete and SID derivation.
- `src/launch/mod.rs`: process creation, security capabilities, stdio, jobs, handle inheritance, environment inheritance.
- `src/token.rs`: token introspection and capability SID reporting.

Acceptance criteria:

- Restore files from known-good commit or release artifact.
- `cargo check --all-targets --all-features --locked` passes.
- `cargo test --all-targets --all-features --locked` passes on Windows.
- Add a repository hygiene check that fails on NUL bytes in text source/docs files.

Verification commands:

```powershell
rg -n --hidden -S "`0" src docs tests examples scripts .github
cargo check --all-targets --all-features --locked
cargo test --all-targets --all-features --locked
```

## B2. Canonical Planning Artifacts Are Corrupted

Classification: broken/stale  
Files: `AGENTS.md`, `MASTER-CHECKLIST.md`, `docs/SPEC.md`, `legacy/rappct/STUBS.md`

Planning docs that should establish project truth are corrupted. Other docs and legacy execution plans claim milestones are complete, but those claims cannot be trusted against broken source.

Acceptance criteria:

- Reconstruct a single current plan from restored source, readable docs, and current test evidence.
- Mark legacy docs as archived/stale where applicable.
- Ensure `README.md`, `docs/API.md`, `docs/ARCHITECTURE.md`, `MASTER-CHECKLIST.md`, `EXECUTION-PLAN.md`, and `RELEASE-CHECKLIST.md` agree.

Verification commands:

```powershell
rg -n --hidden -S "DEPRECATED|Milestone|complete|release|publish|AppContainerProfile::open|UseCase" *.md docs legacy
cargo doc --workspace --all-features --no-deps
```

## B3. Launch Path Cannot Be Audited

Classification: broken  
Files: `src/launch/mod.rs`, `src/launch/env.rs`, `tests/windows_launch.rs`

Launch is the riskiest runtime boundary because it combines `STARTUPINFOEX`, raw handles, `SECURITY_CAPABILITIES`, stdio inheritance, jobs, and LPAC policy. The main launch file is corrupted, so lifetime and security invariants cannot be verified.

Acceptance criteria:

- Restore launch implementation.
- Confirm `OwnedSecurityCapabilities` lifetime is retained until `CreateProcessW`.
- Confirm inherited handles are duplicated and closed correctly.
- Confirm `LaunchOptions::env` default inheritance behavior matches docs.
- Confirm `suspended`, `startup_timeout`, `join_job`, and stdio modes are tested.

Verification commands:

```powershell
cargo test --test windows_launch --all-features -- --nocapture
cargo clippy --all-targets --all-features -- -D warnings
```

## B4. Release Governance Has Conflicting Sources of Truth

Classification: stale/partial  
Files: `RELEASE-CHECKLIST.md`, `docs/TOOLING.md`, `legacy/docs/root/WORKFLOW.md`, `Justfile`, `scripts/release*.ps1`

Current root docs describe local-only publishing. Legacy docs describe GitHub release workflow publishing. Release checklist has incomplete clean-tree gates and stale registry evidence.

Acceptance criteria:

- Decide and document one release path.
- Remove or clearly archive stale GitHub-publish instructions.
- Refresh version baseline against crates.io.
- Run clean-tree release gate and store transcript.

Verification commands:

```powershell
just release-version-check
just package-list-clean
just publish-dry-run-clean
just release-gate-log
```

## B5. Hosted CI Does Not Cover Claimed Local Quality Bar

Classification: partial  
Files: `.github/workflows/ci.yml`, `Justfile`, `scripts/ci.ps1`, `scripts/ci-local.ps1`

Hosted CI covers matrix tests and clippy, but local `ci-deep` includes coverage, `cargo deny`, `cargo audit`, advisory policy, docs, and other checks not represented as hard hosted gates. Duplicate dependency check is explicitly non-blocking with `|| true`.

Acceptance criteria:

- Define required PR gates separately from release gates.
- Make duplicate dependency failures blocking if that is an intended quality bar.
- Add release/security/docs/coverage checks to a scheduled or release workflow, or explicitly document them as local-only.

Verification commands:

```powershell
just ci-fast
just ci-deep
scripts/ci-local.ps1
```
