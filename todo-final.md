# TODO (PowerShell) - Final Notes

## Remaining Work
- [x] Resolve `CreateProcessW` failures when running `cargo test --features net,introspection,tracing -- --nocapture`. Each launch test returns ERROR_INVALID_HANDLE and ends with STATUS_HEAP_CORRUPTION when tracing is enabled (fixed).

## Reproduction Steps
- `cargo test --features tracing --test windows_launch -- --nocapture`
- `cargo test --features net,introspection,tracing -- --nocapture`

## Observations
- Root cause: attribute buffers and handle lists were freed immediately after setup, leaving `CreateProcessW` with dangling pointers when tracing was enabled.
- Added instrumented logging for inherited handles to cross-check state when tracing is active.

## Resolution
- Introduced `AttributeContext` RAII owner to keep `SECURITY_CAPABILITIES`, handle lists, and LPAC policy memory valid until after `CreateProcessW`.
- Updated launch cleanup to drop the context only after process creation and removed redundant manual `LocalFree` calls.

## Verification
- `cargo test --features tracing --test windows_launch -- --nocapture`
- `cargo test --features net,introspection,tracing -- --nocapture`

## Next Actions
- None; item completed.
