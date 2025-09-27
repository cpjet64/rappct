# TODO (PowerShell) - Final Notes

## Remaining Work
- [ ] Resolve `CreateProcessW` failures when running `cargo test --features net,introspection,tracing -- --nocapture`. Each launch test returns ERROR_INVALID_HANDLE and ends with STATUS_HEAP_CORRUPTION when tracing is enabled.

## Reproduction Steps
- `cargo test --features tracing --test windows_launch -- --nocapture`
- `cargo test --features net,introspection,tracing -- --nocapture`

## Observations
- Failures occur inside `launch_impl` after `UpdateProcThreadAttribute`, hinting that handle lists/attribute memory handling still needs adjustment when tracing is enabled.
- Without the `tracing` feature, all launch tests pass.

## Next Actions
- Instrument handle list setup to inspect inherited handles when tracing is on.
- Verify attribute list lifetimes and CloseHandle usage after launch failure paths.
