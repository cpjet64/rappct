# Production Readiness Gaps

> Historical audit snapshot. Findings may have been remediated; use `RELEASE-CHECKLIST.md`, `CHANGELOG.md`, and fresh validation evidence for current status.

| ID | Gap | Classification | Evidence | Production impact | Acceptance criteria |
| --- | --- | --- | --- | --- | --- |
| P1 | Core source files are corrupted | broken | `src/profile.rs`, `src/launch/mod.rs`, `src/token.rs` appear zeroed/corrupted. | Crate cannot be trusted, likely cannot compile. | Restore and pass full gates. |
| P2 | Text corruption is not caught by hygiene | missing | `scripts/hygiene.ps1` checks large files, conflict markers, `.gitignore`, but not NUL bytes or file encoding corruption. | Corrupted docs/source can survive local checks until compiler/test failure. | Add NUL/binary-content checks for tracked text files. |
| P3 | Docs claim production completion despite current broken state | stale | `legacy/docs/root/EXECUTION-PLAN.md` and readable docs claim milestone completion and green matrix. | Future agents may build on false assumptions. | Refresh docs after restoration and include current evidence dates. |
| P4 | Release checklist is incomplete | partial | `RELEASE-CHECKLIST.md` has unchecked clean-tree release gates and final publish approval. | Cannot publish safely. | Complete `just release-gate-log` on clean tree before any publish. |
| P5 | Registry/version evidence is stale | stale | crates.io latest baseline is documented as `0.13.3` on 2026-03-04. | Version publish check may be wrong. | Re-run `just release-version-check` with network access. |
| P6 | CodeQL workflow appears corrupted | broken | `.github/workflows/codeql.yml` read contained unrelated XML-like content. | Security scanning may not run. | Restore valid workflow and verify GitHub parses it. |
| P7 | Hosted CI allows duplicate dependencies | partial | `.github/workflows/ci.yml` runs `cargo tree -d || true`. | Dependency bloat/conflicts may be ignored. | Decide policy; remove `|| true` if duplicates should fail. |
| P8 | Local matrix can skip missing toolchains | partial | `scripts/ci-local.ps1` warns and skips MSRV/beta/nightly if toolchains missing. | A local "OK" may not mean full matrix ran. | Emit summary of skipped toolchains and fail release gate if required toolchains absent. |
| P9 | Beta/nightly failures are warnings locally and continue-on-error hosted | partial | `scripts/ci-local.ps1` warns; hosted CI `continue-on-error` for beta/nightly. | Future compatibility drift is informational only. | Keep as advisory or document as non-blocking. |
| P10 | Privileged/mutating tests are ignored | partial | `tests/windows_net*.rs`, `tests/windows_job_guard.rs` require env/elevation. | Firewall/job semantics are not continuously protected. | Add documented manual pre-release checklist with transcript or controlled CI host. |
| P11 | mdBook build was never proven in docs generation pass | partial | `docs/PROGRESS.md` says `mdbook` was not on PATH. | Documentation site may not build. | Install mdBook and run `mdbook build docs --dest-dir book`. |
| P12 | Legacy docs contain stale workflows and examples | stale | `legacy/docs/root/WORKFLOW.md` describes release workflow that current docs say was removed. | Operators may follow wrong process. | Add top-level archival notice or remove from production docs package. |
| P13 | `AcError::Unimplemented` remains in public error enum | partial/stubbed | `src/error.rs` includes `Unimplemented(&'static str)`. | Users may see placeholder semantics if used; public API suggests stubs may exist. | Search usage; remove if unused or document intended compatibility. |
| P14 | Deprecated `util` RAII wrappers remain exported | partial | `src/util.rs` exports deprecated Windows wrappers. | Compatibility burden and unsafe surface remain. | Decide deprecation/removal timeline and test migration path. |
| P15 | Logging/tracing behavior is underdocumented | partial | Feature matrix lists `tracing`, but audit did not find detailed observability contract. | Production users lack operational guidance. | Document trace targets/events and add example subscriber usage. |
| P16 | Performance evidence is historical | stale | `docs/optimization-report.md` dated 2026-02-26. | Current performance after restore may differ. | Re-run targeted benchmark/probe if performance claims remain in docs. |

## Minimum Production Bar

Before any feature work or release, the project should satisfy:

```powershell
cargo check --all-targets --all-features --locked
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features --locked
just ci-fast
just ci-deep
scripts/ci-local.ps1
just release-gate-log
```
