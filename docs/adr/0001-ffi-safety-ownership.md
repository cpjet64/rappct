# ADR 0001: FFI Safety & Ownership Boundaries

Status: Accepted

Date: 2025-10-22

## Context

The crate interacts with Windows APIs via the `windows` crate and retains several `unsafe` call sites (e.g., handle management, SID memory, COM-related pointers). To strengthen correctness and security, we will formalize ownership and lifetime boundaries across FFI, reduce `unsafe` surface area, and document invariants near platform code.

## Goals

- Define clear ownership rules for handles, SIDs, PWSTR/PCWSTR buffers, and COM-related pointers.
- Centralize common FFI helpers/guards to reduce duplication (e.g., RAII for handles, SID, CoTaskMem allocations).
- Annotate and justify remaining `unsafe` blocks with invariants and preconditions.
- Add targeted tests (unit + integration smoke) where practical to prevent regressions.

## Non-Goals

- No public API breaking changes unless strictly necessary and clearly documented with tests/migration notes.
- No expansion of system privileges or relaxation of sandbox policies.

## Proposed Approach

1. Inventory unsafe/FFI sites in:
   - `src/util.rs`, `src/profile.rs`, `src/launch/mod.rs`, `src/token.rs`, `src/acl.rs`, `src/capability.rs`, `src/net.rs` (feature-gated).
2. Extract/standardize guards and helpers:
   - Handle RAII (CloseHandle), CoTaskMem RAII, FreeSid RAII.
   - Safe conversions for UTF-16 (OsStr/OsString) and `PCWSTR`/`PWSTR` with lifetime notes.
3. Document invariants inline near `unsafe` blocks and in module-level docs.
4. Add tests:
   - Unit tests for guard drop semantics (no double free; handle closed once).
   - Integration smoke tests under Windows only; feature-gate for `net`.
5. Keep patches small and reversible; land incrementally behind internal helpers without changing call sites until validated.

## Checklist

- [x] Introduce crate-private FFI RAII wrappers in `src/ffi/` (handles, mem: LocalAlloc/CoTaskMem, sid: OwnedSid, wstr, sec_caps, attr_list).
- [x] Replace ad-hoc frees/closes with wrappers across `launch/`, `profile.rs`, `acl.rs`, `net.rs`, and `capability.rs`.
- [x] Add unit tests for guard behaviors and conversions (e.g., SID string round-trips, handle close-on-drop).
- [x] Add Windows-only smoke tests; gate networking under `net` feature.
- [x] Enumerate all remaining `unsafe` blocks and add explicit safety notes where missing (final sweep complete; residual wrappers in `src/ffi/` are documented or gated).

## Risks

- Behavioral changes if lifetimes or ownership were previously unsound; mitigate via incremental refactors and tests.
- Windows-only test coverage; ensure default build remains clean without features.

## Rollout

- Phase 1: Inventory + helpers with no call-site changes. [Done]
- Phase 2: Adopt helpers in core modules; keep legacy util as shim for now. [Done]
- Phase 3: Expand to `profile.rs`, `launch/`, `net.rs`, `capability.rs` (feature-gated); Capability Catalog implemented for AppContainer capability discovery. [Done]
- Phase 4: Documentation sweep and finalize tests. [Done]

## Status Log

- 2025-10-23: Phase 3 - Capability Catalog implemented.
- 2025-10-24: Phase 3 - Capability Catalog verified via parity, shape, and smoke tests.

## Verification

- 2025-10-23: `cargo clippy --all-targets --all-features -- -D warnings`
- 2025-10-23: `cargo test --features net`
- 2025-10-24: `cargo test --all-features --target x86_64-pc-windows-msvc`
- 2025-10-24: `RAPPCT_ITESTS=1 cargo test --features net --target x86_64-pc-windows-msvc --test cap_smoke`
