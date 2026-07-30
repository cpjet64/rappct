# Security and Reliability Review

## Security Posture Summary

The intended project is security-sensitive. It manipulates Windows AppContainer profiles, capability SIDs, process creation attributes, ACLs, token inspection, job objects, and firewall loopback exemptions. The readable design favors RAII ownership and explicit feature gates, but current repository corruption prevents production assurance.

## Security Findings

| ID | Finding | Classification | Evidence | Risk | Required action |
| --- | --- | --- | --- | --- | --- |
| S1 | Core security-boundary modules are corrupted | broken | `src/profile.rs`, `src/launch/mod.rs`, `src/token.rs` | Cannot audit AppContainer identity, process launch, or token claims. | Restore and re-audit before feature work. |
| S2 | CodeQL workflow appears corrupted | broken | `.github/workflows/codeql.yml` read as unrelated XML-like data | Static security analysis may be absent. | Restore valid CodeQL workflow and confirm run. |
| S3 | Loopback exemption mutates host firewall | partial/known | `src/net.rs` docs/tests require `LoopbackAdd(...).confirm_debug_only()` | Misuse can weaken local network isolation. | Keep latch; document production use limits; require opt-in tests before release. |
| S4 | ACL helpers can grant broad permissions | partial/known | docs examples mention `GENERIC_ALL`; ACL module grants file/dir/registry entries. | Misconfiguration can overexpose host resources to AppContainer SID/capability. | Add least-privilege examples and warnings; test invalid/nonexistent targets. |
| S5 | Launch handle inheritance is high risk | broken/unverified | docs claim `InheritList`, copied handle list, stdio modes; implementation corrupted. | Handle leaks can violate sandbox assumptions. | Restore launch, inspect inherited handle list, run explicit tests. |
| S6 | LPAC support override exposed through a Cargo feature | resolved | Removed the feature, public test-support module, and ambient launch/LPAC overrides. | None; native detection now fails closed. | Keep pure unit tests for version evaluation and public-API integration tests. |
| S7 | Deprecated RAII wrappers remain public | partial | `src/util.rs` exports deprecated `OwnedHandle`, `LocalFreeGuard`, `FreeSidGuard`. | Larger public unsafe-adjacent compatibility surface. | Define deprecation timeline; keep tests until removal. |
| S8 | `AcError::Unimplemented` public variant remains | partial/stubbed | `src/error.rs` | Placeholder error may mask incomplete behavior if used. | Search usage and remove or document. |
| S9 | Dependency/advisory checks are local-only | partial | `Justfile` has `cargo deny`, `cargo audit`, policy script; hosted CI omits them. | PRs can merge without security checks if local gate skipped. | Add scheduled/release security workflow or require local transcript. |

## Reliability Findings

| ID | Finding | Classification | Evidence | Risk | Required action |
| --- | --- | --- | --- | --- | --- |
| R1 | Text/source corruption is not prevented | missing | hygiene script does not check NUL bytes | Broken files can enter repo. | Add tracked text integrity check. |
| R2 | Historical green evidence is stale | stale | docs cite 2026-02-25 and 2026-02-26 passes | Current state invalidates prior validation. | Refresh all evidence after restore. |
| R3 | Ignored tests cover important behavior | weak | net/job tests are `#[ignore]` and env-gated | Critical OS-mutating semantics lack regular coverage. | Pre-release manual test transcript or dedicated runner. |
| R4 | Release script depends on network registry check | partial | `scripts/release_version_check.ps1` calls crates.io | Release gate can fail offline; needs documented network prerequisite. | Document and handle network failure explicitly. |
| R5 | `scripts/ci-local.ps1` can skip missing toolchains | partial | script warns and continues | Local matrix completion can be overstated. | Add strict mode for release. |
| R6 | Docs include generated-output links | partial | docs link to `target/doc` and `docs/book` | Links break until generated locally. | Make generated docs instructions clear; avoid treating generated paths as committed evidence. |

## Security Acceptance Criteria

```powershell
cargo clippy --all-targets --all-features -- -D warnings
cargo deny check
cargo audit
python scripts/enforce_advisory_policy.py
```

Manual/elevated security boundary checks before release:

```powershell
cargo test --test windows_launch --all-features -- --nocapture
cargo test --test windows_acl --all-features -- --nocapture
cargo test --test windows_net --features net -- --ignored --nocapture
```
