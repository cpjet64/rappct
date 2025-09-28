# rule-breakers.md

All RULES.md requirements are currently satisfied.
- Rule 1 (No magic): `supports_lpac()` now depends solely on runtime OS checks.
- Rule 5 (RAII for everything): SID and LocalAlloc lifetimes are managed through `LocalFreeGuard` / `FreeSidGuard` helpers across profile, capability, launch, net, token, and tests.
- Rule 7 (Docs are part of the surface): Loopback helpers are documented in both the API (`src/net.rs`) and README.
