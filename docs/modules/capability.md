# Capability Module (`src/capability.rs`)

## Purpose

Builds and validates `SECURITY_CAPABILITIES` inputs by deriving capability SIDs from known or named capability entries.

## Key Types and Functions

- `CapabilityName` / `KnownCapability`
- `CapabilityCatalog`
- `SecurityCapabilities`
- `SecurityCapabilitiesBuilder`
- `derive_named_capability_sids(names: &[&str])`

## Responsibilities

- Map capability names to SID-backed attributes.
- Compose AppContainer SID + capability SIDs into launch-ready security settings.
- Support LPAC default capability presets when explicitly enabled.

## Typical Flow

```rust
use rappct::{KnownCapability, SecurityCapabilitiesBuilder};

fn build_caps(profile_sid: &rappct::AppContainerSid) -> rappct::Result<rappct::SecurityCapabilities> {
    SecurityCapabilitiesBuilder::new(profile_sid)
        .with_known(&[KnownCapability::InternetClient])
        .with_lpac_defaults() // optional, LPAC-specific defaults
        .build()
}
```

## Related Docs

- [Profile Module](./profile.md)
- [Launch Module](./launch.md)
- [Rustdoc: capability module](../../target/doc/rappct/capability/index.html)
