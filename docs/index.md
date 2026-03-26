# Documentation Index

This index is the canonical entry point for local project documentation.

## Navigation

- [Docs README](./README.md)
- [Tooling and Regeneration](./TOOLING.md)
- [Module Index](./modules/index.md)

## Quickstart Commands

```powershell
# format/lint/test
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets

# optional feature validation
cargo test --all-targets --features net
cargo test --all-targets --features introspection
cargo test --all-targets --features "net introspection tracing serde"
```

## Feature Matrix

| Feature | Default | Purpose | Key Modules |
| --- | --- | --- | --- |
| `default` | yes (empty set) | Core AppContainer/LPAC library surface | `profile`, `capability`, `launch`, `acl`, `sid`, `token` |
| `net` | no | Firewall loopback helper APIs for AppContainer identities | `net` |
| `introspection` | no | Configuration diagnostics and capability suggestion helpers | `diag` |
| `tracing` | no | Structured logging support via `tracing` crate | cross-cutting |
| `serde` | no | Serialization support for selected public types | `capability`, `sid` |

## Module Documentation

- [Module Overview](./modules/index.md)
- [Profile Module](./modules/profile.md)
- [Capability Module](./modules/capability.md)
- [Launch Module](./modules/launch.md)
- [ACL Module](./modules/acl.md)
- [Token and SID Modules](./modules/token-sid.md)
- [Diagnostics and Network Modules](./modules/diag-net.md)
- [FFI and Utility Modules](./modules/ffi-util.md)

## Generated Documentation Outputs

- rustdoc (generated): [`target/doc/rappct/index.html`](../target/doc/rappct/index.html)
- mdBook (generated): [`docs/book/index.html`](./book/index.html)

## High-Level Component Map

This project structure is rendered below in plain text for maximum compatibility:

- `profile` -> `capability`
  - `sid` and `capability` both feed into capability selection and security context assembly.
- `capability` -> `launch`
- `sid` -> `acl`
- `token` -> `launch`
- `launch` -> `acl`
- `diag` (under `introspection`) -> `capability`
- `net` feature -> `profile` helpers and lifecycle integration

## Regeneration Entry Point

For exact prerequisites and one-command regeneration flow, see [TOOLING.md](./TOOLING.md).
