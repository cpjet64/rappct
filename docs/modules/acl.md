# ACL Module (`src/acl.rs`)

## Purpose

Provides helpers to grant filesystem and registry permissions to AppContainer package SIDs or capability SIDs.

## Key Types and Functions

- `ResourcePath`
- `AccessMask`
- `AceInheritance`
- `grant_to_package(...)`
- `grant_to_capability(...)`

## Responsibilities

- Apply DACL entries to files, directories, and supported registry roots.
- Keep permission grants explicit and auditable in container setup paths.

## Constraints

- Registry targets are limited to `HKCU\...` and `HKLM\...`.
- Paths must exist before grant operations.

## Related Docs

- [Token and SID Modules](./token-sid.md)
- [Launch Module](./launch.md)
- [Rustdoc: acl module](../../target/doc/rappct/acl/index.html)
