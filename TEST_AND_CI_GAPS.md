# Test and CI Gaps

> Historical audit snapshot. Findings may have been remediated; use `RELEASE-CHECKLIST.md`, `CHANGELOG.md`, and fresh validation evidence for current status.

## Current Test Coverage Shape

Readable test files show strong intended Windows integration coverage:

| Test area | Files | Status | Notes |
| --- | --- | --- | --- |
| Core SID/capability/token | `tests/windows_core.rs` | partial | Token assertions depend on corrupted `src/token.rs`. |
| Launch | `tests/windows_launch.rs` | partial | Broad tests exist for failures, stdio, LPAC, token match, waits, handles, diagnostics; implementation corrupted. |
| ACL | `tests/windows_acl.rs` | partial | Prior read found ACL coverage; must re-run after restore. |
| Profile | `tests/windows_profile.rs` | partial | Profile tests likely exist, but implementation corrupted. |
| Network | `tests/windows_net.rs`, `tests/windows_net_loopback_guard.rs` | weak/partial | Mutating roundtrips are ignored and require env/elevation. |
| Job guard | `tests/windows_job_guard.rs` | weak | Ignored and env-gated. |
| API surface | `tests/api_surface.rs` | partial | Compile-time surface test only; blocked by corrupted modules. |
| Smoke/capability | `tests/cap_smoke.rs` | partial | Prior read indicated capability smoke coverage. |

## CI Gaps

| Gap | File | Classification | Risk | Fix |
| --- | --- | --- | --- | --- |
| CI cannot be trusted until source is restored | `src/profile.rs`, `src/launch/mod.rs`, `src/token.rs` | broken | All tests are blocked by repository integrity. | Restore source and run gates. |
| Duplicate dependency check is non-blocking | `.github/workflows/ci.yml` | partial | `cargo tree -d` failures are ignored. | Remove `|| true` if duplicates should block. |
| Hosted CI omits deep local gates | `.github/workflows/ci.yml`, `Justfile` | partial | No hosted `cargo audit`, `cargo deny`, advisory policy, docs, coverage in main CI. | Add release/scheduled workflow or document as local-only release gate. |
| Local matrix may skip MSRV toolchains | `scripts/ci-local.ps1` | partial | Missing installed toolchains can still produce final `OK`. | Fail when release mode requires full matrix. |
| Beta/nightly are non-blocking | `.github/workflows/ci.yml`, `scripts/ci-local.ps1` | intentional/partial | Future Rust breakage may be ignored. | Document advisory status or promote to blocking later. |
| Mutating Windows behavior lacks default CI proof | `tests/windows_net*.rs`, `tests/windows_job_guard.rs` | weak | Loopback and job guard regressions can ship without automated signal. | Add controlled opt-in pre-release transcript requirement. |
| CodeQL workflow corrupted | `.github/workflows/codeql.yml` | broken | Static analysis may not run. | Restore and verify GitHub Actions. |
| mdBook docs build not gated | `docs/PROGRESS.md`, `docs/TOOLING.md` | partial | Docs site can rot. | Add `mdbook build docs --dest-dir book` to docs gate when mdBook is installed. |

## Required Verification Commands

Minimal after source restoration:

```powershell
cargo check --all-targets --all-features --locked
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features --locked
```

Full local:

```powershell
just ci-fast
just ci-deep
scripts/ci-local.ps1
```

Privileged/manual pre-release:

```powershell
$env:RAPPCT_ALLOW_NET_TESTS='1'
cargo test --test windows_net --features net -- --ignored --nocapture
cargo test --test windows_net_loopback_guard --features net -- --ignored --nocapture
$env:RAPPCT_ALLOW_JOB_TESTS='1'
cargo test --test windows_job_guard --all-features -- --ignored --nocapture
```

Docs and release:

```powershell
cargo doc --workspace --all-features --no-deps
mdbook build docs --dest-dir book
just release-gate-log
```
