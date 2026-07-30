# FFI and Utility Modules (`src/ffi/*`, `src/util.rs`)

## Purpose

Summarize internal safety infrastructure and compatibility helpers used by public modules.

## FFI (`src/ffi/*`)

The crate-private `ffi` module contains RAII wrappers for Windows resources:

- Handles (`CloseHandle`)
- Local allocations (`LocalFree`)
- SID and security capability memory
- Attribute list lifecycle for process launch setup

These wrappers isolate unsafe cleanup behavior from higher-level modules.

## Utility (`src/util.rs`)

The public utility module contains UTF-16 conversion helpers. The legacy
`OwnedHandle`, `LocalFreeGuard`, and `FreeSidGuard` types were removed for
0.14.0; consumers should use standard owned handles or the corresponding
`windows` crate ownership APIs.

## Related Docs

- [Launch Module](./launch.md)
- [Profile Module](./profile.md)
- [Rustdoc: util module](../../target/doc/rappct/util/index.html)
