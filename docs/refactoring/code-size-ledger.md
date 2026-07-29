# Code Size Refactor Ledger

Prompt: Codebase 500/75 Refactor & Test Organization Loop

## Baseline

- Baseline scan command: `python scripts/check_code_size.py`
- Maintained files scanned: 67
- File violations: 6
- Function violations: 18
- Production violations: 7
- Test violations: 4
- Script/tooling/config violations: 4
- Example violations: 9
- Final maintained files scanned: 89 of 183 discovered files
- Final justified generated/vendor/data/doc exceptions: 94

## Violation Resolution

| Path | Category | Baseline lines | Oversized symbols | Root cause | Resolution | Status | Validation evidence |
|---|---:|---:|---|---|---|---|---|
| `src/launch/mod.rs` | production-source | 1250 | `setup_stdio` 487-710, `launch_impl` 722-964 | Launch options, Win32 attribute setup, stdio plumbing, job object setup, process creation, startup waiting, and unit tests shared one module. | Split launch internals into `attributes`, `job`, `startup`, `stdio`, `spawn`, and private child tests while preserving the public launch API. | done | `cargo test launch --all-features`; final `scripts/ci-local.ps1`; final size scan 0/0 |
| `src/capability.rs` | production-source | 922 | `derive_single_capability_sids` 347-486 | Capability names, catalog lookup, SID derivation FFI, builder presets, and unit tests shared one file. | Split builder and derivation logic into `src/capability/builder.rs` and `src/capability/derive.rs` with re-exports intact. | done | `cargo test capability --all-features`; final `scripts/ci-local.ps1`; final size scan 0/0 |
| `src/acl.rs` | production-source | 572 | `grant_sid_access` 102-389 | Filesystem and registry target validation, ACE construction, and DACL mutation were combined. | Split grant implementation into `src/acl/grant.rs` and private tests into `src/acl/tests.rs`. | done | `cargo test acl --all-features`; elevated `tests/windows_acl.rs`; final `scripts/ci-local.ps1`; final size scan 0/0 |
| `src/net.rs` | production-source | 430 | `list_appcontainers` 53-160, `set_loopback` 320-402 | Firewall enumeration, SID conversion, config reconciliation, and mutation lived in two FFI-heavy functions. | Moved Windows firewall enumeration and update helpers into `src/net/windows_impl.rs`. | done | `cargo test --test windows_net --features net`; final `scripts/ci-local.ps1`; final size scan 0/0 |
| `examples/advanced_features.rs` | example-source | 715 | `demo_advanced_launch` 409-507 | One example binary contained all demos, cleanup guards, scratch helpers, and feature-gated launch/network demos. | Split example demos into `examples/advanced_features/{diagnostics,launching,network_acl}.rs`. | done | `cargo check --example advanced_features --all-features`; final `scripts/ci-local.ps1`; final size scan 0/0 |
| `examples/comprehensive_demo.rs` | example-source | 801 | `demo_network_capabilities` 194-305, `demo_file_acls` 309-436, `demo_comprehensive` 595-735 | Tutorial demo combined unrelated network, ACL, scraper, and comprehensive workflows. | Split example workflows into `examples/comprehensive_demo/{network,file_acls,web_scraper}.rs`. | done | `cargo check --example comprehensive_demo --all-features`; final `scripts/ci-local.ps1`; final size scan 0/0 |
| `examples/rappct_demo.rs` | example-source | 375 | `main` 47-329 | One tutorial main function performed setup, launch demos, localhost checks, internet checks, cleanup, and summary output. | Extracted domain-named demo steps and preserved observable cleanup ordering. | done | `cargo check --example rappct_demo --all-features`; final `scripts/ci-local.ps1`; final size scan 0/0 |
| `tests/windows_launch.rs` | test-source | 700 | `launch_appcontainer_token_matches_profile` 126-209, `launch_lpac_token_sets_flag_and_caps` 213-324, `launch_job_limits_reported_by_query` 352-449 | One integration crate mixed launch, token, job, stdio, diagnostics, and shared helpers. | Split same-crate integration modules into `basic`, `token`, `job`, `stdio`, and `diagnostics`; kept test helpers test-only. | done | `cargo test --test windows_launch --all-features`; final all-features test inventory 135 tests; final `scripts/ci-local.ps1`; final size scan 0/0 |
| `scripts/ci-local.ps1` | script-tooling | 148 | `<top-level>` 1-148 | Full CI matrix orchestration was implemented directly at top level. | Extracted host checks, scratch handling, feature matrix, MSRV matrix, advisory toolchain checks, and size enforcement. | done | PowerShell parse check; final `scripts/ci-local.ps1`; final size scan 0/0 |
| `scripts/ci-gitlab-windows.ps1` | script-tooling | 112 | `<top-level>` 1-112 | GitLab Windows job orchestration was implemented directly at top level. | Extracted temp validation, version reporting, feature selection, and validation sequence functions; added size check to the stable no-feature lane. | done | PowerShell parse check; final `scripts/ci-local.ps1`; final size scan 0/0 |
| `scripts/hygiene.ps1` | script-tooling | 99 | `<top-level>` 1-99 | Hygiene checks ran as one top-level scan block. | Extracted binary, conflict-marker, large-file, and required-file checks. | done | PowerShell parse check; final `scripts/ci-local.ps1`; final size scan 0/0 |
| `scripts/release_version_check.ps1` | script-tooling | 224 | `<top-level>` 1-224 | Release version fetch, parse, compare, and report logic shared one top-level body. | Extracted local version, crates.io version, git tag, comparison, and reporting steps. | done | PowerShell parse check; final `scripts/ci-local.ps1`; final size scan 0/0 |

## Exceptions

| Path | Category | Reason | Status |
|---|---:|---|---|
| `Cargo.lock` | lockfile | Cargo-maintained lockfile; inspected but not split for line count. | accepted |
| `target/**`, `.tmp/**`, `.cache/**`, `.agent-logs/**`, `.worktrees/**` | build-artifact/cache/temp | Ignored generated or transient state; excluded from maintained-source scanning. | accepted |
| `docs/**`, `README.md`, `SECURITY.md`, `RELEASE-CHECKLIST.md` | documentation | Inspected for workflow contradictions, but not executable source subject to mechanical line splitting. | accepted |
| `legacy/**` | legacy-source | Reference legacy code, not the maintained production surface for this refactor loop. | accepted |
| fixtures, snapshots, generated data, migrations | data/generated | No actionable hand-authored executable violations found; data-like files are not split solely for line count. | accepted |

## Final Enforcement

- Size checker: `scripts/check_code_size.py`
- Exact command: `python scripts/check_code_size.py`
- Hook/task runner/CI wiring: `.githooks/pre-commit`, `scripts/ci-local.ps1`, `scripts/ci-gitlab-windows.ps1`, `Justfile`
- Agent guidance: root `AGENTS.md` updated with concise 500/75 rules, test organization policy, exact size command, and validation commands.
- Supporting docs corrected: `docs/TOOLING.md`, `docs/index.md`, `docs/overview.md`, `docs/SPEC.md`

## Final Results

- First clean scan: `python scripts/check_code_size.py` -> 89 maintained files of 183 discovered, 0 file violations, 0 symbol violations
- Second clean scan: `python scripts/check_code_size.py` -> 89 maintained files of 183 discovered, 0 file violations, 0 symbol violations
- Final post-enforcement scan: `python scripts/check_code_size.py` -> 89 maintained files of 183 discovered, 0 file violations, 0 symbol violations
- Current modernization scan: `python scripts/check_code_size.py` -> 91 maintained files of 185 discovered, 0 file violations, 0 symbol violations
- Fast local CI after CI portability updates: `just ci-fast` -> passed; 127 nextest tests passed with 1 skipped, then 135 coverage nextest tests passed with 3 skipped.
- GitLab Windows helper smoke: `powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File scripts\ci-gitlab-windows.ps1 -RustToolchain stable -FeatureSet none` -> passed outside the Codex sandbox.
- Commits/pushes: modernization branch `codex/modernization-sweep-20260729` pushed to GitLab merge request `!2`.
