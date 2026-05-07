# Recommended Completion Plan

This plan is ordered so ChatGPT or another agent can turn it into a production-grade `/goal` prompt. Do not implement feature work before Phase 0 is complete.

## Phase 0: Restore Repository Integrity

Goal: recover from corrupted source/docs and make the crate compile again.

Tasks:

1. Identify all corrupted tracked text files using byte-level NUL detection.
2. Restore `src/profile.rs`, `src/launch/mod.rs`, `src/token.rs`, `AGENTS.md`, `MASTER-CHECKLIST.md`, `docs/SPEC.md`, `docs/modules/launch.md`, `.github/workflows/codeql.yml`, and corrupted legacy docs from a known-good commit or release artifact.
3. Add or extend hygiene automation to fail on NUL bytes in tracked text files.
4. Run the minimal compile gate.

Acceptance criteria:

- No NUL-corrupted tracked source/docs/config files remain.
- `cargo check --all-targets --all-features --locked` passes.
- `scripts/hygiene.ps1` detects future NUL corruption.

Verification commands:

```powershell
rg --files
cargo check --all-targets --all-features --locked
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/hygiene.ps1
```

## Phase 1: Revalidate Core Runtime Behavior

Goal: prove documented core APIs work from restored code.

Tasks:

1. Validate profile lifecycle: `AppContainerProfile::ensure`, `open`, `delete`, `folder_path`, `named_object_path`, `derive_sid_from_name`.
2. Validate capability derivation/catalog/use-case presets.
3. Validate launch with AC and LPAC, stdio modes, env inheritance, cwd errors, nonexistent executable errors, explicit handle list, wait exit code, suspended mode if supported.
4. Validate token introspection and launched child token SID/capabilities.
5. Validate ACL grants for file, directory, custom inheritance, and supported registry roots.

Acceptance criteria:

- All non-ignored Windows core tests pass with all features.
- CLI/example help commands compile and run.
- Any expected privileged/manual behavior is documented separately.

Verification commands:

```powershell
cargo test --all-targets --all-features --locked
cargo run --example acrun -- --help
cargo run --example rappct_demo -- --help
cargo run --example advanced_features -- --help
cargo run --example network_demo --features net -- --help
```

## Phase 2: Reconcile Documentation with Source Truth

Goal: make docs reliable enough for production users and future agents.

Tasks:

1. Rewrite or repair `MASTER-CHECKLIST.md` and `EXECUTION-PLAN.md` with current evidence only.
2. Update `README.md`, `docs/API.md`, `docs/ARCHITECTURE.md`, `docs/modules/*`, and `SECURITY.md` to match restored behavior.
3. Mark legacy docs as archived/stale where they conflict with current workflow.
4. Regenerate rustdoc and mdBook.

Acceptance criteria:

- No doc claims milestone completion without current command evidence.
- No conflicting release instructions remain in active docs.
- Rustdoc and mdBook build successfully.

Verification commands:

```powershell
cargo doc --workspace --all-features --no-deps
mdbook build docs --dest-dir book
rg -n --hidden -S "release workflow|release-plz|GitHub Releases|complete|100%|0.13.10|0.13.4" README.md docs *.md legacy
```

## Phase 3: Strengthen CI and Test Gates

Goal: align automation with the production bar.

Tasks:

1. Decide whether duplicate dependencies should fail CI; if yes, remove `|| true` from `.github/workflows/ci.yml`.
2. Restore `.github/workflows/codeql.yml`.
3. Add a strict mode or release mode to `scripts/ci-local.ps1` that fails when required toolchains are missing.
4. Add docs/security/coverage gates to release or scheduled CI, or document them as local release-only gates.
5. Add explicit pre-release manual checklist for ignored privileged net/job tests.

Acceptance criteria:

- Hosted CI parses and runs.
- Local strict matrix cannot silently skip required release toolchains.
- Release gate includes security and packaging checks.

Verification commands:

```powershell
just ci-fast
just ci-deep
scripts/ci-local.ps1
cargo tree -d
```

## Phase 4: Security and Reliability Hardening

Goal: close high-risk security and reliability gaps after compile/test restoration.

Tasks:

1. Re-audit launch handle inheritance and `SECURITY_CAPABILITIES` pointer lifetimes.
2. Re-audit ACL grant defaults and examples for least privilege.
3. Confirm loopback exemption latch cannot be bypassed accidentally.
4. Search and resolve any real uses of `AcError::Unimplemented`.
5. Review public deprecated `util` wrappers and document removal policy.
6. Document tracing/logging events and expected observability integration.

Acceptance criteria:

- No unexplained `unsafe` blocks.
- No placeholder or fake implementations.
- Security-sensitive APIs include explicit privilege and mutation warnings.
- All security gates pass.

Verification commands:

```powershell
rg -n --hidden -S "Unimplemented|TODO|FIXME|XXX|HACK|unsafe" src tests examples docs
cargo clippy --all-targets --all-features -- -D warnings
cargo deny check
cargo audit
python scripts/enforce_advisory_policy.py
```

## Phase 5: Release Readiness

Goal: prepare a clean, evidence-backed release candidate.

Tasks:

1. Refresh crates.io version baseline.
2. Align `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, `RELEASE-CHECKLIST.md`, and docs release references.
3. Confirm package include allow-list.
4. Run clean-tree package and dry-run gates.
5. Run logged release gate.
6. Publish only with explicit user approval.

Acceptance criteria:

- `just release-gate-log` passes on clean tree.
- Transcript path is recorded in `RELEASE-CHECKLIST.md`.
- Changelog accurately describes the release candidate.
- No stale GitHub-publish workflow instructions remain active.

Verification commands:

```powershell
just release-version-check
just package-list-clean
just publish-dry-run-clean
just release-gate-log
```

## Completion Definition

The project is production-ready only when:

- Corrupted files are restored and protected by hygiene checks.
- Full local gates pass from a clean tree.
- Hosted CI and CodeQL are valid and green.
- Docs match source behavior.
- Release gate passes with current crates.io baseline.
- Privileged Windows behavior has explicit manual or automated evidence.

