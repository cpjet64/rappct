# TODO.md - work plan for rappct

## v0.1 skeleton (this repo)
- [x] Project scaffolding, modules, and docs
- [x] Public API signatures
- [x] Example CLI stub (`acrun`)
- [x] Error enums and result type
- [x] Feature flags

## v0.2 (Windows implementations core)
- [x] Implement `profile::ensure`, `delete`, `folder_path`, `named_object_path`
  - `CreateAppContainerProfile`, `DeriveAppContainerSidFromAppContainerName`
  - `GetAppContainerFolderPath` (CoTaskMemFree), `GetAppContainerNamedObjectPath`
- [x] Implement `capability::derive_named_capability_sids` (LocalFree) + KnownCapability mapping
- [x] Implement `launch`:
  - STARTUPINFOEX + PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES
  - LPAC via `ALL_APPLICATION_PACKAGES_POLICY = OPT_OUT` (gated by `supports_lpac()`)
  - Stdio: inherit/null, env, cwd
  - RAII attribute-list wrapper
- [x] Implement `token` queries:
  - `TokenIsAppContainer`, `TokenIsLessPrivilegedAppContainer`, `TokenAppContainerSid`, `TokenCapabilities`
- [x] Implement `acl` helpers:
  - Files/dirs: SetNamedSecurityInfo (DACL_SECURITY_INFORMATION)
  - Registry: SetSecurityInfo for registry keys
- [x] Implement `net` (feature: net):
  - `list_appcontainers`, `add/remove_loopback_exemption` with forced ack
- [x] Add `supports_lpac()` using `RtlGetVersion` (Win10 1703+)

## v0.3 (diagnostics, jobs, polish)
- [x] `diag::validate_configuration` warnings
- [x] Job objects for resource limits (memory, cpu); `kill_on_job_close` supported via `launch_in_container_with_io` JobGuard
- [x] `whoami --json` improvements, token dump (example compiles)
- [x] Capability name suggestion via `strsim` (optional dep)
- [x] Add minimal LPAC presets: `with_lpac_defaults()`

## v0.4 (tests & docs)
- [x] Integration tests for network isolation (loopback blocked by default)
- [x] Integration tests for ACL grant flows
- [x] README capability cheat sheet and migration notes (see README "Capability cheat sheet & migration notes")
- [x] CI matrix: Windows Server 2019/2022, Windows 10/11
  - Windows Server 2019 (1809) x64, MSVC stable toolchain, feature sets: `default`, `net`, `introspection`
  - Windows Server 2022 (21H2) x64, MSVC stable toolchain, full feature combo `net,introspection,tracing`
  - Windows 10 22H2 x64, MSVC stable toolchain, run README examples + unit tests
  - Windows 11 23H2 x64, MSVC stable + nightly (for future async work), ensure `acrun --help`, `whoami --json`

