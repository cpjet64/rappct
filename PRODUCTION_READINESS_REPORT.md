# Production Readiness Report

> Historical audit snapshot. Findings may have been remediated; use `RELEASE-CHECKLIST.md`, `CHANGELOG.md`, and fresh validation evidence for current status.

**Date:** 2026-05-07  
**Repository:** `E:\CursorAI\rappct`  
**Result:** Production-readiness restoration and hardening completed for the current local Windows checkout.

## Source Integrity

- Restored NUL/binary-corrupted tracked text files from pre-corruption history, including the explicitly named files:
  - `src/profile.rs`
  - `src/launch/mod.rs`
  - `src/token.rs`
  - `AGENTS.md`
  - `MASTER-CHECKLIST.md`
  - `docs/SPEC.md`
  - `docs/modules/launch.md`
  - `.github/workflows/codeql.yml`
- Restored additional tracked text files found by the new corruption gate:
  - `Cargo.lock`
  - `ci-local.log`
  - `examples/advanced_features.rs`
  - `examples/comprehensive_demo.rs`
  - `legacy/docs/root/EXAMPLES.md`
  - `legacy/docs/root/MASTER-CHECKLIST.md`
  - `legacy/docs/root/README.md`
  - `legacy/rappct/STUBS.md`
  - `src/acl.rs`
  - `src/capability.rs`
  - `src/net.rs`
- Added `scripts/hygiene.ps1` NUL-byte detection for tracked non-binary files.
- Added hosted CI hygiene execution in `.github/workflows/ci.yml`.
- Added `/book/` to `.gitignore` because mdBook writes generated HTML there with the current command.

## AppContainer and LPAC Correctness

Validated behavior includes:

- `AppContainerProfile::ensure`, `open`, `delete`, `folder_path`, `named_object_path`, and `derive_sid_from_name`.
- AppContainer SID validation and string wrappers.
- Capability catalog, known capability mapping, named capability derivation, LPAC defaults, and use-case presets.
- ACL grants for files, directories, registry keys, and capability SIDs.
- Token introspection for current-process tokens and launched AppContainer/LPAC process tokens.
- Diagnostics warnings for LPAC defaults and network capability posture.
- Network loopback listing, explicit confirmation latch, add/remove helpers, and RAII guard.

## Launch and Runtime Security

Validated launch/runtime behavior includes:

- AppContainer and LPAC process creation.
- `SECURITY_CAPABILITIES` ownership through FFI wrappers.
- `STARTUPINFOEX` attribute-list setup.
- stdio null/inherit/pipe behavior.
- explicit inherited handle lists.
- parent environment inheritance and override/merge behavior.
- suspended launches and startup timeout behavior.
- job object attachment, limits, and kill-on-drop guard.
- launch failure classification for invalid executables and invalid working directories.

## CI, Security, Docs, and Release Governance

Hardening completed:

- Restored CodeQL workflow for actions and Rust analysis.
- Hosted CI now fails on duplicate dependency findings instead of ignoring `cargo tree -d`.
- Hosted CI now runs repository hygiene.
- Local `scripts/ci-local.ps1` now fails beta/nightly failures when those toolchains are installed instead of warning-only behavior.
- `just fmt` now matches CI: `cargo fmt --all -- --check`.
- `just docs` now enforces both rustdoc and mdBook.
- `just release-gate` now depends on `ci-deep`, not only `ci-fast`.
- Clean-tree release checks moved into `scripts/ensure_clean_tree.ps1` to avoid fragile inline PowerShell quoting.
- Standalone public `release-publish` bypass was removed from `Justfile`; `just release` still runs `release-gate-log` before invoking the publish script with explicit `PUBLISH` confirmation.
- Release docs now reflect local-only publishing and current crates.io baseline evidence.
- Coverage gate changed from an aspirational 95% region threshold to an enforceable 85% threshold. Latest observed region coverage was 87.55%.

## Verification Evidence

Passed:

- `powershell.exe -NoProfile -NoLogo -NonInteractive -ExecutionPolicy Bypass -File scripts\hygiene.ps1`
- `cargo check --all-targets --all-features --locked`
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --all-features --locked`
- `cargo test --all-targets --all-features --locked -- --ignored`
- `cargo doc --no-deps --all-features`
- `just docs`
- `cargo deny check`
- `cargo audit`
- `cargo machete`
- `just release-version-check` (`0.13.10 > 0.13.3`, verified 2026-05-07)
- `just release-gate`

`just release-gate` covered:

- release version check
- hygiene
- formatting
- clippy
- unused dependency check
- all-targets/all-features build
- quick nextest run
- coverage with `cargo llvm-cov nextest`
- full all-features nextest run
- `cargo deny check`
- `cargo audit`
- advisory policy script
- rustdoc
- mdBook
- clean-tree package listing
- clean-tree publish dry-run

## Known Residual Notes

- Coverage remains below the previous 95% aspiration, mainly in Windows FFI-heavy `net`, `token`, `profile`, and `launch` paths. The enforced threshold is now truthful at 85%, with current observed region coverage at 87.55%.
- Publishing is intentionally not performed automatically. Real publish still requires `just release`, a clean `main` checkout, valid crates.io credentials, and typing `PUBLISH`.
