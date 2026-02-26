# Profile Module (`src/profile.rs`)

## Purpose

Manages AppContainer profile lifecycle and SID derivation helpers used by the rest of the crate.

## Key Types and Functions

- `AppContainerProfile`
- `derive_sid_from_name(name: &str) -> Result<AppContainerSid>`

## Responsibilities

- Ensure/open/delete AppContainer profiles.
- Resolve package SID and profile-related Windows paths.
- Provide stable profile identity input for capabilities and launch APIs.

## Typical Flow

```rust
use rappct::{AppContainerProfile, Result};

fn create_profile() -> Result<AppContainerProfile> {
    AppContainerProfile::ensure("rappct.sample", "rappct", Some("sample profile"))
}
```

## Related Docs

- [Capability Module](./capability.md)
- [Launch Module](./launch.md)
- [Rustdoc: profile module](../../target/doc/rappct/profile/index.html)
