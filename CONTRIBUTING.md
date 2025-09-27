# Contributing to rappct

**Goals:** small, sharp, and correct. No policy language, no broker in the crate.

## Ground rules
- Keep all `unsafe` in minimal, well-documented leaf functions.
- No `unwrap()` in library code.
- Platform-gate everything Windows-specific with `#[cfg(windows)]` and provide
  `UnsupportedPlatform` errors elsewhere.
- Prefer RAII over manual resource management (handles, SIDs, attribute lists).
- Add tests alongside features (unit + integration). Gate integration tests by features.
- Document memory ownership and which Windows deallocator applies (LocalFree, FreeSid, CoTaskMemFree).

## Adding a KnownCapability
- Update the mapping table in `capability.rs`.
- Add tests for the mapping and for `UnknownCapability` handling.

## Submitting PRs
- Open an issue first for significant changes.
- Include a short rationale and links to official Windows docs.
