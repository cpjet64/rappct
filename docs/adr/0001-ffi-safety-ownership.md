# ADR 0001: FFI Safety & Ownership Boundaries

Status: Proposed

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

- [ ] Enumerate all `unsafe` blocks and FFI calls (per file above).
- [ ] Introduce/verify `FreeSidGuard`, `CoTaskMemGuard`, `HandleGuard` (or equivalents) in a shared module.
- [ ] Replace ad-hoc frees/closes with guards.
- [ ] Add Rustdoc for invariants and safety notes.
- [ ] Add unit tests for guard behaviors and edge cases.
- [ ] Add smoke tests in `tests/` honoring feature gates.

## Risks

- Behavioral changes if lifetimes or ownership were previously unsound; mitigate via incremental refactors and tests.
- Windows-only test coverage; ensure default build remains clean without features.

## Rollout

- Phase 1: Inventory + helpers with no call-site changes.
- Phase 2: Adopt helpers in `util.rs`, `token.rs`.
- Phase 3: Expand to `profile.rs`, `launch/`, `net.rs` (feature-gated).
- Phase 4: Documentation sweep and finalize tests.

