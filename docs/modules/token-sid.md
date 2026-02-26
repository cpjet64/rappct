# Token and SID Modules (`src/token.rs`, `src/sid.rs`)

## Purpose

Expose SID wrappers and token introspection for understanding current process security context.

## Key Types and Functions

- `AppContainerSid`
- `SidAndAttributes`
- `TokenInfo`
- `query_current_process_token()`

## Responsibilities

- Represent SIDs in ergonomic Rust types.
- Query token AppContainer/LPAC state and capability membership.
- Feed identity metadata into ACL and launch workflows.

## Related Docs

- [ACL Module](./acl.md)
- [Capability Module](./capability.md)
- [Rustdoc: sid module](../../target/doc/rappct/sid/index.html)
- [Rustdoc: token module](../../target/doc/rappct/token/index.html)
