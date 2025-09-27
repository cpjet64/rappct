# TODO (WSL) - Work Items for Linux Subsystem

## Tests & Code Quality
- [x] Add targeted tests for `AcError` variants to validate context and source wiring (src/error.rs:6, tests/windows_core.rs:22) - new unit cases live in `src/error.rs`.
- [x] Audit feature gating to ensure optional modules (`net`, `introspection`, `tracing`) only compile when requested and add cfg-driven smoke coverage where missing (Cargo.toml:13, src/lib.rs:12) - `cargo check` passes for all feature combos on WSL; warnings remain for unused placeholders.
- [x] Add unit coverage for capability name suggestions to verify strsim thresholds (src/capability.rs:61) - see new tests in `src/capability.rs`.
- [x] Verify `with_lpac_defaults()` yields the documented capability set by strengthening unit tests/examples (src/capability.rs:112) - covered by new builder tests in `src/capability.rs`.

## Documentation & Planning
- [x] Write README capability cheat sheet and migration guidance (README.md:60) - see new README section.
- [x] Document the `whoami --json` output contract now that token fields are populated (examples/acrun.rs:33) - documented in README.
- [x] Plan/record CI matrix coverage for Windows Server 2019/2022 and Windows 10/11 (TODO.md:33) - matrix captured in `TODO.md` v0.4 section.
- [x] Document intentional net/introspection feature gating so consumers know which APIs return `Unimplemented` (src/net.rs:42, src/diag.rs:1) - see "Feature gating and platform behaviour" section in `README.md`.
- [x] Ensure release artifacts/documentation mention required Windows SDK/CRT dependencies and installation steps (README.md:20) - documented in README "Toolchain prerequisites".

## Repository Hygiene
- [x] Remove stray numbered source snapshots before publishing (src/launch/mod.rs.num:1, src/launch/attr.rs.num:1).
- [x] Drop unused dependencies/features (`once_cell`, `async`, `nt`) or implement the promised functionality (Cargo.toml:16, Cargo.toml:17, Cargo.toml:22) - removed unused flags and dependency.
