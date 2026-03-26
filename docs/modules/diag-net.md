# Diagnostics and Network Modules (`src/diag.rs`, `src/net.rs`)

## Purpose

Document feature-gated operational helpers:

- `diag` for configuration validation (`introspection` feature)
- `net` for loopback exemption controls (`net` feature)

## Diagnostics (`introspection`)

Key APIs:

- `validate_configuration(...)`
- `ConfigWarning`

Use when you need actionable warnings about capability and launch configuration before runtime.

## Network Helpers (`net`)

Key APIs:

- `list_appcontainers()`
- `add_loopback_exemption(...)`
- `remove_loopback_exemption(...)`
- `LoopbackExemptionGuard`

Use only in development/debug workflows. Loopback exemption changes host firewall policy.

## Related Docs

- [Feature Matrix](../index.md#feature-matrix)
- [Rustdoc: diag module](../../target/doc/rappct/diag/index.html)
- [Rustdoc: net module](../../target/doc/rappct/net/index.html)
